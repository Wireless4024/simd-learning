#![feature(portable_simd)]
#![feature(hasher_prefixfree_extras)]
#![feature(pointer_is_aligned_to)]
#![feature(const_intrinsic_copy)]
#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(const_eval_select)]
#![feature(const_pointer_is_aligned)]
#![feature(stdarch_x86_avx512)]
#![feature(strict_provenance)]
#![feature(const_mut_refs)]

pub mod buffer;
mod parser;
pub mod utils;
mod parts;
pub mod limit;
pub mod offset;