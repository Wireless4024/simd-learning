#![feature(test)]
#![feature(portable_simd)]
extern crate test;

use std::hint::black_box;
use std::simd::Simd;

use simd_http::utils::simd;

const DATA: [u8; 32] = *b"HTTP/1.1\r\nHost: developer.mozill";

#[bench]
fn simd_left_pad(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    b.iter(|| {
        let vector = Simd::from_array(vector);
        let result = black_box(simd::pad_left_zero(vector, 16));
        assert_eq!(result.as_array()[..16], [0; 16]);
        assert_eq!(result.as_array()[16..], DATA[16..]);
    });
}

#[bench]
fn left_pad(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    b.iter(|| {
        let mut out = [0; 32];
        out[16..].copy_from_slice(&vector[16..]);
        let result = black_box(out);
        assert_eq!(result[..16], [0; 16]);
        assert_eq!(result[16..], DATA[16..]);
    });
}