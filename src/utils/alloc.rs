pub unsafe fn alloc_u8_aligned(size: usize, align: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    unsafe { std::alloc::alloc(layout) }
}

pub unsafe fn dealloc_u8_aligned(ptr: *mut u8, size: usize, align: usize) {
    let layout = std::alloc::Layout::from_size_align_unchecked(size, align);
    unsafe { std::alloc::dealloc(ptr, layout) }
}