use str_buf::StrBuf;

type ZeroStr = StrBuf<0>;
const _: ZeroStr = ZeroStr::from_str("");
type SmolStr = StrBuf<5>;
type MediumStr = StrBuf<290>;
type BigStr = StrBuf<67_000>;

#[test]
#[should_panic]
fn from_str_overflow_panic() {
    let _ = ZeroStr::from_str("lolka");
}

#[test]
#[should_panic]
fn from_str_overflow_panic_smol() {
    let _ = SmolStr::from_str("lolkaasdasd");
}

#[test]
#[should_panic]
fn and_overflow_panic() {
    let _ = SmolStr::from_str("1234").and("extra");
}

#[test]
fn and_not_overflow_after_set_len() {
    let smol = unsafe {
        SmolStr::from_str("1234").const_set_len(2).and("34")
    };
    assert_eq!(smol, "1234")
}

#[cfg_attr(miri, ignore)]
#[test]
#[should_panic]
fn and_overflow_panic_medium() {
    let mut buffer = MediumStr::new();
    loop {
        buffer = buffer.and("extra");
    }
}

#[cfg_attr(miri, ignore)]
#[test]
#[should_panic]
fn and_overflow_panic_big() {
    let mut buffer = BigStr::new();
    loop {
        buffer = buffer.and("extra");
    }
}
