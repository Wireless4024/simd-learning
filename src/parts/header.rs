use crate::utils::ascii::simd_lowercase;
use crate::utils::simd::aligned::Aligned32;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::hint::assert_unchecked;
use std::mem::transmute;
use std::ops::{Deref, DerefMut};
use std::simd::num::SimdUint;
use std::simd::Simd;

// AVX2
const MAX_HEADER_KEY_LENGTH: usize = 32;
type InnerValue = Aligned32;

#[derive(Eq)]
pub struct AlignedHeaderKey(pub InnerValue);

impl Deref for AlignedHeaderKey {
    type Target = [u8; MAX_HEADER_KEY_LENGTH];

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
        Self(InnerValue::default())
    }


    /// # Note
    /// This function will strip value longer than [MAX_HEADER_KEY_LENGTH] bytes.
    #[inline]
    pub const fn new(value: &[u8]) -> Self {
        let mut key = AlignedHeaderKey::default();
        unsafe {
            std::ptr::copy_nonoverlapping(value.as_ptr(), key.0.0.as_mut_ptr(), value.len());
            let lowered = simd_lowercase(Simd::from_array(key.0.0));
            std::ptr::copy_nonoverlapping(lowered.as_array().as_ptr(), key.0.0.as_mut_ptr(), lowered.len());
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

    /// bytes must have [MAX_HEADER_KEY_LENGTH] bits alignment
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        // # Safety
        // value is [MAX_HEADER_KEY_LENGTH] bits aligned from AlignedHeaderKey and has [MAX_HEADER_KEY_LENGTH] bytes length.
        unsafe {
            assert_unchecked(bytes.as_ptr().is_aligned_to(MAX_HEADER_KEY_LENGTH));
            assert_unchecked(bytes.len() == MAX_HEADER_KEY_LENGTH);
        }
        let mut buffer = [0u8; MAX_HEADER_KEY_LENGTH];
        buffer.copy_from_slice(bytes);
        let vector = Simd::from_array(buffer);
        unsafe {
            self.0 = transmute::<_, Simd<u64, { MAX_HEADER_KEY_LENGTH / 8 }>>(vector).reduce_sum() + self.0;
        }
    }

    fn write_length_prefix(&mut self, len: usize) {
        self.0 = len.swap_bytes() as u64;
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