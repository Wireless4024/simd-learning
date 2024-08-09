#![feature(portable_simd)]
#![feature(stdarch_x86_avx512)]
extern crate core;

use std::ffi::CStr;
use std::fmt::Display;
use std::hash::Hash;
use std::hint::black_box;
use std::simd::cmp::SimdPartialEq;
use std::simd::u8x64;

use simd_http::buffer::Buffer;
use simd_http::utils::avx::mask_false_i8x8;
use simd_http::utils::avx::search::{index_of4};
use simd_http::utils::simd::{move_left_zero_end, pad_right_zero};

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("Invalid method")]
    Method,
    #[error("Invalid path")]
    Path,
    #[error("Invalid header")]
    Header,
}

#[inline(never)]
fn memmem_no_inline(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    memchr::memmem::find(haystack, needle)
}
fn main() {
    // let mut idx = 0b000010000001i32;
    // println!("{:#032b}", idx);
    // // r9
    // let pos = idx.trailing_zeros();
    // println!("{}", pos);
    // unsafe {
    //     asm!("btc {},{}", inout(reg) idx, in(reg) pos);
    //     //asm!("btc {},{}", out(reg) idx, in(reg) pos);
    // }
    // println!("{:#032b}", idx);
    //
    // let x = 0;
    // if 2 == (2 + x) {
    //     return;
    // }
    println!("{:#064b}", mask_false_i8x8(4));
    let mut src = unsafe { Buffer::allocate_unchecked(1024, 4096) };
    src.copy_from_slice(black_box(b"HTTP/1.1\r\nHost: developer.mozilla.org\r\nAccept-Language: fr\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nScheme: http\r\nCache-Control: max-age=0\r\nUpgrade-Insecure-Requests: 1\r\nConnection: keep-alive\r\nSec-Ch-Ua-Arch: x86\r\nSec-Ch-Ua-Mobile: ?0\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36\r\nAccept-Encoding: gzip, deflate, br\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nSec-Fetch-Dest: empty\r\nSec-Fetch-User: ?1\r\nSec-Fetch-User: ?1\r\nAccept-Language: en-US,en;q=0.9\r\n\r\n"));
    let cmp = b"e: e";
    unsafe {
        println!("RES:{}", index_of4(&*src, cmp));
        println!("CORRECT:{:?}", memmem_no_inline(&*black_box(src), black_box(cmp)));
    }


    let request = b"GET /hello HTTP/1.1\r\nHost: www.example.com\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n";
    let mut buffer = Buffer::allocate(8192).unwrap();
    buffer.set_empty();
    buffer.copy_from_slice(request);
    println!("{:?}", buffer);
    unsafe {
        let buffer = buffer.assert_aligned(8192, 4096);
        println!("{:?}", parse_request(&buffer));
        read_request(&buffer);
        println!("{:?}", String::from_utf8_lossy(&buffer[21..]));
    };
}
const SPACE: u8x64 = u8x64::from_array([b' '; 64]);
const CR: u8x64 = u8x64::from_array([b'\r'; 64]);
const OFFSET64_SHIFT: usize = 6;
const PATH_OFFSET: usize = 4096 >> OFFSET64_SHIFT;
const METHOD_OFFSET: usize = (8192 >> OFFSET64_SHIFT) - 1;
const VERSION_OFFSET: usize = (8192 >> OFFSET64_SHIFT) - 2;
const MAX_METHOD_COUNT: usize = (4096 - 64) >> OFFSET64_SHIFT;

unsafe fn read_request(request: &Buffer) {
    let method = request.load_cstr::<64>(METHOD_OFFSET as u32, 1);
    let path = request.load_cstr::<64>(PATH_OFFSET as u32, 62);
    let version = request.load_cstr::<64>(VERSION_OFFSET as u32, 1);
    println!("{:?} {:?} {:?}", method, path, version);
}

#[inline]
unsafe fn parse_request(request: &Buffer) -> Result<usize, ParseError> {
    let mut method = 0;
    let mut path = 0;
    let vector = request.load_simd::<u8, 64>(0);
    let cr = vector.simd_eq(CR).first_set().unwrap_or(64);
    if cr < 64 {
        return parse_fast(vector, cr, request);
    } else {
        // TODO: parse multiple vector
        todo!()
    }

    Ok(0)
}

/// parse header of request in single simd vector
#[inline]
unsafe fn parse_fast(vector: u8x64, cr: usize, request: &Buffer) -> Result<usize, ParseError> {
    let mask = vector.simd_eq(SPACE);
    let method_idx = mask.to_bitmask().trailing_zeros() as usize;
    let mut consumed = 0;
    // no more than 63 chars
    let method = pad_right_zero(vector, method_idx);
    request.store_simd(METHOD_OFFSET, method);
    consumed += method_idx + 1;
    let vector = move_left_zero_end(vector, method_idx + 1);
    let mask = vector.simd_eq(SPACE);
    let path_idx = mask.to_bitmask().trailing_zeros() as usize;
    let path = pad_right_zero(vector, path_idx);

    request.store_simd(PATH_OFFSET, path);
    consumed += path_idx;
    let vector = move_left_zero_end(vector, path_idx);
    println!("{:?}", CStr::from_bytes_until_nul(vector.as_array()));
    // HTTP/{VERSION}
    let vector = move_left_zero_end(vector, 6);
    let cr = vector.simd_eq(CR).first_set().unwrap_or(64);
    let vector = pad_right_zero(vector, cr);
    request.store_simd(VERSION_OFFSET, vector);
    // HTTP/ = 5
    // + CRLF = 7
    consumed += 7 + cr;

    Ok(consumed)
}


