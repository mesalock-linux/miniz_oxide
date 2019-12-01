use std::prelude::v1::*;
use miniz_oxide::deflate::core::{
    compress_to_output, create_comp_flags_from_zip_params, read_u16_le, write_u16_le,
    CompressionStrategy, CompressorOxide, TDEFLFlush, TDEFLStatus, DEFAULT_FLAGS,
};
use miniz_oxide::inflate::decompress_to_vec;
use miniz_oxide::shared::MZ_DEFAULT_WINDOW_BITS;

//#[test]
pub fn u16_to_slice() {
    let mut slice = [0, 0];
    write_u16_le(2000, &mut slice, 0);
    assert_eq!(slice, [208, 7]);
}

//#[test]
pub fn u16_from_slice() {
    let mut slice = [208, 7];
    assert_eq!(read_u16_le(&mut slice, 0), 2000);
}

//#[test]
pub fn compress_output() {
    assert_eq!(
        DEFAULT_FLAGS,
        create_comp_flags_from_zip_params(
            4,
            MZ_DEFAULT_WINDOW_BITS,
            CompressionStrategy::Default as i32
        )
    );

    let slice = [
        1, 2, 3, 4, 1, 2, 3, 1, 2, 3, 1, 2, 6, 1, 2, 3, 1, 2, 3, 2, 3, 1, 2, 3,
    ];
    let mut encoded = vec![];
    let flags = create_comp_flags_from_zip_params(6, 0, 0);
    let mut d = CompressorOxide::new(flags);
    let (status, in_consumed) =
        compress_to_output(&mut d, &slice, TDEFLFlush::Finish, |out: &[u8]| {
            encoded.extend_from_slice(out);
            true
        });

    assert_eq!(status, TDEFLStatus::Done);
    assert_eq!(in_consumed, slice.len());

    let decoded = decompress_to_vec(&encoded[..]).unwrap();
    assert_eq!(&decoded[..], &slice[..]);
}
