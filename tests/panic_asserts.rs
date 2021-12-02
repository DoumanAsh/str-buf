use str_buf::StrBuf;

type ZeroStr = StrBuf<0>;
const _: ZeroStr = ZeroStr::from_str("");
type SmolStr = StrBuf<5>;
type MaxCapStr = StrBuf<{u8::max_value() as usize}>;

#[test]
#[should_panic]
fn overflow_on_invalid_capacity() {
    let _ = StrBuf::<500>::new();
}

#[test]
#[should_panic]
fn from_str_overflow_panic() {
    let _ = ZeroStr::from_str("lolka");
}

#[test]
#[should_panic]
fn and_overflow_panic() {
    let _ = SmolStr::from_str("lolka").and("extra");
}

#[test]
#[should_panic]
fn and_overflow_panic_over_max_capacity() {
    let _ = MaxCapStr::from_str("lolka").and("extra")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890")
                                        .and("1234567890123456789012345678901234567890123456789012345678901234567890");

}
