#![feature(test)]
#![feature(portable_simd)]
extern crate test;

use std::hint::black_box;
use std::simd::Simd;

use simd_http::utils::ascii;

const DATA: [u8; 32] = *b"HTTP/1.1\r\nHost: developer.mozill";
const DATA_LOWERCASE: [u8; 32] = *b"http/1.1\r\nhost: developer.mozill";

#[bench]
fn simd_lowercase(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    b.iter(|| {
        let vector = Simd::from_array(vector);
        let result = black_box(ascii::simd_lowercase(vector));
        assert_eq!(result.as_array(), &DATA_LOWERCASE);
    });
}

#[bench]
fn lowercase(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    b.iter(|| {
        let mut out = black_box(vector);
        out.make_ascii_lowercase();
        assert_eq!(out, DATA_LOWERCASE);
    });
}