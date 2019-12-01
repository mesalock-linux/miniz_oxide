use std::io::Cursor;
use std::prelude::v1::*;
use miniz_oxide::inflate::core::inflate_flags::*;
use miniz_oxide::inflate::core::HuffmanTable;
use miniz_oxide::inflate::core::*;
use miniz_oxide::inflate::TINFLStatus;

//TODO: Fix these.

fn tinfl_decompress_oxide<'i>(
    r: &mut DecompressorOxide,
    input_buffer: &'i [u8],
    output_buffer: &mut [u8],
    flags: u32,
) -> (TINFLStatus, &'i [u8], usize) {
    let (status, in_pos, out_pos) =
        decompress(r, input_buffer, &mut Cursor::new(output_buffer), flags);
    (status, &input_buffer[in_pos..], out_pos)
}

//#[test]
pub fn decompress_zlib() {
    let encoded = [
        120, 156, 243, 72, 205, 201, 201, 215, 81, 168, 202, 201, 76, 82, 4, 0, 27, 101, 4, 19,
    ];
    let flags = TINFL_FLAG_COMPUTE_ADLER32 | TINFL_FLAG_PARSE_ZLIB_HEADER;

    let mut b = DecompressorOxide::new();
    const LEN: usize = 32;
    let mut b_buf = vec![0; LEN];

    // This should fail with the out buffer being to small.
    let b_status = tinfl_decompress_oxide(&mut b, &encoded[..], b_buf.as_mut_slice(), flags);

    assert_eq!(b_status.0, TINFLStatus::Failed);

    let flags = flags | TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;

    b = DecompressorOxide::new();

    // With TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF set this should no longer fail.
    let b_status = tinfl_decompress_oxide(&mut b, &encoded[..], b_buf.as_mut_slice(), flags);

    assert_eq!(b_buf[..b_status.2], b"Hello, zlib!"[..]);
    assert_eq!(b_status.0, TINFLStatus::Done);
}

//#[test]
pub fn raw_block() {
    const LEN: usize = 64;

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

    //let flags = TINFL_FLAG_COMPUTE_ADLER32 | TINFL_FLAG_PARSE_ZLIB_HEADER |
    let flags = TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;

    let mut b = DecompressorOxide::new();

    let mut b_buf = vec![0; LEN];

    let b_status = tinfl_decompress_oxide(&mut b, &encoded[..], b_buf.as_mut_slice(), flags);
    assert_eq!(b_buf[..b_status.2], text[..]);
    assert_eq!(b_status.0, TINFLStatus::Done);
}

fn masked_lookup(table: &HuffmanTable, bit_buf: BitBuffer) -> (i32, u32) {
    let ret = table.lookup(bit_buf).unwrap();
    (ret.0 & 511, ret.1)
}

//#[test]
pub fn fixed_table_lookup() {
    let mut d = DecompressorOxide::new();
    d.block_type = 1;
    start_static_table(&mut d);
    let mut l = LocalVars {
        bit_buf: d.bit_buf,
        num_bits: d.num_bits,
        dist: d.dist,
        counter: d.counter,
        num_extra: d.num_extra,
    };
    init_tree(&mut d, &mut l);
    let llt = &d.tables[LITLEN_TABLE];
    let dt = &d.tables[DIST_TABLE];
    assert_eq!(masked_lookup(llt, 0b00001100), (0, 8));
    assert_eq!(masked_lookup(llt, 0b00011110), (72, 8));
    assert_eq!(masked_lookup(llt, 0b01011110), (74, 8));
    assert_eq!(masked_lookup(llt, 0b11111101), (143, 8));
    assert_eq!(masked_lookup(llt, 0b000010011), (144, 9));
    assert_eq!(masked_lookup(llt, 0b111111111), (255, 9));
    assert_eq!(masked_lookup(llt, 0b00000000), (256, 7));
    assert_eq!(masked_lookup(llt, 0b1110100), (279, 7));
    assert_eq!(masked_lookup(llt, 0b00000011), (280, 8));
    assert_eq!(masked_lookup(llt, 0b11100011), (287, 8));

    assert_eq!(masked_lookup(dt, 0), (0, 5));
    assert_eq!(masked_lookup(dt, 20), (5, 5));
}

fn check_result(input: &[u8], expected_status: TINFLStatus, expected_state: State, zlib: bool) {
    let mut r = DecompressorOxide::default();
    let mut output_buf = vec![0; 1024 * 32];
    let mut out_cursor = Cursor::new(output_buf.as_mut_slice());
    let flags = if zlib {
        inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
    } else {
        0
    } | TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF
        | TINFL_FLAG_HAS_MORE_INPUT;
    let (d_status, _in_bytes, _out_bytes) = decompress(&mut r, input, &mut out_cursor, flags);
    assert_eq!(expected_status, d_status);
    assert_eq!(expected_state, r.state);
}

//#[test]
pub fn bogus_input() {
    use self::check_result as cr;
    const F: TINFLStatus = TINFLStatus::Failed;
    const OK: TINFLStatus = TINFLStatus::Done;
    // Bad CM.
    cr(&[0x77, 0x85], F, State::BadZlibHeader, true);
    // Bad window size (but check is correct).
    cr(&[0x88, 0x98], F, State::BadZlibHeader, true);
    // Bad check bits.
    cr(&[0x78, 0x98], F, State::BadZlibHeader, true);

    // Too many code lengths. (From inflate library issues)
    cr(
        b"M\xff\xffM*\xad\xad\xad\xad\xad\xad\xad\xcd\xcd\xcdM",
        F,
        State::BadTotalSymbols,
        false,
    );
    // Bad CLEN (also from inflate library issues)
    cr(
        b"\xdd\xff\xff*M\x94ffffffffff",
        F,
        State::BadTotalSymbols,
        false,
    );

    // Port of inflate coverage tests from zlib-ng
    // https://github.com/Dead2/zlib-ng/blob/develop/test/infcover.c
    let c = |a, b, c| cr(a, b, c, false);

    // Invalid uncompressed/raw block length.
    c(&[0, 0, 0, 0, 0], F, State::BadRawLength);
    // Ok empty uncompressed block.
    c(&[3, 0], OK, State::DoneForever);
    // Invalid block type.
    c(&[6], F, State::BlockTypeUnexpected);
    // Ok uncompressed block.
    c(&[1, 1, 0, 0xfe, 0xff, 0], OK, State::DoneForever);
    // Too many litlens, we handle this later than zlib, so this test won't
    // give the same result.
    //        c(&[0xfc, 0, 0], F, State::BadTotalSymbols);
    // Invalid set of code lengths - TODO Check if this is the correct error for this.
    c(&[4, 0, 0xfe, 0xff], F, State::BadTotalSymbols);
    // Invalid repeat in list of code lengths.
    // (Try to repeat a non-existant code.)
    c(&[4, 0, 0x24, 0x49, 0], F, State::BadCodeSizeDistPrevLookup);
    // Missing end of block code (should we have a separate error for this?) - fails on futher input
    //    c(&[4, 0, 0x24, 0xe9, 0xff, 0x6d], F, State::BadTotalSymbols);
    // Invalid set of literals/lengths
    c(
        &[
            4, 0x80, 0x49, 0x92, 0x24, 0x49, 0x92, 0x24, 0x71, 0xff, 0xff, 0x93, 0x11, 0,
        ],
        F,
        State::BadTotalSymbols,
    );
    // Invalid set of distances _ needsmoreinput
    // c(&[4, 0x80, 0x49, 0x92, 0x24, 0x49, 0x92, 0x24, 0x0f, 0xb4, 0xff, 0xff, 0xc3, 0x84], F, State::BadTotalSymbols);
    // Invalid distance code
    c(&[2, 0x7e, 0xff, 0xff], F, State::InvalidDist);

    // Distance refers to position before the start
    c(
        &[0x0c, 0xc0, 0x81, 0, 0, 0, 0, 0, 0x90, 0xff, 0x6b, 0x4, 0],
        F,
        State::DistanceOutOfBounds,
    );

    // Trailer
    // Bad gzip trailer checksum GZip header not handled by miniz_oxide
    //cr(&[0x1f, 0x8b, 0x08 ,0 ,0 ,0 ,0 ,0 ,0 ,0 ,0x03, 0, 0, 0, 0, 0x01], F, State::BadCRC, false)
    // Bad gzip trailer length
    //cr(&[0x1f, 0x8b, 0x08 ,0 ,0 ,0 ,0 ,0 ,0 ,0 ,0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0x01], F, State::BadCRC, false)
}

//#[test]
pub fn empty_output_buffer_non_wrapping() {
    let encoded = [
        120, 156, 243, 72, 205, 201, 201, 215, 81, 168, 202, 201, 76, 82, 4, 0, 27, 101, 4, 19,
    ];
    let flags = TINFL_FLAG_COMPUTE_ADLER32
        | TINFL_FLAG_PARSE_ZLIB_HEADER
        | TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
    let mut r = DecompressorOxide::new();
    let mut output_buf = vec![];
    let mut out_cursor = Cursor::new(output_buf.as_mut_slice());
    // Check that we handle an empty buffer properly and not panicking.
    // https://github.com/Frommi/miniz_oxide/issues/23
    let res = decompress(&mut r, &encoded, &mut out_cursor, flags);
    assert_eq!(res, (TINFLStatus::HasMoreOutput, 4, 0));
}

//#[test]
pub fn empty_output_buffer_wrapping() {
    let encoded = [
        0x73, 0x49, 0x4d, 0xcb, 0x49, 0x2c, 0x49, 0x55, 0x00, 0x11, 0x00,
    ];
    let flags = TINFL_FLAG_COMPUTE_ADLER32;
    let mut r = DecompressorOxide::new();
    let mut output_buf = vec![];
    let mut out_cursor = Cursor::new(output_buf.as_mut_slice());
    // Check that we handle an empty buffer properly and not panicking.
    // https://github.com/Frommi/miniz_oxide/issues/23
    let res = decompress(&mut r, &encoded, &mut out_cursor, flags);
    assert_eq!(res, (TINFLStatus::HasMoreOutput, 2, 0));
}
