use miniz_oxide::deflate::{compress_to_vec, compress_to_vec_inner};
use miniz_oxide::deflate::core::CompressionStrategy;
use miniz_oxide::inflate::decompress_to_vec;
use std::prelude::v1::*;

/// Test deflate example.
///
/// Check if the encoder produces the same code as the example given by Mark Adler here:
/// https://stackoverflow.com/questions/17398931/deflate-encoding-with-static-huffman-codes/17415203
//#[test]
pub fn compress_small() {
    let test_data = b"Deflate late";
    let check = [
        0x73, 0x49, 0x4d, 0xcb, 0x49, 0x2c, 0x49, 0x55, 0x00, 0x11, 0x00,
    ];

    let res = compress_to_vec(test_data, 1);
    assert_eq!(&check[..], res.as_slice());

    let res = compress_to_vec(test_data, 9);
    assert_eq!(&check[..], res.as_slice());
}

//#[test]
pub fn compress_huff_only() {
    let test_data = b"Deflate late";

    let res = compress_to_vec_inner(test_data, 1, 0, CompressionStrategy::HuffmanOnly as i32);
    let d = decompress_to_vec(res.as_slice()).expect("Failed to decompress!");
    assert_eq!(test_data, d.as_slice());
}

/// Test that a raw block compresses fine.
//#[test]
pub fn compress_raw() {
    let text = b"Hello, zlib!";
    let encoded = {
        let len = text.len();
        let notlen = !len;
        let mut encoded = vec![
            1,
            len as u8,
            (len >> 8) as u8,
            notlen as u8,
            (notlen >> 8) as u8,
        ];
        encoded.extend_from_slice(&text[..]);
        encoded
    };

    let res = compress_to_vec(text, 0);
    assert_eq!(encoded, res.as_slice());
}

//#[test]
pub fn short() {
    let test_data = [10, 10, 10, 10, 10, 55];
    let c = compress_to_vec(&test_data, 9);

    let d = decompress_to_vec(c.as_slice()).expect("Failed to decompress!");
    assert_eq!(&test_data, d.as_slice());
    // Check that a static block is used here, rather than a raw block
    // , so the data is actually compressed.
    // (The optimal compressed length would be 5, but neither miniz nor zlib manages that either
    // as neither checks matches against the byte at index 0.)
    assert!(c.len() <= 6);
}

pub mod stream;
pub mod core;
