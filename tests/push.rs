use str_buf::StrBuf;

type SmolStr = StrBuf<5>;

#[test]
fn should_correctly_convert_ascii_case() {
    let mut buf = SmolStr::new();
    assert_eq!(buf.push_str("ロri"), "ロri".len());

    let buf_copy = buf.clone().into_ascii_uppercase();
    buf.make_ascii_uppercase();
    assert_eq!(buf, "ロRI");
    assert_eq!(buf_copy, "ロRI");

    let buf_copy = buf.clone().into_ascii_lowercase();
    buf.make_ascii_lowercase();
    assert_eq!(buf, "ロri");
    assert_eq!(buf_copy, "ロri");
}

#[test]
fn should_correctly_truncate_by_char_boundary() {
    let mut buf = SmolStr::new();
    assert_eq!(buf.push_str("ロリ"), 3);
    assert_eq!(buf, "ロ");
    assert_eq!(buf.push_str("リ"), 0);
    assert_eq!(buf.push_str("r"), 1);
    assert_eq!(buf, "ロr");
    assert_eq!(buf.push_str("i"), 1);
    assert_eq!(buf, "ロri");
    assert_eq!(buf.push_str("."), 0);

    let copy = buf;
    assert_eq!(copy, buf);
}
