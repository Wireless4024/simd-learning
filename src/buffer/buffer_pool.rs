use std::cell::UnsafeCell;
use std::collections::VecDeque;
use std::hint::assert_unchecked;

use super::Buffer;

pub struct BufferPool {
    size: u32,
    align: u32,
    buffers: UnsafeCell<VecDeque<Buffer>>,
    // !Send
    _marker: std::marker::PhantomData<*mut ()>,
}

impl BufferPool {
    pub const fn new(size: u32, align: usize) -> Self {
        Self {
            size: size.next_power_of_two(),
            align: align as u32,
            buffers: UnsafeCell::new(VecDeque::new()),
            _marker: std::marker::PhantomData,
        }
    }

    /// # Safety
    /// Caller must ensure size and align are power of two.
    #[inline]
    pub unsafe fn pre_allocate(&self, amount: usize) {
        let buffer = self.buffer();
        buffer.reserve(amount);
        for _ in buffer.len()..amount {
            buffer.push_back(Buffer::allocate_unchecked(self.size, self.align as usize));
        }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn buffer(&self) -> &mut VecDeque<Buffer> {
        &mut *self.buffers.get()
    }

    /// # Safety
    /// Caller must ensure size and align are power of two.
    /// Take buffer with size and alignment garanteed to be the same as the pool.
    pub unsafe fn take(&self) -> Buffer {
        if let Some(buffer) = self.buffer().pop_front() {
            buffer.assert_aligned(self.size as usize, self.align as usize)
        } else {
            Buffer::allocate_unchecked(self.size, self.align as usize)
                .assert_aligned(self.size as usize, self.align as usize)
        }
    }

    /// # Safety
    /// buffer must have same size and alignment as the pool.
    pub unsafe fn put(&self, buffer: Buffer) {
        self.buffer().push_back(buffer);
    }

    /// # Safety
    /// This function is unsafe because it does not check the length of the buffer.
    /// but allow alignment optimization such as SIMD to do aligned load.
    #[inline]
    pub const unsafe fn assert_aligned(&self, size: usize, align: usize) {
        assert_unchecked(self.size as usize == size);
        assert_unchecked(self.align as usize == align);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_allocate() {
        let pool = BufferPool::new(4096, 4096);
        unsafe {
            pool.pre_allocate(10);
            assert_eq!(pool.buffer().len(), 10);
        }
    }

    #[test]
    fn test_take() {
        let pool = BufferPool::new(4096, 4096);
        unsafe {
            pool.pre_allocate(10);
            for _ in 0..10 {
                assert_eq!(pool.take().len(), 4096);
            }
        }

        let pool = BufferPool::new(4096, 4096);
        unsafe {
            pool.pre_allocate(10);
            for _ in 0..10 {
                assert_eq!(pool.take().len(), 4096);
                pool.put(pool.take());
            }
            assert_eq!(pool.buffer().len(), 0);
        }
    }
}