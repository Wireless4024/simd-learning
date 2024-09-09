use std::arch::x86_64::{__m128, __m128d, __m128i, __m256, __m256d, __m256i, __m512, __m512d, __m512i, _mm256_load_si256, _mm512_load_epi64, _mm512_load_si512, _mm512_store_epi64, _mm_load_si128};
use std::cmp::Ordering;
use std::fmt::{Binary, Formatter, UpperHex, Write};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::simd::Simd;

macro_rules! aligned {
    ($name:ident,$align:literal,$vector:ident,$vector2:ident,$vector3:ident,$load:expr) => {
        #[derive(Eq, Copy, Clone)]
        #[repr(align($align))]
        pub struct $name(pub [u8; $align]);

        impl Hash for $name {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write(&self.0);
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.to_simd().reverse())
                    .finish()
            }
        }

        impl PartialEq for $name {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                Simd::from_array(self.0) == Simd::from_array(other.0)
            }
        }

        impl Deref for $name {
            type Target = [u8; $align];

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                self.to_simd().partial_cmp(&other.to_simd())
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> Ordering {
                self.to_simd().cmp(&other.to_simd())
            }
        }

        impl From<$vector> for $name {
            #[inline]
            fn from(vector: $vector) -> Self {
                Self(unsafe { std::mem::transmute(vector) })
            }
        }

        impl From<$vector2> for $name {
            #[inline]
            fn from(vector: $vector2) -> Self {
                Self(unsafe { std::mem::transmute(vector) })
            }
        }

        impl From<$vector3> for $name {
            #[inline]
            fn from(vector: $vector3) -> Self {
                Self(unsafe { std::mem::transmute(vector) })
            }
        }

        impl From<$name> for $vector {
            #[inline]
            fn from(value: $name) -> Self {
                unsafe { std::mem::transmute(value.0) }
            }
        }

        impl Binary for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(concat!(stringify!($name), "(["))?;
                for i in 0..($align - 1) {
                    Binary::fmt(&self.0[($align - 1) - i], f)?;
                    f.write_str(", ")?;
                }
                Binary::fmt(&self.0[0], f)?;
                f.write_str("])")
            }
        }

        impl UpperHex for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(concat!(stringify!($name), "(["))?;
                for i in 0..($align - 1) {
                    UpperHex::fmt(&self.0[($align - 1) - i], f)?;
                    f.write_str(", ")?;
                }
                UpperHex::fmt(&self.0[0], f)?;
                f.write_str("])")
            }
        }

        impl $name {
            #[inline]
            pub const fn default() -> Self {
                Self([0; $align])
            }

            #[inline]
            pub const fn to_simd(self) -> Simd<u8, $align> {
                Simd::from_array(self.0)
            }

            #[inline]
            pub fn to_vector(self) -> $vector {
                unsafe{ $load((&self.0 as *const u8).cast()) }
            }

            #[inline]
            pub fn from_slice(slice: &[u8]) -> Self {
                let mut array = [0; $align];
                array[..slice.len()].copy_from_slice(slice);
                Self(array)
            }
        }
    };
}

aligned!(Aligned16, 16, __m128i, __m128, __m128d, _mm_load_si128);
aligned!(Aligned32, 32, __m256i, __m256, __m256d, _mm256_load_si256);
aligned!(Aligned64, 64, __m512i, __m512, __m512d, _mm512_load_si512);

#[derive(PartialEq, Hash, Eq, Copy, Clone, Debug)]
#[repr(align(64))]
pub enum Aligned {
    Aligned16(Aligned16),
    Aligned32(Aligned32),
    Aligned64(Aligned64),
}
impl Aligned64 {
    pub unsafe fn load_vector(&self) -> __m512i {
        _mm512_load_epi64(self.0.as_ptr() as *const i64)
    }

    pub unsafe fn store_vector(&mut self, vector: __m512i) {
        _mm512_store_epi64(self.0.as_mut_ptr() as *mut i64, vector);
    }
}