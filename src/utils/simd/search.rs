use std::simd::{LaneCount, Simd, SupportedLaneCount};

use crate::utils::simd::mask_false;

pub fn find_slice<const LANES: usize, const NEEDLE: usize>(vector: &[u8], needle: Simd<u8, NEEDLE>, mask: Simd<i8, NEEDLE>) -> usize
where
    LaneCount<LANES>: SupportedLaneCount,
    LaneCount<NEEDLE>: SupportedLaneCount,
{
 
    LANES
}