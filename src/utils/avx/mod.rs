use std::arch::x86_64::{__mmask16, __mmask64};

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
    POW256[count as usize]
}

macro_rules! avx_mask_true {
    ($count:expr) => {
        ((1 << $count) - 1)
    }
}

pub(crate) use avx_mask_true;