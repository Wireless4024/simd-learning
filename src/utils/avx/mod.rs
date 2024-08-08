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
#[inline]
pub const fn mask_false_i8x8(count: u32) -> u64 {
    ((256u64.pow(count)) - 1)
}