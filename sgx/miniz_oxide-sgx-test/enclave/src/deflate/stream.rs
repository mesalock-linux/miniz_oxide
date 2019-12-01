use miniz_oxide::deflate::stream::deflate;
use miniz_oxide::deflate::core::CompressorOxide;
use miniz_oxide::inflate::decompress_to_vec_zlib;
use miniz_oxide::{MZFlush, MZStatus};
//#[test]
use std::prelude::v1::*;

pub fn test_state() {
    let data = b"Hello zlib!";
    let mut compressed = vec![0; 50];
    let mut compressor = Box::<CompressorOxide>::default();
    let res = deflate(&mut compressor, data, &mut compressed, MZFlush::Finish);
    let status = res.status.expect("Failed to compress!");
    let decomp =
        decompress_to_vec_zlib(&compressed).expect("Failed to decompress compressed data");
    assert_eq!(status, MZStatus::StreamEnd);
    assert_eq!(decomp[..], data[..]);
    assert_eq!(res.bytes_consumed, data.len());
}
