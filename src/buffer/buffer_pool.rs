use std::cell::UnsafeCell;
use std::collections::VecDeque;

use super::Buffer;

pub struct BufferPool<const SIZE: usize, const ALIGN: usize> {
    buffers: UnsafeCell<VecDeque<Buffer<SIZE, ALIGN>>>,
    // !Send
    _marker: std::marker::PhantomData<*mut ()>,
}

impl BufferPool<0, 4096> {
    pub const fn new4k<const SIZE: usize>() -> BufferPool<SIZE, 4096> {
        BufferPool::default()
    }
}

impl<const SIZE: usize, const ALIGN: usize> BufferPool<SIZE, ALIGN> {
    pub const fn default() -> Self {
        Self {
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
            buffer.push_back(Buffer::allocate_unchecked());
        }
    }

    #[allow(clippy::mut_from_ref)]
    unsafe fn buffer(&self) -> &mut VecDeque<Buffer<SIZE, ALIGN>> {
        &mut *self.buffers.get()
    }

    /// # Safety
    /// Caller must ensure size and align are power of two.
    /// Take buffer with size and alignment garanteed to be the same as the pool.
    pub unsafe fn take(&self) -> Buffer<SIZE, ALIGN> {
        if let Some(buffer) = self.buffer().pop_front() {
            buffer
        } else {
            Buffer::allocate_unchecked()
        }
    }

    /// # Safety
    /// buffer must have same size and alignment as the pool.
    pub unsafe fn put(&self, buffer: Buffer<SIZE, ALIGN>) {
        self.buffer().push_back(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_allocate() {
        let pool = BufferPool::new4k::<8192>();
        unsafe {
            pool.pre_allocate(10);
            assert_eq!(pool.buffer().len(), 10);
        }
    }
}