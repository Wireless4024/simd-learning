use std::hint::assert_unchecked;
use std::simd::cmp::SimdPartialEq;
use std::simd::{u8x8, Simd};

use crate::utils::simd;

const MAX_NEEDLE_SIZE: usize = 8;
const PROCESS_SIZE: usize = 64;

pub struct SimdFindIter<'a> {
    source: &'a [u8],
    needle: &'a [u8],
    match_index: u64,
    position: isize,
}

impl<'a> SimdFindIter<'a> {
    /// # Safety
    /// `aligned_slice` must be aligned to 64 bytes at lease 64 bytes long, and length divisible by 64 for full aligned load.
    pub unsafe fn new(aligned_slice: &'a [u8], needle: &'a [u8]) -> Self {
        // still check in debug mode
        assert_unchecked(needle.len() <= MAX_NEEDLE_SIZE);
        assert_unchecked(aligned_slice.len() >= needle.len());
        assert_unchecked(aligned_slice.as_ptr().is_aligned_to(PROCESS_SIZE));

        Self {
            source: aligned_slice,
            match_index: 0,
            position: -(PROCESS_SIZE as isize),
            needle,
        }
    }
}

impl<'a> Iterator for SimdFindIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            assert_unchecked(self.source.as_ptr().is_aligned_to(PROCESS_SIZE));
            assert_unchecked(self.needle.len() <= MAX_NEEDLE_SIZE);
            assert_unchecked(!self.needle.is_empty());
        }
        let mask = !simd::mask_false(self.needle.len());
        let needle = unsafe { Simd::load_select_unchecked(self.needle, mask, Simd::splat(0)) };
        'start: loop {
            while self.match_index != 0 {
                let occurrence = self.match_index.trailing_zeros() as isize;
                self.match_index ^= 1 << occurrence;
                let pos = ((self.position) + occurrence) as usize;
                let data = unsafe { u8x8::load_select_unchecked(&self.source[pos..], mask, needle) };
                if data.simd_eq(needle).all() {
                    return Some(pos);
                }
            }
            let prefix = Simd::splat(unsafe { self.needle.as_ptr().read() });
            let mut pos = (self.position as usize) + PROCESS_SIZE;
            let mut match_index = 0;
            while match_index == 0 && pos < self.source.len() {
                let head = unsafe { self.source.as_ptr().add(pos).cast::<Simd<u8, PROCESS_SIZE>>().read() };
                match_index = head.simd_eq(prefix).to_bitmask();
                pos += PROCESS_SIZE;
                if match_index != 0 {
                    self.position = pos as isize - PROCESS_SIZE as isize;
                    self.match_index = match_index;
                    continue 'start;
                }
            }
            break None;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::{Buffer, BufferSlice};

    use super::*;

    #[test]
    fn test() {
        let needle = b"hello";
        let haystack = BufferSlice::<4096, 4096>::from_slice(b"hello world hello world hello world");
        unsafe {
            let mut iter = SimdFindIter::new(&haystack, needle);
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), Some(12));
            assert_eq!(iter.next(), Some(24));
            assert_eq!(iter.next(), None);
        }
    }
}