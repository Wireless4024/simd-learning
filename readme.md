# SIMD learning project to parse HTTP request
this project will SIGSEGV when you run it in debug mode.
still unfinished, I will come back later.

# Goal
- [ ] learn how to utilize SIMD instruction to bulk process multiple data
- [ ] custom hash function for http header : http header don't need DOS resistance
- [ ] buffer reuse : allocation is expensive
- [ ] thread per core model (no Arc&lt;Mutex&lt;T&gt;&gt;): mfence is expensive

# Note
- utils are contains buffer overrun (but buffer are over allocated to prevent SEGFAULT in release mode)
- require nightly rust and avx512 cpu to run

# Reason learn
- Portable SIMD will cause slowdown when T*LANES is more than data path
  - Zen 4 (double pump avx512) will have less performance compare to avx2
  - if compile target to below AVX2 will likely have slightly lower performance than hand rolled SIMD