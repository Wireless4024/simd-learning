use std::intrinsics::const_eval_select;
use std::simd::{LaneCount, Mask, Simd, SupportedLaneCount};
use std::simd::cmp::SimdPartialEq;

pub mod aligned;
pub mod iter;
mod search;

#[inline]
fn pad_right_zero_runtime<const LANES: usize>(simd: Simd<u8, LANES>, to_index: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let mask = mask_false(to_index);
    mask.select(Simd::splat(0), simd)
}

#[inline]
const fn pad_right_zero_comptime<const LANES: usize>(simd: Simd<u8, LANES>, to_index: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let source = simd.as_array();
    let mut array = [0; LANES];
    let mut i = 0;
    while i < to_index {
        array[i] = source[i];
        i += 1;
    }
    while i < 64 {
        array[i] = 0;
        i += 1;
    }
    Simd::from_array(array)
}

#[inline]
pub const fn pad_right_zero<const LANES: usize>(simd: Simd<u8, LANES>, to_index: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    const_eval_select((simd, to_index), pad_right_zero_comptime, pad_right_zero_runtime)
}


#[inline]
fn pad_left_zero_runtime<const LANES: usize>(simd: Simd<u8, LANES>, to_index: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let mask = mask_false(to_index);
    mask.select(simd, Simd::splat(0))
}

#[inline(always)]
pub fn mask_false<const LANES: usize>(count: usize) -> Mask<i8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    Mask::from_bitmask(u64::MAX - ((1 << count) - 1))
}

#[inline]
const fn pad_left_zero_comptime<const LANES: usize>(simd: Simd<u8, LANES>, to_index: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    let source = simd.as_array();
    let mut array = [0; LANES];
    let mut i = 0;
    while i < to_index {
        array[i] = 0;
        i += 1;
    }
    while i < 64 {
        array[i] = source[i];
        i += 1;
    }
    Simd::from_array(array)
}

#[inline]
pub const fn pad_left_zero<const LANES: usize>(simd: Simd<u8, LANES>, to_index: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    const_eval_select((simd, to_index), pad_left_zero_comptime, pad_left_zero_runtime)
}

#[inline]
pub fn move_left_zero_end<const LANES: usize>(simd: Simd<u8, LANES>, amount: usize) -> Simd<u8, LANES>
where
    LaneCount<LANES>: SupportedLaneCount,
{
    Simd::load_or_default(&simd.as_array()[amount..])
}


#[inline]
pub fn start_with<const LANES: usize>(vector: Simd<u8, LANES>, needle: Simd<u8, LANES>, length: u32) -> bool
where
    LaneCount<LANES>: SupportedLaneCount,
{
    vector.simd_ne(needle).to_bitmask().trailing_zeros() == length
}


/// # Safety
/// caller must ensure LANES >= NEEDLE, and if vector read might be out of bounds.
#[inline(never)]
pub unsafe fn index_of<const LANES: usize, const NEEDLE: usize>(vector: Simd<u8, LANES>, needle: Simd<u8, NEEDLE>, mask: Mask<i8, NEEDLE>) -> usize
where
    LaneCount<LANES>: SupportedLaneCount,
    LaneCount<NEEDLE>: SupportedLaneCount,
{
    let head = Simd::splat(needle.as_array()[0]);
    // mark last element as 0 to avoid out of bounds
   // let vector = (!mask_false(mask.to_bitmask().leading_zeros() as usize - 1)).select(vector, Simd::splat(0));
    let mut idx = vector.simd_eq(head).to_bitmask();
    while idx != 0 {
        let occurrence = idx.trailing_zeros() as usize;
        let chunk = Simd::load_select_ptr(vector.as_array().as_ptr().add(occurrence), mask, needle);
        let cmp = mask.select(chunk, needle).simd_eq(needle);
        if cmp.all() {
            return occurrence;
        }
        idx ^= 1 << occurrence;
    }

    LANES
}