use crate::buffer::ALIGN;
use std::hint::assert_unchecked;

/// Allocates memory with the global allocator.
/// This function forwards calls to the [std::alloc::alloc]
/// # Safety
/// Caller must ensure size and align are power of two.
#[inline(always)]
pub unsafe fn alloc_u8_aligned(size: usize, align: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    let ptr = std::alloc::alloc(layout);

    assert_unchecked(ptr.is_aligned_to(ALIGN));
    ptr
}

/// Deallocates memory with the global allocator.
/// This function forwards calls to the [std::alloc::dealloc]
/// # Safety
/// Caller must ensure size and align are the same when allocated.
#[inline(always)]
pub unsafe fn dealloc_u8_aligned(ptr: *mut u8, size: usize, align: usize) {
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    std::alloc::dealloc(ptr, layout)
}