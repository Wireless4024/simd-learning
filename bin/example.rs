#![feature(portable_simd)]
#![feature(stdarch_x86_avx512)]

use simd_http::buffer::{Buffer, BufferSlice};
use simd_http::utils::simd::{move_left_zero_end, pad_left_zero, pad_right_zero};
use std::simd::cmp::SimdPartialEq;
use std::simd::{u8x64, Simd};
use simd_http::utils::avx::search::avx_search;

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
static mut LEN: usize = 256;
fn main() {
    let buffer = Buffer::<63, 4096>::allocate();
    let request = b"GET / HTTP/1.1\r\nHost: developer.mozilla.org\r\nAccept-Language: fr\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nScheme: http\r\nCache-Control: max-age=0\r\nUpgrade-Insecure-Requests: 1\r\nConnection: keep-alive\r\nSec-Ch-Ua-Arch: x86\r\nSec-Ch-Ua-Mobile: ?0\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36\r\nAccept-Encoding: gzip, deflate, br\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nSec-Fetch-Dest: empty\r\nSec-Fetch-User: ?1\r\nSec-Fetch-User: ?1\r\nAccept-Language: en-US,en;q=0.9\r\n\r\n".as_slice();
    let mut buffer = BufferSlice::<8192, 4096>::from_slice(request);
    println!("{}", unsafe{ avx_search(&buffer, b"\r\n") });
    println!("{:?}", buffer.as_str());
    unsafe {
        println!("{:?}", parse_request(&buffer));
        read_request(&buffer.buffer());
        println!("{:?}", String::from_utf8_lossy(&buffer[21..]));
    };
}
const OFFSET64_SHIFT: usize = 6;
const PATH_OFFSET: usize = 4096 >> OFFSET64_SHIFT;
const END: usize = 8192 >> OFFSET64_SHIFT;
const METHOD_OFFSET: usize = END - 1;
const VERSION_OFFSET: usize = END - 2;
const MAX_METHOD_COUNT: usize = (4096 - 64) >> OFFSET64_SHIFT;

unsafe fn read_request<const LEN: usize, const ALIGN: usize>(request: &Buffer<LEN, ALIGN>) {
    let method = request.load_cstr::<64>(METHOD_OFFSET as u32, 1);
    let path = request.load_cstr::<64>(PATH_OFFSET as u32, 62);
    let version = request.load_cstr::<64>(VERSION_OFFSET as u32, 1);
    println!("{:?} {:?} {:?}", method, path, version);
}

#[inline]
unsafe fn parse_request<const LEN: usize, const ALIGN: usize>(request: &BufferSlice<LEN, ALIGN>) -> Result<usize, ParseError> {
    let buffer = request.buffer();
    let vector = buffer.load_simd::<u8, 64>(0);
    let cr = vector.simd_eq(Simd::splat(b'\r')).first_set().unwrap_or(64);
    let consumed = if cr < 64 {
        parse_fast(vector, cr, buffer)?
    } else {
        // TODO: parse multiple vector
        todo!();
        parse_slow(vector, cr, buffer)?
    };
    let vector = pad_left_zero(vector, consumed);
    // avoid match on consumed
    buffer.store_simd(0, vector);
    println!("{:?}", request);

    parse_header(consumed, buffer)
}

/// parse header of request in single avx512 vector
#[inline(never)]
unsafe fn parse_fast<const LEN: usize, const ALIGN: usize>(vector: u8x64, cr: usize, request: &Buffer<LEN, ALIGN>) -> Result<usize, ParseError> {
    // fill data after CR with 0
    let vector = pad_right_zero(vector, cr);
    let space = Simd::splat(b' ');
    // find matched space
    let mask = vector.simd_eq(space);
    let method_idx = mask.to_bitmask().trailing_zeros() as usize;
    let path_end = 64 - mask.to_bitmask().leading_zeros() as usize;
    // no more than 63 chars
    let method = pad_right_zero(vector, method_idx);
    request.store_simd(METHOD_OFFSET, method);
    let path = pad_right_zero(vector, path_end - 1);
    let path = move_left_zero_end(path, method_idx + 1);
    let version = move_left_zero_end(vector, path_end);
    request.store_simd(PATH_OFFSET, path);
    request.store_simd(VERSION_OFFSET, version);
    // consumed += 7 + cr;
    if path_end == method_idx {
        Err(ParseError::Path)
    } else {
        Ok(cr + 2)
    }
}


unsafe fn parse_slow<const LEN: usize, const ALIGN: usize>(vector: u8x64, cr: usize, request: &Buffer<LEN, ALIGN>) -> Result<usize, ParseError> {
    Ok(0)
}

unsafe fn parse_header<const LEN: usize, const ALIGN: usize>(shift:usize, request: &Buffer<LEN, ALIGN>) -> Result<usize, ParseError> {
    Ok(0)
}