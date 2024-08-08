use std::alloc::Layout;
use std::ffi::CStr;
use std::hint::assert_unchecked;
use std::ops::{Deref, DerefMut};
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};
use std::simd::cmp::SimdPartialEq;

use monoio::buf::{IoBuf, IoBufMut};

use crate::utils::alloc::{alloc_u8_aligned, dealloc_u8_aligned};

pub mod buffer_pool;

pub const ALIGN: usize = 4096;

#[derive(Debug)]
#[repr(align(16))]
pub struct Buffer {
    ptr: *mut u8,
    offset: u32,
    len: u32,
    capacity: u32,
    align: u32,
}

impl Buffer {
    pub fn allocate(len: u32) -> Option<Self> {
        let len = len.next_power_of_two();
        if len == 0 {
            None
        } else {
            // # Safety
            // Len is power of two.
            Some(unsafe { Self::allocate_unchecked(len, ALIGN) })
        }
    }

    /// # Safety
    pub unsafe fn allocate_unchecked(len: u32, align: usize) -> Self {
        let ptr = alloc_u8_aligned(len as usize, align);
        Self {
            ptr,
            len,
            offset: 0,
            capacity: len,
            align: ALIGN as _,
        }
    }

    pub const fn layout(&self) -> Layout {
        unsafe { Layout::from_size_align_unchecked(self.len as _, self.align as _) }
    }

    /// # Safety
    /// This function is unsafe because it does not check the length of the buffer.
    pub unsafe fn set_len(&mut self, len: u32) {
        self.len = len;
    }

    /// # Safety
    /// This function is unsafe because it does not check the length of the buffer.
    pub unsafe fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub const fn ptr(&self) -> *mut u8 {
        unsafe { self.ptr.add(self.offset as usize) }
    }

    pub fn copy_from_slice(&mut self, data: &[u8]) {
        unsafe {
            let len = (self.capacity as usize).min(data.len());
            std::ptr::copy(data.as_ptr(), self.ptr(), len);
            self.offset = 0;
            self.len = len as u32;
        }
    }

    pub fn set_empty(&mut self) {
        self.len = 0;
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        self.len = self.capacity;
    }

    pub fn flip(&mut self) {
        self.offset = 0;
    }

    pub fn put(&mut self, value: &[u8]) -> u32 {
        unsafe {
            let len = self.len;
            let remain = (self.capacity - self.offset).min(value.len() as u32);
            std::ptr::copy(value.as_ptr(), self.ptr(), remain as usize);
            self.offset += remain;
            self.len += remain;
            remain
        }
    }

    /// # Safety
    /// This function is unsafe because it does not check the length of the buffer.
    /// but allow alignment optimization such as SIMD to do aligned load.
    #[inline]
    pub const unsafe fn assert_aligned(self, size: usize, align: usize) -> Buffer {
        assert_unchecked(self.ptr.is_aligned_to(align));
        assert_unchecked(self.capacity as usize == size);
        self
    }

    /// # Safety
    /// Caller must ensure block * LANES is less than the buffer length.
    #[inline]
    pub unsafe fn load_simd<T, const LANES: usize>(&self, block: usize) -> Simd<T, LANES>
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
    #[inline]
    pub unsafe fn store_simd<const LANES: usize>(&self, block: usize, value: Simd<u8, LANES>)
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
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            dealloc_u8_aligned(self.ptr, self.len as usize, self.align as usize);
        }
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe {
            std::slice::from_raw_parts(self.ptr(), self.len as _)
        }
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len as _) }
    }
}

unsafe impl IoBuf for Buffer {
    fn read_ptr(&self) -> *const u8 {
        self.ptr()
    }

    fn bytes_init(&self) -> usize {
        (self.len - self.offset) as _
    }
}
unsafe impl IoBufMut for Buffer {
    fn write_ptr(&mut self) -> *mut u8 {
        self.ptr()
    }

    fn bytes_total(&mut self) -> usize {
        (self.capacity - self.offset) as _
    }

    unsafe fn set_init(&mut self, pos: usize) {
        self.len = self.offset + pos as u32;
    }
}