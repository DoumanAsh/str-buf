use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::StrBuf;

impl<S: Sized> Serialize for StrBuf<S> {
    #[inline]
    fn serialize<SER: Serializer>(&self, ser: SER) -> Result<SER::Ok, SER::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'a, S: Sized> Deserialize<'a> for StrBuf<S> {
    fn deserialize<D: Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
        let text: &'a str = Deserialize::deserialize(des)?;

        if text.len() <= Self::capacity() {
            let mut result = Self::new();
            unsafe {
                result.push_str_unchecked(text);
            }
            Ok(result)
        } else {
            Err(serde::de::Error::custom(format_args!("Exceeds buffer capacity({} bytes)", Self::capacity())))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::StrBuf;

    use serde::de::Deserialize;
    use serde::de::value::{BorrowedStrDeserializer, Error as ValueError};

    #[test]
    fn should_error_one_exceeding_capacity() {
        let des = BorrowedStrDeserializer::<ValueError>::new("lolka");
        let res = StrBuf::<[u8;4]>::deserialize(des);
        assert!(res.is_err());
    }

    #[test]
    fn should_ok_within_capacity() {
        let des = BorrowedStrDeserializer::<ValueError>::new("lolka");
        let res = StrBuf::<[u8;6]>::deserialize(des).expect("Unexpected fail");
        assert_eq!(res.as_str(), "lolka");
    }
}
