use crate::utils::avx::avx_mask_true;
use std::arch::asm;
use std::arch::x86_64::{__mmask64, _kshiftli_mask64, _mm512_cmpeq_epi8_mask, _mm512_load_epi64, _mm512_mask_cmpeq_epi8_mask, _mm512_set1_epi8};
use std::hint::assert_unchecked;
use std::simd::Mask;

macro_rules! avx_search {
    ($name:ident,$lanes:literal) => {
        /// # Safety
        /// Haystack must be 64 bytes aligned and length must be >= 64 + needle.len() and divisible by 64.
        /// you can use any u8 that not included in needle as padding.
        #[inline(never)]
        unsafe fn $name(haystack: &[u8], needle: &[u8]) -> usize {
            let len = haystack.len();
            let end = haystack.as_ptr().add(len - needle.len());
            assert_unchecked(len >= 64);
            assert_unchecked(needle.len() == $lanes);

            let mut ptr = haystack.as_ptr() as *const i64;
            assert_unchecked(ptr.is_aligned_to(64));
            let needle = std::simd::Simd::<u8, $lanes>::load_select_ptr(needle.as_ptr(), Mask::splat(true), Default::default());
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
                    let chunk = std::simd::Simd::<u8, $lanes>::load_select_ptr(chunk_ptr, Mask::splat(true), Default::default());

                    if chunk == needle {
                        return chunk_ptr.addr() - haystack.as_ptr().addr();
                    }

                    // idx ^= 1 << occurrence;
                    asm!("btc {},{}", inout(reg) idx, in(reg) occurrence);
                }

                //ptr = ptr.add(bytes);
                asm!("add {},64", inout(reg) ptr);
                if ptr.cast() > end {
                    break;
                }
            }
            len
        }
    }
}
macro_rules! avx_search_sub_1 {
    ($name:ident,$lanes:literal) => {
        /// # Safety
        /// Haystack must be 64 bytes aligned and length must be >= 64 + needle.len() and divisible by 64.
        /// you can use any u8 that not included in needle as padding.
        #[inline(never)]
        unsafe fn $name(haystack: &[u8], needle: &[u8]) -> usize {
            let len = haystack.len();
            let end = haystack.as_ptr().add(len - needle.len());
            assert_unchecked(len >= 64);
            assert_unchecked(needle.len() == $lanes + 1);

            let needle_head = needle[0];
            let head = _mm512_set1_epi8(needle_head as i8);
            let mut ptr = haystack.as_ptr() as *const i64;
            assert_unchecked(ptr.is_aligned_to(64));
            let needle = std::simd::Simd::<u8, $lanes>::load_select_ptr(needle.as_ptr().add(1), Mask::splat(true), Default::default());

            loop {
                let vector = _mm512_load_epi64(ptr);
                let mut idx = _mm512_cmpeq_epi8_mask(vector, head);
                while idx != 0 {
                    let occurrence = idx.trailing_zeros() as usize;

                    let chunk_ptr = ptr.cast::<u8>().add(occurrence + 1);
                    let chunk = std::simd::Simd::<u8, $lanes>::load_select_ptr(chunk_ptr, Mask::splat(true), Default::default());

                    if chunk == needle {
                        return (chunk_ptr.addr()-1) - haystack.as_ptr().addr();
                    }

                    // idx ^= 1 << occurrence;
                    asm!("btc {},{}", inout(reg) idx, in(reg) occurrence);
                }

                //ptr = ptr.add(bytes);
                asm!("add {},64", inout(reg) ptr);
                if ptr.cast() > end {
                    break;
                }
            }
            len
        }
    }
}

macro_rules! avx_search_padded {
    ($name:ident,$lanes:literal) => {
        /// # Safety
        /// Haystack must be 64 bytes aligned and length must be >= 64 + needle.len() and divisible by 64.
        /// you can use any u8 that not included in needle as padding.
        #[inline(never)]
        unsafe fn $name(haystack: &[u8], needle: &[u8]) -> usize {
            const LEN_HALF:usize = $lanes / 2;
            let len = haystack.len();
            let end = haystack.as_ptr().add(len - needle.len());
            assert_unchecked(len >= 64);
            assert_unchecked(needle.len() >= ((LEN_HALF) + 1) && needle.len() < $lanes);
            let mut ptr = haystack.as_ptr() as *const i64;
            assert_unchecked(ptr.is_aligned_to(64));

            let mask = avx_mask_true!((needle.len()-LEN_HALF));
            let mask = Mask::from_bitmask(mask);

            let needle_low = std::simd::Simd::<u8, LEN_HALF>::load_select_ptr(needle.as_ptr(), Mask::splat(true), Default::default());
            let needle_high = std::simd::Simd::<u8, LEN_HALF>::load_select_ptr(needle.as_ptr().add(LEN_HALF), mask, Default::default());
            let head = _mm512_set1_epi8(needle[0] as i8);
            loop {
                let vector = _mm512_load_epi64(ptr);
                let mut idx = _mm512_cmpeq_epi8_mask(vector, head);
                while idx != 0 {
                    let occurrence = idx.trailing_zeros() as usize;
                    let chunk_ptr = ptr.cast::<u8>().add(occurrence);
                    let chunk_low = std::simd::Simd::<u8, LEN_HALF>::load_select_ptr(chunk_ptr, Mask::splat(true), Default::default());
                    let chunk_high = std::simd::Simd::<u8, LEN_HALF>::load_select_ptr(chunk_ptr.add(LEN_HALF), mask, Default::default());

                    if chunk_low == needle_low && chunk_high == needle_high {
                        return chunk_ptr.addr() - haystack.as_ptr().addr();
                    }
                    asm!("btc {0},{1}", inout(reg) idx, in (reg) occurrence);
                }
                asm!("add {0},64", inout(reg) ptr);
                if ptr.cast() > end {
                    break len;
                }
            }
        }
    };
}

// needle.len() == 2
//avx_search!(index_of2, 2);
// needle.len() == 3
avx_search_sub_1!(index_of3, 2);
// needle.len() == 4
avx_search!(index_of4, 4);
// needle.len() == 5
avx_search_sub_1!(index_of5, 4);
// needle.len() > 5 && needle.len() < 8
avx_search_padded!(index_of6_lt8, 8);
// needle.len() == 8
avx_search!(index_of8, 8);
// needle.len() == 9
avx_search_sub_1!(index_of9, 8);
// needle.len() > 9 && needle.len() < 16
avx_search_padded!(index_of10_lt16, 16);
avx_search!(index_of16, 16);
// needle.len() == 17
avx_search_sub_1!(index_of17, 16);
// needle.len() > 17 && needle.len() < 32
avx_search_padded!(index_of18_lt32, 32);
avx_search!(index_of32, 32);
// needle.len() == 33
avx_search_sub_1!(index_of33, 32);
// needle.len() > 33 && needle.len() < 64
avx_search_padded!(index_of34_lt64, 64);
avx_search!(index_of64, 64);
avx_search_sub_1!(index_of65, 64);

/// Perform O(n) search for needle length <= 65 bytes. this method is phantom, 
/// and will remove and generate best search function for specific needle size.
/// # Safety
/// Haystack must be 64 bytes aligned and length must be >= 64 + needle.len() and divisible by 64.
/// you can use any u8 that not included in needle as padding.
#[inline(always)]
pub unsafe fn avx_search<const NEEDLE_SIZE: usize>(haystack: &[u8], needle: &[u8; NEEDLE_SIZE]) -> usize {
    // compiler will choose best function for specific needle size
    if NEEDLE_SIZE > 65 {
        // don't have yet :D
        rt_search(haystack, needle)
    } else if NEEDLE_SIZE == 65 {
        index_of65(haystack, needle)
    } else if NEEDLE_SIZE == 64 {
        index_of64(haystack, needle)
    } else if NEEDLE_SIZE > 33 {
        index_of34_lt64(haystack, needle)
    } else if NEEDLE_SIZE == 33 {
        index_of33(haystack, needle)
    } else if NEEDLE_SIZE == 32 {
        index_of32(haystack, needle)
    } else if NEEDLE_SIZE > 17 {
        index_of18_lt32(haystack, needle)
    } else if NEEDLE_SIZE == 17 {
        index_of17(haystack, needle)
    } else if NEEDLE_SIZE == 16 {
        index_of16(haystack, needle)
    } else if NEEDLE_SIZE > 9 {
        index_of10_lt16(haystack, needle)
    } else if NEEDLE_SIZE == 9 {
        index_of9(haystack, needle)
    } else if NEEDLE_SIZE == 8 {
        index_of8(haystack, needle)
    } else if NEEDLE_SIZE > 5 {
        index_of6_lt8(haystack, needle)
    } else if NEEDLE_SIZE == 5 {
        index_of5(haystack, needle)
    } else if NEEDLE_SIZE == 4 {
        index_of4(haystack, needle)
    } else if NEEDLE_SIZE == 3 {
        index_of3(haystack, needle)
    } else if NEEDLE_SIZE == 2 {
        index_of2(haystack, needle)
    } else if NEEDLE_SIZE == 1 {
        index_of(haystack, needle[0])
    } else {
        rt_search(haystack, needle)
    }
}

#[inline(never)]
fn rt_search(haystack: &[u8], needle: &[u8]) -> usize {
    unsafe { assert_unchecked(haystack.as_ptr().is_aligned_to(64)) };
    memchr::memmem::find(haystack, needle).unwrap_or(haystack.len())
}

/// # Safety
/// Haystack must be 64 bytes aligned and length must be >= 64.
pub unsafe fn index_of(haystack: &[u8], needle: u8) -> usize {
    let len = haystack.len();
    let end = haystack.as_ptr().add(len);
    assert_unchecked(len >= 64);
    let mut ptr = haystack.as_ptr() as *const i64;
    assert_unchecked(ptr.is_aligned_to(64));

    let head = _mm512_set1_epi8(needle as i8);
    loop {
        // mark last element as 0 to avoid out of bounds
        // let vector = (!mask_false(mask.to_bitmask().leading_zeros() as usize - 1)).select(vector, Simd::splat(0));
        let vector = _mm512_load_epi64(ptr);
        let idx = _mm512_cmpeq_epi8_mask(vector, head);
        if idx != 0 {
            let occurrence = idx.trailing_zeros() as usize;
            return (ptr.addr() - haystack.as_ptr().addr()) + occurrence;
        }

        asm!("add {},64", inout(reg) ptr);
        if ptr.cast() > end {
            break;
        }
    }
    len
}

// last bit
const CARRY_MASK: __mmask64 = 1 << 63;

/// # Safety
/// Haystack must be 64 bytes aligned and length must be >= 64.
#[inline(never)]
pub unsafe fn index_of2(haystack: &[u8], needle: &[u8]) -> usize {
    let len = haystack.len();
    let end = haystack.as_ptr().add(len);
    assert_unchecked(len >= 64);
    let mut ptr = haystack.as_ptr() as *const i64;
    assert_unchecked(ptr.is_aligned_to(64));
    assert_unchecked(needle.len() == 2);
    let mut carry: __mmask64 = 0;

    let head = _mm512_set1_epi8(needle[0] as i8);
    let tail = _mm512_set1_epi8(needle[1] as i8);
    loop {
        let vector = _mm512_load_epi64(ptr);
        let idx = _mm512_cmpeq_epi8_mask(vector, head);
        let mask = _kshiftli_mask64::<1>(idx) | carry;
        carry = if idx & CARRY_MASK == 0 { 0 } else { 1 };
        let cmp = _mm512_mask_cmpeq_epi8_mask(mask, vector, tail);
        let idx = mask & cmp;
        if idx != 0 {
            let occurrence = idx.trailing_zeros() as usize;

            return (ptr.addr() - haystack.as_ptr().addr()) + (occurrence - 1);
        }

        asm!("add {},64", inout(reg) ptr);
        if ptr.cast() > end {
            break;
        }
    }
    len
}