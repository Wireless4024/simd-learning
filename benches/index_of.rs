#![feature(test)]
#![feature(portable_simd)]
extern crate core;
extern crate test;

use std::hint::black_box;

use simd_http::buffer::Buffer;
use simd_http::utils::{avx, simd};
use simd_http::utils::simd::iter::SimdFindIter;

static DATA: &[u8] = b"HTTP/1.1\r\nHost: developer.mozilla.org\r\nAccept-Language: fr\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nScheme: http\r\nCache-Control: max-age=0\r\nUpgrade-Insecure-Requests: 1\r\nConnection: keep-alive\r\nSec-Ch-Ua-Arch: x86\r\nSec-Ch-Ua-Mobile: ?0\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36\r\nAccept-Encoding: gzip, deflate, br\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nSec-Fetch-Dest: empty\r\nSec-Fetch-User: ?1\r\nSec-Fetch-User: ?1\r\nAccept-Language: en-US,en;q=0.9\r\n\r\n";
static NEEDLE: &[u8] = b"e: e";
const WHERE: usize = 514;

#[bench]
fn slice_index_of(b: &mut test::Bencher) {
    let data = black_box(DATA);
    let needle = black_box(NEEDLE);
    b.iter(|| {
        let result = black_box(data.windows(black_box(needle.len())).position(|subslice| black_box(subslice) == needle));
        assert_eq!(result, Some(WHERE));
    });
}


// #[bench]
// fn simd_index_of(b: &mut test::Bencher) {
//     let vector = black_box(DATA);
//     let needle = black_box(NEEDLE);
//     b.iter(|| {
//         let vector = Simd::<_, 64>::from_slice(vector);
//         let mut needle_array = [0; 8];
//         needle_array[..needle.len()].copy_from_slice(&needle);
//         let needle = Simd::from_array(needle_array);
//         let mask = Simd::splat(0).simd_ne(needle);
//         let result = black_box(unsafe { simd::index_of(black_box(vector), black_box(needle), black_box(mask)) });
//         assert_eq!(result, 10);
//     });
// }


#[bench]
fn memmem_index_of(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    let needle = black_box(NEEDLE);
    b.iter(|| {
        let result = black_box(memchr::memmem::find(vector, needle));
        assert_eq!(result, Some(WHERE));
    });
}

#[bench]
fn simd_index_of_single_iter(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    let needle = black_box(NEEDLE);
    b.iter(|| {
        let mut buffer = unsafe { Buffer::allocate_unchecked(vector.len() as u32, 4096) };
        buffer.copy_from_slice(vector);
        let mut needle = unsafe { SimdFindIter::new(&buffer, &needle) };
        assert_eq!(Some(WHERE), needle.next());
    });
}

#[bench]
fn avx512_index_of(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    let needle = black_box(NEEDLE);
    let mut buffer = unsafe { Buffer::allocate_unchecked(1024, 4096) };
    buffer.copy_from_slice(vector);
    let buffer = black_box(buffer);
    b.iter(|| {
        let idx = unsafe { avx::search::index_of4(&buffer, needle) };
        assert_eq!(WHERE, idx);
    });
}


const ITER_SEARCH: &[u8] = b"hello world hello world hello world";
const ITER_NEEDLE: &[u8] = b"hello";
#[bench]
fn index_of_iter(b: &mut test::Bencher) {
    let haystack = black_box(ITER_SEARCH);
    let needle = black_box(ITER_NEEDLE);

    b.iter(|| {
        let mut iter = haystack.windows(needle.len())
            .enumerate()
            .filter(|(idx, subslice)| *subslice == needle)
            .map(|(idx, _)| idx);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(12));
        assert_eq!(iter.next(), Some(24));
        assert_eq!(iter.next(), None);
    });
}

#[bench]
fn memmem_index_of_iter(b: &mut test::Bencher) {
    let haystack = black_box(ITER_SEARCH);
    let needle = black_box(ITER_NEEDLE);

    b.iter(|| {
        let mut iter = black_box(memchr::memmem::find_iter(haystack, &needle));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(12));
        assert_eq!(iter.next(), Some(24));
        assert_eq!(iter.next(), None);
    });
}

#[bench]
fn simd_index_of_iter(b: &mut test::Bencher) {
    let vector = black_box(ITER_SEARCH);
    let needle = black_box(ITER_NEEDLE);
    b.iter(|| {
        unsafe {
            let mut buf = Buffer::allocate_unchecked(vector.len() as u32, 4096);
            buf.copy_from_slice(vector);
            buf.set_len(64);
            let mut iter = simd::iter::SimdFindIter::new(&buf, needle);
            assert_eq!(iter.next(), Some(0));
            assert_eq!(iter.next(), Some(12));
            assert_eq!(iter.next(), Some(24));
            assert_eq!(iter.next(), None);
        }
    });
}