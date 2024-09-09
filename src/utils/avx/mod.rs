use std::arch::x86_64::{__m512i, __mmask16, __mmask64, _mm512_add_epi64, _mm512_add_epi8, _mm512_broadcast_i64x2, _mm512_castsi256_si512, _mm512_castsi512_si256, _mm512_mask_blend_epi64, _mm512_set1_epi64, _mm512_sub_epi64, _mm_set_epi64x};

pub mod search;

#[inline]
pub const fn mask_false64(count: usize) -> __mmask64 {
    !((1 << count) - 1)
}
#[inline]
pub const fn mask_false16(count: usize) -> __mmask16 {
    !((1 << count) - 1)
}

const POW256: [u64; 8] = [
    0x00,
    0xFF,
    0xFFFF,
    0xFFFFFF,
    0xFFFFFFFF,
    0xFFFFFFFFFF,
    0xFFFFFFFFFFFF,
    0xFFFFFFFFFFFFFF,
];

#[inline(always)]
pub const fn mask_false_i8x8(count: u32) -> u64 {
    debug_assert!(count < 8);
    POW256[count as usize]
}

macro_rules! avx_mask_true {
    ($count:expr) => {
        ((1 << $count) - 1)
    }
}

pub(crate) use avx_mask_true;


/// Generate a reverse sequence of u8 from 63 to 0
/// # Return
/// [i8x64] from 63 to 0 
#[inline(always)]
pub fn decrement_u8() -> __m512i {
    /// # Safety
    /// Hardware support for AVX512 is required.
    unsafe {
        // 16i8 x 64
        let _16 = _mm512_set1_epi64(0x1010101010101010);
        // 32i8 x 64
        let _32 = _mm512_add_epi64(_16, _16);
        // 32i8 x 32
        let _32_256i = _mm512_castsi512_si256(_32);

        // 32i8 x 32 + 0i8 x32
        let _32_half = _mm512_castsi256_si512(_32_256i);
        // 48i8 x 32 + 16i8 x32
        let _48_half_16_half = _mm512_add_epi64(_32_half, _16);
        // 48i8 x 16 + 32i8 x 16 + 16i8 x 16 + 0i8 x16
        let dec_64 = _mm512_mask_blend_epi64(0b00_11_00_11, _32_half, _48_half_16_half);
        // sequence of 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15
        let dec_1 = _mm512_broadcast_i64x2(_mm_set_epi64x(0x01020304050607, 0x08090A0B0C0D0E0F));
        _mm512_add_epi8(dec_64, dec_1)
    }
}

/// Generate a sequence of u8 from 0 to 63\
/// # Return
/// [i8x64] from 0 to 63
#[inline(always)]
pub fn increment_u8() -> __m512i {
    /// # Safety
    /// Hardware support for AVX512 is required.
    unsafe {
        // 16i8 x 64
        let _16 = _mm512_set1_epi64(0x1010101010101010);
        // 32i8 x 64
        let _32 = _mm512_add_epi64(_16, _16);

        // 32i8 x 32
        let _32_256i = _mm512_castsi512_si256(_32);

        // 32i8 x 32 + 0i8 x32
        let _32_half = _mm512_castsi256_si512(_32_256i);
        // 0i8 x32 + 32i8 x32
        let _0_half_32_half = _mm512_sub_epi64(_32, _32_half);
        // 16i8 x 32 + 48i8 x32
        let _16_half_48_half = _mm512_add_epi64(_0_half_32_half, _16);

        let inc_64 = _mm512_mask_blend_epi64(0b00_11_00_11, _16_half_48_half, _0_half_32_half);

        let inc_1 = _mm512_broadcast_i64x2(_mm_set_epi64x(0x0F0E0D0C0B0A0908, 0x0706050403020100));
        _mm512_add_epi8(inc_64, inc_1)
    }
}