use std::arch::asm;
use std::arch::x86_64::{_mm512_cmpeq_epi8_mask, _mm512_load_epi64, _mm512_set1_epi8};
use std::hint::assert_unchecked;
use std::ops::Add;
use std::simd::{u8x4, Mask};

/// # Safety
/// caller must ensure LANES >= NEEDLE, and if vector read might be out of bounds.
/// and needle must be 4 bytes length
#[inline(never)]
pub unsafe fn index_of4(haystack: &[u8], needle: &[u8]) -> usize {
    let len = haystack.len();
    let end = haystack.as_ptr().add(len);
    assert_unchecked(len >= 64);
    assert_unchecked(needle.len() == 4);

    let mut ptr = haystack.as_ptr() as *const i64;
    assert_unchecked(ptr.is_aligned_to(64));
    let needle = u8x4::load_select_ptr(needle.as_ptr(), Mask::splat(true), Default::default());
    // let mask = avx::mask_false_i8x8(needle.len() as u32);
    // let needle_ptr = needle.as_ptr();
    // let mut needle64 = [0; 8];
    // copy_nonoverlapping(needle.as_ptr(), needle64.as_mut_ptr(), needle.len());
    // let needle64 = u64::from_le_bytes(needle64);
    let needle_head = needle[0];
    let head = _mm512_set1_epi8(needle_head as i8);
    loop {
        // mark last element as 0 to avoid out of bounds
        // let vector = (!mask_false(mask.to_bitmask().leading_zeros() as usize - 1)).select(vector, Simd::splat(0));
        let vector = _mm512_load_epi64(ptr);
        let mut idx = _mm512_cmpeq_epi8_mask(vector, head);
        while idx != 0 {
            let occurrence = idx.trailing_zeros() as usize;
            //let chunk = Simd::load_select_ptr(haystack.as_array().as_ptr().add(occurrence), mask, needle);
            let chunk_ptr = ptr.cast::<u8>().add(occurrence)/*.cast::<u64>().read_unaligned()*/;
            let chunk = u8x4::load_select_ptr(chunk_ptr, Mask::splat(true), Default::default());

            if chunk == needle {
                return chunk_ptr.addr() - haystack.as_ptr().addr();
            }

            // idx ^= 1 << occurrence;
            asm!("btc {},{}", inout(reg) idx, in(reg) occurrence);
        }

        //ptr = ptr.add(16);
        asm!("add {},64", inout(reg) ptr);
        if ptr.cast() > end {
            break;
        }
    }
    len
}