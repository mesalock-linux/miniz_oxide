use miniz_oxide::inflate::decompress_to_vec_zlib;
//#[test]
pub fn decompress_vec() {
    let encoded = [
        120, 156, 243, 72, 205, 201, 201, 215, 81, 168, 202, 201, 76, 82, 4, 0, 27, 101, 4, 19,
    ];
    let res = decompress_to_vec_zlib(&encoded[..]).unwrap();
    assert_eq!(res.as_slice(), &b"Hello, zlib!"[..]);
}

pub mod stream;
pub mod core;
