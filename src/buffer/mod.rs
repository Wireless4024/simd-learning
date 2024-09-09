use monoio::buf::{IoBuf, IoBufMut};
use std::alloc::Layout;
use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::hint::assert_unchecked;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut, Index};
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

use crate::utils::alloc::{alloc_u8_aligned, dealloc_u8_aligned};

pub mod buffer_pool;

pub const ALIGN: usize = 4096;

#[derive(Debug)]
#[repr(transparent)]
pub struct Buffer<const LEN: usize, const ALIGN: usize = 4096> {
    ptr: *mut u8,
}

impl<const LEN: usize> Buffer<LEN, 4096> {
    #[inline(always)]
    pub fn allocate4k() -> Buffer<LEN, 4096> {
        Buffer::allocate()
    }
}

impl<const LEN: usize, const ALIGN: usize> Buffer<LEN, ALIGN> {
    #[inline(always)]
    pub fn allocate() -> Self {
        if !ALIGN.is_power_of_two() {
            let next_power_of_two = ALIGN.next_power_of_two();
            panic!("ALIGN was {ALIGN} but must be power of two, next power of two is {next_power_of_two}");
        }
        unsafe { Self::allocate_unchecked() }
    }

    /// # Safety
    #[inline(always)]
    pub unsafe fn allocate_unchecked() -> Self {
        let ptr = alloc_u8_aligned(LEN, ALIGN);
        std::ptr::write_bytes(ptr, 0, LEN);
        Self {
            ptr,
        }
    }

    #[inline(always)]
    pub fn fill(&mut self, value: u8) {
        unsafe {
            std::ptr::write_bytes(self.ptr, value, LEN);
        }
    }

    #[inline(always)]
    pub const fn layout(&self) -> Layout {
        unsafe { Layout::from_size_align_unchecked(LEN, ALIGN) }
    }

    pub fn copy_from_slice(&mut self, data: &[u8]) {
        unsafe {
            let len = LEN.min(data.len());
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, len);
        }
    }

    /// # Safety
    /// Caller must ensure block * LANES is less than the buffer length.
    #[inline(always)]
    pub const unsafe fn load_simd<T, const LANES: usize>(&self, block: usize) -> Simd<T, LANES>
    where
        LaneCount<LANES>: SupportedLaneCount,
        T: SimdElement,
    {
        let ptr = self.ptr.add(size_of::<T>() * LANES * block);
        assert_unchecked(ptr.is_aligned_to(LANES));
        let arr = *(ptr as *const [T; LANES]);
        Simd::from_array(arr)
    }

    /// # Safety
    /// Caller must ensure block * LANES is less than the buffer length.
    #[inline(always)]
    pub const unsafe fn store_simd<const LANES: usize>(&self, block: usize, value: Simd<u8, LANES>)
    where
        LaneCount<LANES>: SupportedLaneCount,
    {
        let ptr = self.ptr.add(LANES * block);
        assert_unchecked(ptr.is_aligned_to(LANES));
        let arr = value.as_array();
        std::ptr::copy_nonoverlapping(arr.as_ptr(), ptr, LANES);
    }

    #[inline]
    pub unsafe fn cstr_len<const LANES: usize>(&self, block: usize, count: usize) -> usize
    where
        LaneCount<LANES>: SupportedLaneCount,
    {
        let mut counter = 0;
        while counter < count {
            let vector = self.load_simd::<u8, LANES>(block);
            let mask = vector.simd_eq(Simd::splat(0));
            if let Some(idx) = mask.first_set() {
                return idx + (counter * LANES);
            }
            counter += 1;
        }

        count * LANES
    }

    pub unsafe fn load_cstr<const LANES: usize>(&self, block: u32, count: u32) -> &CStr
    where
        LaneCount<LANES>: SupportedLaneCount,
    {
        let block = block as usize;
        let count = count as usize;
        let len = self.cstr_len::<LANES>(block, count) + 1;
        let ptr = self.ptr.add(LANES * block);
        CStr::from_bytes_with_nul_unchecked(std::slice::from_raw_parts(ptr, len))
    }

    #[inline(always)]
    pub const fn ptr_cast<T>(&self) -> *mut T {
        self.ptr as *mut T
    }

    #[inline(always)]
    pub const fn ptr_size<T>(&self) -> usize {
        LEN / size_of::<T>()
    }

    #[inline(always)]
    pub const fn slice(self, offset: usize) -> BufferSlice<LEN, ALIGN> {
        BufferSlice::new(self, offset as _, (LEN - offset) as _)
    }

    #[inline(always)]
    pub const unsafe fn with_alignment<const NEW_ALIGN: usize>(self) -> Buffer<LEN, NEW_ALIGN> {
        let ptr = self.ptr;
        assert_unchecked(ptr.is_aligned_to(NEW_ALIGN));
        let _ = ManuallyDrop::new(self);
        Buffer {
            ptr,
        }
    }
}

impl<const LEN: usize, const ALIGN: usize> Drop for Buffer<LEN, ALIGN> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            dealloc_u8_aligned(self.ptr, LEN, ALIGN);
        }
    }
}

pub struct BufferSlice<const LEN: usize, const ALIGN: usize> {
    buffer: Buffer<LEN, ALIGN>,
    offset: u32,
    len: u32,
}

impl<const LEN: usize, const ALIGN: usize> Debug for BufferSlice<LEN, ALIGN> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.iter())
            .finish()
    }
}

impl<const LEN: usize, const ALIGN: usize> BufferSlice<LEN, ALIGN> {
    pub const fn new(buffer: Buffer<LEN, ALIGN>, offset: u32, len: u32) -> Self {
        Self { buffer, offset, len }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        let mut buffer = Buffer::<LEN, ALIGN>::allocate();
        let len = slice.len().min(LEN);
        buffer.copy_from_slice(slice);
        Self::new(buffer, 0, len as _)
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self)
    }

    #[inline(always)]
    pub fn buffer(&self) -> &Buffer<LEN, ALIGN> {
        &self.buffer
    }

    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset as _
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as _
    }

    #[inline]
    pub fn set_len(&mut self, len: u32) {
        if len > self.capacity() as u32 {
            panic!("len {len} is greater than capacity {}", self.capacity());
        }
        unsafe { self.set_len_unchecked(len); }
    }

    #[inline(always)]
    pub unsafe fn set_len_unchecked(&mut self, len: u32) {
        self.len = len;
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        LEN - self.offset()
    }

    #[inline(always)]
    pub unsafe fn ptr(&self) -> *mut u8 {
        self.buffer.ptr.add(self.offset as _)
    }

    #[inline(always)]
    pub fn into_inner(self) -> Buffer<LEN, ALIGN> {
        self.buffer
    }
}

impl<const LEN: usize, const ALIGN: usize> Deref for BufferSlice<LEN, ALIGN> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.buffer.ptr.add(self.offset()), self.len()) }
    }
}

impl<const LEN: usize, const ALIGN: usize> DerefMut for BufferSlice<LEN, ALIGN> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.buffer.ptr.add(self.offset()), self.len()) }
    }
}

unsafe impl<const LEN: usize, const ALIGN: usize> IoBuf for BufferSlice<LEN, ALIGN> {
    #[inline(always)]
    fn read_ptr(&self) -> *const u8 {
        unsafe { self.ptr() }
    }

    #[inline(always)]
    fn bytes_init(&self) -> usize {
        self.len as _
    }
}

unsafe impl<const LEN: usize, const ALIGN: usize> IoBufMut for BufferSlice<LEN, ALIGN> {
    #[inline(always)]
    fn write_ptr(&mut self) -> *mut u8 {
        unsafe { self.ptr() }
    }

    #[inline(always)]
    fn bytes_total(&mut self) -> usize {
        self.capacity()
    }

    #[inline(always)]
    unsafe fn set_init(&mut self, pos: usize) {
        self.set_len_unchecked(pos as _);
    }
}