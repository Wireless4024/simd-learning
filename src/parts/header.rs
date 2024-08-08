use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::hint::assert_unchecked;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::simd::{u64x8, u8x64};
use std::simd::num::SimdUint;

use crate::utils::ascii::simd_lowercase;
use crate::utils::simd::aligned::Aligned64;

#[derive(Eq)]
pub struct AlignedHeaderKey(pub Aligned64);

impl Deref for AlignedHeaderKey {
    type Target = [u8; 64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AlignedHeaderKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AlignedHeaderKey {
    pub const fn default() -> Self {
        Self(Aligned64::default())
    }

    #[inline]
    pub const fn new(value: &[u8]) -> Self {
        let key = AlignedHeaderKey::default();
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), key.0.0.as_ptr() as *mut u8, value.len());
            let lowered = simd_lowercase(u8x64::from_array(key.0.0));
            std::ptr::copy_nonoverlapping(lowered.as_array().as_ptr(), key.0.0.as_ptr() as *mut u8, lowered.len());
        }
        key
    }
}

#[derive(Eq, PartialEq, Hash)]
struct CustomHash<'a>(&'a AlignedHeaderKey);

impl<'a> CustomHash<'a> {
    #[inline]
    fn new(bytes: &'a AlignedHeaderKey) -> Self {
        Self(bytes)
    }
}

impl Hash for AlignedHeaderKey {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for AlignedHeaderKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Default)]
#[repr(transparent)]
struct HeaderHasher(u64);

impl Hasher for HeaderHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    /// bytes must have 64 bits alignment
    fn write(&mut self, bytes: &[u8]) {
        // # Safety
        // value is 64 bits aligned from AlignedHeaderKey and has 64 bytes length.
        unsafe {
            assert_unchecked(bytes.as_ptr().is_aligned_to(64));
            assert_unchecked(bytes.len() == 64);
        }
        let mut buffer = [0u8; 64];
        buffer.copy_from_slice(bytes);
        let vector = u8x64::from_array(buffer);
        unsafe {
            self.0 = transmute::<u8x64, u64x8>(vector).reduce_sum() + self.0.swap_bytes();
        }
    }

    fn write_length_prefix(&mut self, len: usize) {
        self.0 = len as u64;
    }
}
#[derive(Default)]
struct RandomState;
impl BuildHasher for RandomState {
    type Hasher = HeaderHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        HeaderHasher::default()
    }
}

pub struct HeaderMap<'a> {
    headers: HashMap<CustomHash<'a>, &'a [u8], RandomState>,
}

impl<'a> HeaderMap<'a> {
    pub fn new() -> Self {
        Self {
            headers: HashMap::with_hasher(RandomState),
        }
    }

    #[inline]
    pub fn insert(&mut self, key: &'a AlignedHeaderKey, value: &'a [u8]) {
        self.headers.insert(CustomHash::new(key), value);
    }

    #[inline]
    pub fn get(&self, key: &AlignedHeaderKey) -> Option<&[u8]> {
        self.headers.get(&CustomHash::new(key)).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut map = HeaderMap::new();
        unsafe {
            let key = AlignedHeaderKey::new(b"key");
            map.insert(&key, b"value1");
            assert_eq!(map.get(&AlignedHeaderKey::new(b"key")), Some(b"value1".as_slice()));
            assert_eq!(map.get(&AlignedHeaderKey::new(b"Key")), Some(b"value1".as_slice()));
            assert_eq!(map.get(&AlignedHeaderKey::new(b"KEy")), Some(b"value1".as_slice()));
            assert_eq!(map.get(&AlignedHeaderKey::new(b"KEY")), Some(b"value1".as_slice()));

            let key = AlignedHeaderKey::new(b"User-Agent");
            map.insert(&key, b"value2");
            assert_eq!(map.get(&AlignedHeaderKey::new(b"user-agent")), Some(b"value2".as_slice()));
            assert_eq!(map.get(&AlignedHeaderKey::new(b"User-AGent")), Some(b"value2".as_slice()));
            assert_eq!(map.get(&AlignedHeaderKey::new(b"USER-AGENT")), Some(b"value2".as_slice()));
        }
    }
}