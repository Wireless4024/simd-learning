use std::arch::x86_64::{__m512i, _mm512_load_epi64, _mm512_store_epi64};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::simd::Simd;

macro_rules! aligned {
    ($name:ident,$align:literal) => {
        #[derive(Eq, Copy, Clone, Debug)]
        #[repr(align($align))]
        pub struct $name(pub [u8; $align]);

        impl Hash for $name {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write(&self.0);
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

        impl $name {
            #[inline]
            pub const fn default() -> Self {
                Self([0; $align])
            }

            #[inline]
            pub const fn to_simd(self) -> Simd<u8, $align> {
                Simd::from_array(self.0)
            }
        }
    };
}

aligned!(Aligned8, 8);
aligned!(Aligned16, 16);
aligned!(Aligned32, 32);
aligned!(Aligned64, 64);

#[derive(PartialEq, Hash, Eq, Copy, Clone, Debug)]
#[repr(align(64))]
pub enum Aligned {
    Aligned8(Aligned8),
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