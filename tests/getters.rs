
type SmolStr = str_buf::StrBuf<5>;

#[test]
fn should_get_byte() {
    const TEST: SmolStr = SmolStr::from_str("123");
    const BYTE1: Option<u8> = TEST.get(0);
    const BYTE2: Option<u8> = TEST.get(1);
    const BYTE3: Option<u8> = TEST.get(2);
    const OUT_OF_BOUNDS_BYTE: Option<u8> = TEST.get(TEST.len());

    assert_eq!(BYTE1, Some(b'1'));
    assert_eq!(BYTE2, Some(b'2'));
    assert_eq!(BYTE3, Some(b'3'));
    assert_eq!(OUT_OF_BOUNDS_BYTE, None);
}
