#![feature(test)]
#![feature(portable_simd)]
extern crate core;
extern crate test;

use std::hint::black_box;

use simd_http::buffer::{Buffer, BufferSlice};
use simd_http::utils::avx;

static DATA: &[u8] = b"HTTP/1.1\r\nHost: developer.mozilla.org\r\nAccept-Language: fr\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nScheme: http\r\nCache-Control: max-age=0\r\nUpgrade-Insecure-Requests: 1\r\nConnection: keep-alive\r\nSec-Ch-Ua-Arch: x86\r\nSec-Ch-Ua-Mobile: ?0\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36\r\nAccept-Encoding: gzip, deflate, br\r\nSec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nSec-Fetch-Dest: empty\r\nSec-Fetch-User: ?1\r\nSec-Fetch-User: ?1\r\nAccept-Language: en-US,en;q=0.9\r\n\r\n";
static NEEDLE: &[u8; 65] = b"-origin\r\nSec-Fetch-Mode: cors\r\nSec-Fetch-Dest: empty\r\nSec-Fetch-U";
const WHERE: usize = 406;

#[bench]
fn slice_index_of(b: &mut test::Bencher) {
    let data = black_box(DATA);
    let needle = black_box(NEEDLE);
    b.iter(|| {
        let result = black_box(data.windows(black_box(needle.len())).position(|subslice| black_box(subslice) == needle));
        assert_eq!(result, Some(WHERE));
    });
}

macro_rules! avx_search_n {
    ($name:ident,$len:literal) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let vector = black_box(DATA);
            let mut needle = [0; $len];
            needle.copy_from_slice(&NEEDLE[..$len]);
            let buffer = BufferSlice::<1024, 4096>::from_slice(vector) ;

            let buffer = black_box(buffer);
            let needle = black_box(needle);
            b.iter(|| {
                let idx = unsafe { avx::search::avx_search(&buffer, &needle) };
                assert_eq!(WHERE, idx);
            });
        }
    };
}
macro_rules! memmem_search_n {
    ($name:ident,$len:literal) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let vector = black_box(DATA);
            let mut needle = [0; $len];
            needle.copy_from_slice(&NEEDLE[..$len]);
            let buffer = BufferSlice::<1024, 4096>::from_slice(vector);
            let buffer = black_box(buffer);
            let needle = black_box(needle);
            b.iter(|| {
                let idx = memchr::memmem::find(&buffer, &needle);
                assert_eq!(Some(WHERE), idx);
            });
        }
    };
}
macro_rules! slice_window_search_n {
    ($name:ident,$len:literal) => {
        #[bench]
        fn $name(b: &mut test::Bencher) {
            let vector = black_box(DATA);
            let mut needle = [0; $len];
            needle.copy_from_slice(&NEEDLE[..$len]);
            let buffer =  BufferSlice::<1024, 4096>::from_slice(vector);
            let buffer = black_box(buffer);
            let needle = black_box(needle);
            b.iter(|| {
                let idx = buffer.windows(needle.len())
                    .position(|subslice| subslice == needle);
                assert_eq!(Some(WHERE), idx);
            });
        }
    };
}
#[bench]
fn slice_window_index_of_01(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    let needle = black_box(NEEDLE);
    b.iter(|| {
        let result = vector.iter().position(|&x| x == needle[0]);
        assert_eq!(result, Some(45));
    });
}
#[bench]
fn memmem_index_of_01(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    let needle = black_box(NEEDLE);
    b.iter(|| {
        let result = black_box(memchr::memchr(needle[0], vector));
        assert_eq!(result, Some(45));
    });
}
#[bench]
fn avx512_index_of_01(b: &mut test::Bencher) {
    let vector = black_box(DATA);
    let needle = black_box([NEEDLE[0]]);
    b.iter(|| {
        let result = black_box(unsafe { avx::search::avx_search(&vector, &needle) });
        assert_eq!(result, 45);
    });
}

slice_window_search_n!(slice_window_index_of_02, 2);
memmem_search_n!(memmem_index_of_02, 2);
avx_search_n!(avx512_index_of_02, 2);

slice_window_search_n!(slice_window_index_of_03, 3);
memmem_search_n!(memmem_index_of_03, 3);
avx_search_n!(avx512_index_of_03, 3);
//
// slice_window_search_n!(slice_window_index_of_04, 4);
// memmem_search_n!(memmem_index_of_04, 4);
// avx_search_n!(avx512_index_of_04, 4);
//
// slice_window_search_n!(slice_window_index_of_05, 5);
// memmem_search_n!(memmem_index_of_05, 5);
// avx_search_n!(avx512_index_of_05, 5);
//
// slice_window_search_n!(slice_window_index_of_06, 6);
// memmem_search_n!(memmem_index_of_06, 6);
// avx_search_n!(avx512_index_of_06, 6);
//
// slice_window_search_n!(slice_window_index_of_07, 7);
// memmem_search_n!(memmem_index_of_07, 7);
// avx_search_n!(avx512_index_of_07, 7);
//
// slice_window_search_n!(slice_window_index_of_08, 8);
// memmem_search_n!(memmem_index_of_08, 8);
// avx_search_n!(avx512_index_of_08, 8);
//
// slice_window_search_n!(slice_window_index_of_09, 9);
// memmem_search_n!(memmem_index_of_09, 9);
// avx_search_n!(avx512_index_of_09, 9);
//
// slice_window_search_n!(slice_window_index_of_10, 10);
// memmem_search_n!(memmem_index_of_10, 10);
// avx_search_n!(avx512_index_of_10, 10);
//
// slice_window_search_n!(slice_window_index_of_11, 11);
// memmem_search_n!(memmem_index_of_11, 11);
// avx_search_n!(avx512_index_of_11, 11);
//
// slice_window_search_n!(slice_window_index_of_12, 12);
// memmem_search_n!(memmem_index_of_12, 12);
// avx_search_n!(avx512_index_of_12, 12);
//
// slice_window_search_n!(slice_window_index_of_13, 13);
// memmem_search_n!(memmem_index_of_13, 13);
// avx_search_n!(avx512_index_of_13, 13);
//
// slice_window_search_n!(slice_window_index_of_14, 14);
// memmem_search_n!(memmem_index_of_14, 14);
// avx_search_n!(avx512_index_of_14, 14);
//
// slice_window_search_n!(slice_window_index_of_15, 15);
// memmem_search_n!(memmem_index_of_15, 15);
// avx_search_n!(avx512_index_of_15, 15);
//
// slice_window_search_n!(slice_window_index_of_16, 16);
// memmem_search_n!(memmem_index_of_16, 16);
// avx_search_n!(avx512_index_of_16, 16);
//
// slice_window_search_n!(slice_window_index_of_17, 17);
// memmem_search_n!(memmem_index_of_17, 17);
// avx_search_n!(avx512_index_of_17, 17);
//
// slice_window_search_n!(slice_window_index_of_18, 18);
// memmem_search_n!(memmem_index_of_18, 18);
// avx_search_n!(avx512_index_of_18, 18);
//
// slice_window_search_n!(slice_window_index_of_19, 19);
// memmem_search_n!(memmem_index_of_19, 19);
// avx_search_n!(avx512_index_of_19, 19);
//
// slice_window_search_n!(slice_window_index_of_20, 20);
// memmem_search_n!(memmem_index_of_20, 20);
// avx_search_n!(avx512_index_of_20, 20);
//
// slice_window_search_n!(slice_window_index_of_21, 21);
// memmem_search_n!(memmem_index_of_21, 21);
// avx_search_n!(avx512_index_of_21, 21);
//
// slice_window_search_n!(slice_window_index_of_22, 22);
// memmem_search_n!(memmem_index_of_22, 22);
// avx_search_n!(avx512_index_of_22, 22);
//
// slice_window_search_n!(slice_window_index_of_23, 23);
// memmem_search_n!(memmem_index_of_23, 23);
// avx_search_n!(avx512_index_of_23, 23);
//
// slice_window_search_n!(slice_window_index_of_24, 24);
// memmem_search_n!(memmem_index_of_24, 24);
// avx_search_n!(avx512_index_of_24, 24);
//
// slice_window_search_n!(slice_window_index_of_25, 25);
// memmem_search_n!(memmem_index_of_25, 25);
// avx_search_n!(avx512_index_of_25, 25);
//
// slice_window_search_n!(slice_window_index_of_26, 26);
// memmem_search_n!(memmem_index_of_26, 26);
// avx_search_n!(avx512_index_of_26, 26);
//
// slice_window_search_n!(slice_window_index_of_27, 27);
// memmem_search_n!(memmem_index_of_27, 27);
// avx_search_n!(avx512_index_of_27, 27);
//
// slice_window_search_n!(slice_window_index_of_28, 28);
// memmem_search_n!(memmem_index_of_28, 28);
// avx_search_n!(avx512_index_of_28, 28);
//
// slice_window_search_n!(slice_window_index_of_29, 29);
// memmem_search_n!(memmem_index_of_29, 29);
// avx_search_n!(avx512_index_of_29, 29);
//
// slice_window_search_n!(slice_window_index_of_30, 30);
// memmem_search_n!(memmem_index_of_30, 30);
// avx_search_n!(avx512_index_of_30, 30);
//
// slice_window_search_n!(slice_window_index_of_31, 31);
// memmem_search_n!(memmem_index_of_31, 31);
// avx_search_n!(avx512_index_of_31, 31);
//
// slice_window_search_n!(slice_window_index_of_32, 32);
// memmem_search_n!(memmem_index_of_32, 32);
// avx_search_n!(avx512_index_of_32, 32);
//
// slice_window_search_n!(slice_window_index_of_33, 33);
// memmem_search_n!(memmem_index_of_33, 33);
// avx_search_n!(avx512_index_of_33, 33);
//
// slice_window_search_n!(slice_window_index_of_34, 34);
// memmem_search_n!(memmem_index_of_34, 34);
// avx_search_n!(avx512_index_of_34, 34);
//
// slice_window_search_n!(slice_window_index_of_35, 35);
// memmem_search_n!(memmem_index_of_35, 35);
// avx_search_n!(avx512_index_of_35, 35);
//
// slice_window_search_n!(slice_window_index_of_36, 36);
// memmem_search_n!(memmem_index_of_36, 36);
// avx_search_n!(avx512_index_of_36, 36);
//
// slice_window_search_n!(slice_window_index_of_37, 37);
// memmem_search_n!(memmem_index_of_37, 37);
// avx_search_n!(avx512_index_of_37, 37);
//
// slice_window_search_n!(slice_window_index_of_38, 38);
// memmem_search_n!(memmem_index_of_38, 38);
// avx_search_n!(avx512_index_of_38, 38);
//
// slice_window_search_n!(slice_window_index_of_39, 39);
// memmem_search_n!(memmem_index_of_39, 39);
// avx_search_n!(avx512_index_of_39, 39);
//
// slice_window_search_n!(slice_window_index_of_40, 40);
// memmem_search_n!(memmem_index_of_40, 40);
// avx_search_n!(avx512_index_of_40, 40);
//
// slice_window_search_n!(slice_window_index_of_41, 41);
// memmem_search_n!(memmem_index_of_41, 41);
// avx_search_n!(avx512_index_of_41, 41);
//
// slice_window_search_n!(slice_window_index_of_42, 42);
// memmem_search_n!(memmem_index_of_42, 42);
// avx_search_n!(avx512_index_of_42, 42);
//
// slice_window_search_n!(slice_window_index_of_43, 43);
// memmem_search_n!(memmem_index_of_43, 43);
// avx_search_n!(avx512_index_of_43, 43);
//
// slice_window_search_n!(slice_window_index_of_44, 44);
// memmem_search_n!(memmem_index_of_44, 44);
// avx_search_n!(avx512_index_of_44, 44);
//
// slice_window_search_n!(slice_window_index_of_45, 45);
// memmem_search_n!(memmem_index_of_45, 45);
// avx_search_n!(avx512_index_of_45, 45);
//
// slice_window_search_n!(slice_window_index_of_46, 46);
// memmem_search_n!(memmem_index_of_46, 46);
// avx_search_n!(avx512_index_of_46, 46);
//
// slice_window_search_n!(slice_window_index_of_47, 47);
// memmem_search_n!(memmem_index_of_47, 47);
// avx_search_n!(avx512_index_of_47, 47);
//
// slice_window_search_n!(slice_window_index_of_48, 48);
// memmem_search_n!(memmem_index_of_48, 48);
// avx_search_n!(avx512_index_of_48, 48);
//
// slice_window_search_n!(slice_window_index_of_49, 49);
// memmem_search_n!(memmem_index_of_49, 49);
// avx_search_n!(avx512_index_of_49, 49);
//
// slice_window_search_n!(slice_window_index_of_50, 50);
// memmem_search_n!(memmem_index_of_50, 50);
// avx_search_n!(avx512_index_of_50, 50);
//
// slice_window_search_n!(slice_window_index_of_51, 51);
// memmem_search_n!(memmem_index_of_51, 51);
// avx_search_n!(avx512_index_of_51, 51);
//
// slice_window_search_n!(slice_window_index_of_52, 52);
// memmem_search_n!(memmem_index_of_52, 52);
// avx_search_n!(avx512_index_of_52, 52);
//
// slice_window_search_n!(slice_window_index_of_53, 53);
// memmem_search_n!(memmem_index_of_53, 53);
// avx_search_n!(avx512_index_of_53, 53);
//
// slice_window_search_n!(slice_window_index_of_54, 54);
// memmem_search_n!(memmem_index_of_54, 54);
// avx_search_n!(avx512_index_of_54, 54);
//
// slice_window_search_n!(slice_window_index_of_55, 55);
// memmem_search_n!(memmem_index_of_55, 55);
// avx_search_n!(avx512_index_of_55, 55);
//
// slice_window_search_n!(slice_window_index_of_56, 56);
// memmem_search_n!(memmem_index_of_56, 56);
// avx_search_n!(avx512_index_of_56, 56);
//
// slice_window_search_n!(slice_window_index_of_57, 57);
// memmem_search_n!(memmem_index_of_57, 57);
// avx_search_n!(avx512_index_of_57, 57);
//
// slice_window_search_n!(slice_window_index_of_58, 58);
// memmem_search_n!(memmem_index_of_58, 58);
// avx_search_n!(avx512_index_of_58, 58);
//
// slice_window_search_n!(slice_window_index_of_59, 59);
// memmem_search_n!(memmem_index_of_59, 59);
// avx_search_n!(avx512_index_of_59, 59);
//
// slice_window_search_n!(slice_window_index_of_60, 60);
// memmem_search_n!(memmem_index_of_60, 60);
// avx_search_n!(avx512_index_of_60, 60);
//
// slice_window_search_n!(slice_window_index_of_61, 61);
// memmem_search_n!(memmem_index_of_61, 61);
// avx_search_n!(avx512_index_of_61, 61);
//
// slice_window_search_n!(slice_window_index_of_62, 62);
// memmem_search_n!(memmem_index_of_62, 62);
// avx_search_n!(avx512_index_of_62, 62);
//
// slice_window_search_n!(slice_window_index_of_63, 63);
// memmem_search_n!(memmem_index_of_63, 63);
// avx_search_n!(avx512_index_of_63, 63);
//
// slice_window_search_n!(slice_window_index_of_64, 64);
// memmem_search_n!(memmem_index_of_64, 64);
// avx_search_n!(avx512_index_of_64, 64);
//
// slice_window_search_n!(slice_window_index_of_65, 65);
// memmem_search_n!(memmem_index_of_65, 65);
// avx_search_n!(avx512_index_of_65, 65);