use str_buf::StrBuf;

type ZeroStr = StrBuf<0>;
const _: ZeroStr = ZeroStr::from_str("");

#[test]
#[should_panic]
fn from_str_overflow_panic() {
    let _ = ZeroStr::from_str("lolka");
}
