//!Static string buffer
//!
//!Features:
//!
//!- `serde` Enables serde serialization. In case of overflow, deserialize fails.
#![warn(missing_docs)]

#![no_std]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

use core::{mem, slice, ptr, cmp, ops, hash, fmt, borrow};

#[cfg(feature = "serde")]
mod serde;

#[derive(Debug, Clone)]
///`StrBuf` conversion error
pub enum StrBufError {
    ///Not enough space for string to be converted into `StrBuf`.
    Overflow,
}

impl fmt::Display for StrBufError {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrBufError::Overflow => fmt.write_str("Buffer overflow"),
        }
    }
}

///Stack based string.
///
///It's size is `mem::size_of::<T>() + mem::size_of::<u8>()`, but remember that it can be padded.
///Can store up to `u8::max_value()` as anything bigger makes a little sense.
///
///Storage `T` is always interpreted as array of bytes.
///
///When attempting to create new instance from `&str` it panics on overflow in debug mode.
///
///```
///use str_buf::StrBuf;
///use core::mem;
///use core::fmt::Write;
///use core::convert::TryInto;
///
///type MyStr = StrBuf::<{mem::size_of::<String>()}>;
///
///const CONST_STR: MyStr = MyStr::new().and("hello").and(" ").and("world");
///
///assert_eq!(CONST_STR, "hello world");
///
///assert_eq!(MyStr::capacity(), mem::size_of::<String>());
/////If you want it to be equal to string you'll have to adjust storage accordingly
///assert_ne!(mem::size_of::<MyStr>(), mem::size_of::<String>());
///assert_eq!(mem::size_of::<StrBuf::<{mem::size_of::<String>() - 1}>>(), mem::size_of::<String>());
///
///let text: MyStr = "test".try_into().expect("To fit string");
///assert_eq!("test", text);
///assert_eq!(text, "test");
///let mut text = MyStr::new();
///let _ = write!(text, "test {}", "hello world");
///assert_eq!(text.as_str(), "test hello world");
///assert_eq!(text.remaining(), MyStr::capacity() - "test hello world".len());
///
///assert_eq!(text.push_str(" or maybe not"), 8); //Overflow!
///assert_eq!(text.as_str(), "test hello world or mayb");
///assert_eq!(text.push_str(" or maybe not"), 0); //Overflow, damn
///
///text.clear();
///assert_eq!(text.push_str(" or maybe not"), 13); //noice
///assert_eq!(text.as_str(), " or maybe not");
///
///assert_eq!(text.clone().as_str(), text.as_str());
///assert_eq!(text.clone(), text);
///```
pub struct StrBuf<const N: usize> {
    inner: [mem::MaybeUninit<u8>; N],
    cursor: u8, //number of bytes written
}

impl<const N: usize> StrBuf<N> {
    #[inline]
    ///Creates new instance
    pub const fn new() -> Self {
        Self {
            inner: [mem::MaybeUninit::uninit(); N],
            cursor: 0,
        }
    }

    #[inline]
    ///Creates new instance from supplied storage and written size.
    ///
    ///It is unsafe, because there is no guarantee that storage is correctly initialized with UTF-8
    ///bytes.
    pub const unsafe fn from_storage(storage: [mem::MaybeUninit<u8>; N], cursor: u8) -> Self {
        Self {
            inner: storage,
            cursor,
        }
    }

    #[inline]
    ///Creates new instance from existing slice with panic on overflow
    pub const fn from_str(text: &str) -> Self {
        let mut storage = [mem::MaybeUninit::<u8>::uninit(); N];

        let mut idx = 0;
        while idx < text.len() {
            storage[idx] = mem::MaybeUninit::new(text.as_bytes()[idx]);
            idx += 1;
        }

        unsafe {
            Self::from_storage(storage, idx as u8)
        }
    }

    #[inline]
    ///Creates new instance from existing slice which returns error on overflow
    pub const fn from_str_checked(text: &str) -> Result<Self, StrBufError> {
        if text.len() <= Self::capacity() {
            Ok(Self::from_str(text))
        } else {
            Err(StrBufError::Overflow)
        }
    }

    #[inline]
    ///Returns pointer  to the beginning of underlying buffer
    pub const fn as_ptr(&self) -> *const u8 {
        &self.inner as *const _ as *const u8
    }

    #[inline]
    ///Returns number of bytes left (not written yet)
    pub const fn remaining(&self) -> usize {
        Self::capacity() - self.cursor as usize
    }

    #[inline]
    ///Returns slice to already written data.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.as_ptr(), self.cursor as usize)
        }
    }

    #[inline]
    ///Returns mutable slice to already written data.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.as_ptr() as *mut u8, self.cursor as usize)
        }
    }

    #[inline]
    ///Returns mutable slice with unwritten parts of the buffer.
    pub fn as_write_slice(&mut self) -> &mut [mem::MaybeUninit<u8>] {
        &mut self.inner[self.cursor as usize..]
    }

    #[inline(always)]
    ///Clears the content of buffer.
    pub fn clear(&mut self) {
        unsafe {
            self.truncate(0);
        }
    }

    #[inline(always)]
    ///Returns empty self.
    pub const fn empty(mut self) -> Self {
        self.cursor = 0;
        self
    }

    #[inline]
    ///Shortens the buffer, keeping the first `cursor` elements.
    ///
    ///Does nothing if new `cursor` is after current position.
    ///
    ///Unsafe as it is up to user to consider character boundary
    pub unsafe fn truncate(&mut self, cursor: u8) {
        if cursor < self.cursor {
            self.set_len(cursor);
        }
    }

    #[inline]
    ///Returns buffer overall capacity.
    pub const fn capacity() -> usize {
        N
    }

    #[inline]
    ///Returns number of bytes written.
    pub const fn len(&self) -> usize {
        self.cursor as usize
    }

    #[inline(always)]
    ///Sets new length of the string.
    pub unsafe fn set_len(&mut self, len: u8) {
        self.cursor = len
    }

    #[inline]
    ///Appends given string without any size checks
    pub unsafe fn push_str_unchecked(&mut self, text: &str) {
        ptr::copy_nonoverlapping(text.as_ptr(), self.as_ptr().offset(self.cursor as isize) as *mut u8, text.len());
        self.set_len(self.cursor.saturating_add(text.len() as u8));
    }

    #[inline]
    ///Appends given string, truncating on overflow, returning number of written bytes
    pub fn push_str(&mut self, text: &str) -> usize {
        let size = cmp::min(text.len(), self.remaining());
        unsafe {
            self.push_str_unchecked(&text[..size]);
        }
        size
    }

    #[inline]
    ///Appends given string, assuming it fits.
    ///
    ///On overflow panics with index out of bounds.
    pub const fn and(self, text: &str) -> Self {
        unsafe {
            self.and_unsafe(text.as_bytes())
        }
    }

   #[inline]
    ///Unsafely appends given bytes, assuming valid utf-8.
    ///
    ///On overflow panics with index out of bounds as `and`.
    pub const unsafe fn and_unsafe(mut self, bytes: &[u8]) -> Self {
        let mut idx = 0;
        while idx < bytes.len() {
            self.inner[self.cursor as usize] = mem::MaybeUninit::new(bytes[idx]);
            self.cursor += 1;
            idx += 1;
        }

        self
    }

    #[inline(always)]
    ///Access str from underlying storage
    ///
    ///Returns empty if nothing has been written into buffer yet.
    pub fn as_str(&self) -> &str {
        unsafe {
            let slice = core::slice::from_raw_parts(self.as_ptr(), self.len());
            core::str::from_utf8_unchecked(slice)
        }
    }
}

impl<const S: usize> AsRef<str> for StrBuf<S> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const S: usize> core::fmt::Write for StrBuf<S> {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.push_str(s) == s.len() {
            Ok(())
        } else {
            Err(core::fmt::Error)
        }
    }
}

impl<const S: usize> core::fmt::Display for StrBuf<S> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const S: usize> core::fmt::Debug for StrBuf<S> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const S: usize> Clone for StrBuf<S> {
    #[inline]
    fn clone(&self) -> Self {
        let mut result = Self::new();
        unsafe {
            result.push_str_unchecked(self.as_str())
        }
        result
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clear();
        unsafe {
            self.push_str_unchecked(source.as_str());
        }
    }
}

impl<const S: usize> AsRef<[u8]> for StrBuf<S> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<const S: usize> AsMut<[u8]> for StrBuf<S> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl<const S: usize> borrow::Borrow<str> for StrBuf<S> {
    #[inline(always)]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const S: usize> ops::Deref for StrBuf<S> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const S: usize> Eq for StrBuf<S> {}

impl<const S: usize> PartialEq<StrBuf<S>> for StrBuf<S> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const S: usize> PartialEq<StrBuf<S>> for &str {
    #[inline(always)]
    fn eq(&self, other: &StrBuf<S>) -> bool {
        *self == other.as_str()
    }
}


impl<const S: usize> PartialEq<StrBuf<S>> for str {
    #[inline(always)]
    fn eq(&self, other: &StrBuf<S>) -> bool {
        self == other.as_str()
    }
}

impl<const S: usize> PartialEq<str> for StrBuf<S> {
    #[inline(always)]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const S: usize> PartialEq<&str> for StrBuf<S> {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const S: usize> cmp::Ord for StrBuf<S> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<const S: usize> PartialOrd for StrBuf<S> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const S: usize> hash::Hash for StrBuf<S> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher)
    }
}

impl<const S: usize> core::convert::TryFrom<&str> for StrBuf<S> {
    type Error = StrBufError;

    #[inline(always)]
    fn try_from(text: &str) -> Result<Self, Self::Error> {
        Self::from_str_checked(text)
    }
}
