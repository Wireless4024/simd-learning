use std::arch::x86_64::{_mm512_cmpeq_epi8_mask, _mm512_load_epi64, _mm512_set1_epi8};
use std::ptr::copy_nonoverlapping;

use crate::utils::avx;

/// # Safety
/// caller must ensure LANES >= NEEDLE, and if vector read might be out of bounds.
#[inline]
pub unsafe fn index_of8(haystack: &[u8], needle: &[u8]) -> usize {
    let mut ptr = haystack.as_ptr() as *const i64;
    let mut position = 0;
    let head = _mm512_set1_epi8(needle.as_ptr().read() as i8);
    let mask = avx::mask_false_i8x8(needle.len() as u32);
    let mut needle64 = [0; 8];
    copy_nonoverlapping(needle.as_ptr(), needle64.as_mut_ptr(), needle.len());
    let needle64 = u64::from_le_bytes(needle64);
    while position < haystack.len() {
        // mark last element as 0 to avoid out of bounds
        // let vector = (!mask_false(mask.to_bitmask().leading_zeros() as usize - 1)).select(vector, Simd::splat(0));
        let vector = _mm512_load_epi64(ptr);
        let mut idx = _mm512_cmpeq_epi8_mask(vector, head);
        while idx != 0 {
            let occurrence = idx.trailing_zeros() as usize;
            //let chunk = Simd::load_select_ptr(haystack.as_array().as_ptr().add(occurrence), mask, needle);
            let chunk = ptr.cast::<u8>().add(occurrence).cast::<u64>().read_unaligned();
            if (chunk & mask) == needle64 {
                return position + occurrence;
            };
            idx ^= 1 << occurrence;
        }

        position += 64;
        ptr = ptr.add(8);
    }
    haystack.len()
}