use str_buf::StrBuf;

use core::fmt;

type SmolStr = StrBuf<6>;
type MediumStr = StrBuf<290>;
type BigStr = StrBuf<67_000>;

#[test]
fn should_return_error_on_fmt_write_overflow() {
    let mut buf = SmolStr::new();

    fmt::Write::write_str(&mut buf, "rorリ").expect_err("Should error on overflow");
    assert_eq!(buf, "ror");
    assert_eq!(buf.len(), 3);

    buf.clear();
    buf.clear();

    assert!(buf.pop().is_none());

    fmt::Write::write_str(&mut buf, "ロri").expect("Should write fully");
    assert_eq!(buf, "ロri");
    assert_eq!(buf.len(), 5);

    assert_eq!(buf.pop(), Some('i'));
    assert_eq!(buf.pop(), Some('r'));
    assert_eq!(buf.pop(), Some('ロ'));
    assert!(buf.pop().is_none());
}

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

    assert_eq!(buf.push_str("rorリ"), 3);
    assert_eq!(buf, "ror");
    assert_eq!(buf.len(), "ror".len());

    unsafe {
        buf.set_len(0);
    }

    assert_eq!(buf.push_str("ロリ"), 3);
    assert_eq!(buf, "ロ");

    assert_eq!(buf.pop(), Some('ロ'));

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

#[test]
fn should_correctly_convert_ascii_case_medium() {
    let mut buf = MediumStr::new();
    assert_eq!(buf.push_str("ロri"), "ロri".len());
    assert_eq!(buf.len(), "ロri".len());

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
fn should_correctly_truncate_by_char_boundary_medium() {
    const PADDING: usize = MediumStr::capacity() - SmolStr::capacity();
    let mut buf = MediumStr::new();
    for idx in 0..PADDING {
        assert_eq!(buf.len(), idx);
        buf.push_str("-");
    }
    assert_eq!(buf.len(), PADDING);

    assert_eq!(buf.push_str("rorリ"), 3);
    assert_eq!(&buf[PADDING..], "ror");
    assert_eq!(buf.len(), PADDING + "ror".len());

    unsafe {
        buf.set_len(PADDING);
    }

    assert_eq!(buf.push_str("ロリ"), 3);
    assert_eq!(&buf[PADDING..], "ロ");

    assert_eq!(buf.pop(), Some('ロ'));

    assert_eq!(buf.push_str("ロリ"), 3);
    assert_eq!(&buf[PADDING..], "ロ");
    assert_eq!(buf.len(), PADDING + "ロ".len());

    assert_eq!(buf.push_str("リ"), 0);

    assert_eq!(buf.push_str("r"), 1);
    assert_eq!(buf.len(), PADDING + "ロ".len() + 1);
    assert_eq!(&buf[PADDING..], "ロr");

    assert_eq!(buf.push_str("i"), 1);
    assert_eq!(buf.len(), PADDING + "ロ".len() + 2);
    assert_eq!(&buf[PADDING..], "ロri");
    assert_eq!(buf.push_str("."), 0);

    let copy = buf;
    assert_eq!(copy, buf);
}

#[test]
fn should_correctly_convert_ascii_case_big() {
    let mut buf = BigStr::new();
    assert_eq!(buf.push_str("ロri"), "ロri".len());
    assert_eq!(buf.len(), "ロri".len());

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
fn should_correctly_truncate_by_char_boundary_big() {
    const PADDING: usize = BigStr::capacity() - SmolStr::capacity();
    let mut buf = BigStr::new();
    for idx in 0..PADDING {
        assert_eq!(buf.len(), idx);
        buf.push_str("-");
    }
    assert_eq!(buf.len(), PADDING);

    assert_eq!(buf.push_str("rorリ"), 3);
    assert_eq!(&buf[PADDING..], "ror");
    assert_eq!(buf.len(), PADDING + "ror".len());

    unsafe {
        buf.set_len(PADDING);
    }

    assert_eq!(buf.push_str("ロリ"), 3);
    assert_eq!(&buf[PADDING..], "ロ");
    assert_eq!(buf.len(), PADDING + "ロ".len());

    assert_eq!(buf.pop(), Some('ロ'));

    assert_eq!(buf.push_str("ロリ"), 3);
    assert_eq!(&buf[PADDING..], "ロ");
    assert_eq!(buf.len(), PADDING + "ロ".len());

    assert_eq!(buf.push_str("リ"), 0);

    assert_eq!(buf.push_str("r"), 1);
    assert_eq!(buf.len(), PADDING + "ロ".len() + 1);
    assert_eq!(&buf[PADDING..], "ロr");

    assert_eq!(buf.push_str("i"), 1);
    assert_eq!(buf.len(), PADDING + "ロ".len() + 2);
    assert_eq!(&buf[PADDING..], "ロri");
    assert_eq!(buf.push_str("."), 0);

    let copy = buf;
    assert_eq!(copy, buf);
}
