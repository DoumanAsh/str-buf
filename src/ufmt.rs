use crate::{StrBuf, StrBufError};
use ufmt_write::uWrite;

impl<const S: usize> uWrite for StrBuf<S> {
    type Error = StrBufError;

    fn write_str(&mut self, text: &str) -> Result<(), Self::Error> {
        if self.remaining() < text.len() {
            Err(Self::Error::Overflow)
        } else {
            unsafe {
                self.push_str_unchecked(text);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_write_within_capacity() {
        let mut text = StrBuf::<11>::new();
        uWrite::write_str(&mut text, "123456789").expect("Success");
        assert_eq!(text.len(), 9);
        uWrite::write_str(&mut text, "1").expect("Success");
        assert_eq!(text.len(), 10);
        assert_eq!(text.as_str(), "1234567891");
        assert!(uWrite::write_str(&mut text, "1").is_err());
    }
}
