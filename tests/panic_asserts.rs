use str_buf::StrBuf;

type ZeroStr = StrBuf<0>;
const _: ZeroStr = ZeroStr::from_str("");
type SmolStr = StrBuf<5>;

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
