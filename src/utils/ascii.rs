use std::intrinsics::const_eval_select;
use std::simd::{LaneCount, Simd, SupportedLaneCount};
use std::simd::cmp::SimdPartialOrd;

#[inline]
fn simd_lowercase_runtime<const LANES: usize>(bytes: Simd<u8, LANES>) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let lower_mask = bytes.simd_ge(Simd::splat(b'A'));
    let upper_mask = bytes.simd_le(Simd::splat(b'Z'));
    let mask = lower_mask & upper_mask;
    bytes | mask.select(Simd::splat(0b100000), Simd::splat(0))
}

#[inline]
const fn simd_lowercase_comptime<const LANES: usize>(bytes: Simd<u8, LANES>) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let mut lower_mask = bytes.to_array();
    let mut i = 0;
    while i < 64 {
        lower_mask[i] = lower_mask[i].to_ascii_lowercase();
        i += 1;
    }
    Simd::from_array(lower_mask)
}

pub const fn simd_lowercase<const LANES: usize>(bytes: Simd<u8, LANES>) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    const_eval_select((bytes,), simd_lowercase_comptime, simd_lowercase_runtime)
}