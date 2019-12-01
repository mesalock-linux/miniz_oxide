use miniz_oxide::inflate::stream::{inflate, InflateState};
use miniz_oxide::{DataFormat, MZFlush, MZStatus};
//#[test]
pub fn test_state() {
    let encoded = [
        120u8, 156, 243, 72, 205, 201, 201, 215, 81, 168, 202, 201, 76, 82, 4, 0, 27, 101, 4,
        19,
    ];
    let mut out = vec![0; 50];
    let mut state = InflateState::new_boxed(DataFormat::Zlib);
    let res = inflate(&mut state, &encoded, &mut out, MZFlush::Finish);
    let status = res.status.expect("Failed to decompress!");
    assert_eq!(status, MZStatus::StreamEnd);
    assert_eq!(out[..res.bytes_written as usize], b"Hello, zlib!"[..]);
    assert_eq!(res.bytes_consumed, encoded.len());

    state.reset(DataFormat::Zlib);
    let status = res.status.expect("Failed to decompress!");
    assert_eq!(status, MZStatus::StreamEnd);
    assert_eq!(out[..res.bytes_written as usize], b"Hello, zlib!"[..]);
    assert_eq!(res.bytes_consumed, encoded.len());
}
