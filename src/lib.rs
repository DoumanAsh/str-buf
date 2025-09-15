//!Static string buffer
//!
//!Features:
//!
//!- `serde` Enables serde serialization. In case of overflow, deserialize fails.
//!- `ufmt-write` Enables ufmt `uWrite` implementation.
#![warn(missing_docs)]

#![no_std]
#![allow(clippy::style)]
#![cfg_attr(rustfmt, rustfmt_skip)]

use core::{mem, slice, ptr, cmp, ops, hash, fmt, borrow};

#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "ufmt-write")]
mod ufmt;

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

///Max capacity to use length of size 1 byte
pub const CAPACITY_U8: usize = 256;
///Max capacity to use length of size 2 byte
pub const CAPACITY_U16: usize = 65537;

///Calculates necessary buffer capacity to fit provided number of bytes
pub const fn capacity(desired: usize) -> usize {
    if desired == 0 {
        desired
    } else if desired <= u8::MAX as usize {
        desired.saturating_add(1)
    } else if desired <= u16::MAX as usize {
        desired.saturating_add(2)
    } else {
        desired.saturating_add(mem::size_of::<usize>())
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
///Stack based string.
///
///It's size is `mem::size_of::<[u8; T]>()` including bytes for length of the string
///
///Depending on size of string, it uses different length size:
///- `0` - Uses 0 bytes to store length
///- `1..=256` - Uses 1 byte to store length
///- `257..=65537` - Uses 2 bytes to store length
///- `65537..` - Uses `mem::size_of::<usize>()` bytes to store length
///
///In case of capacity overflow there is no re-adjustment possible
///Therefore When attempting to create new instance from `&str` it panics on overflow.
///
///```
///use str_buf::StrBuf;
///use core::mem;
///use core::fmt::Write;
///use core::convert::TryInto;
///
///type MyStr = StrBuf::<{str_buf::capacity(mem::size_of::<String>())}>;
///
///const CONST_STR: MyStr = MyStr::new().and("hello").and(" ").and("world");
///
///assert_eq!(CONST_STR.len(), "hello world".len(), "Length should be 11 characters");
///assert_eq!(CONST_STR, "hello world");
///
///assert_eq!(MyStr::capacity(), mem::size_of::<String>(), "Should be equal to size of String");
///
///let text: MyStr = "test".try_into().expect("To fit string");
///assert_eq!("test", text);
///assert_eq!(text, "test");
///let mut text = MyStr::new();
///let _ = write!(text, "test {}", "hello world");
///assert_eq!(text.as_str(), "test hello world");
///assert_eq!(text.remaining(), MyStr::capacity() - "test hello world".len(), "Should modify length");
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
}

impl<const N: usize> StrBuf<N> {
    ///Length of bytes used to store buffer's length
    pub const LEN_OFFSET: usize = if N == 0 {
        0
    } else if N <= CAPACITY_U8 {
        1
    } else if N <= CAPACITY_U16 {
        2
    } else {
        mem::size_of::<usize>()
    };

    const CAPACITY: usize = N - Self::LEN_OFFSET;

    #[inline]
    ///Creates new instance
    pub const fn new() -> Self {
        unsafe {
            Self::from_storage([mem::MaybeUninit::uninit(); N]).const_set_len(0)
        }
    }

    #[inline]
    ///Creates new instance from supplied storage and written size.
    ///
    ///It is unsafe, because there is no guarantee that storage is correctly initialized with UTF-8
    ///bytes.
    ///
    ///First `Self::LEN_OFFSET` must be initialized with its length
    pub const unsafe fn from_storage(storage: [mem::MaybeUninit<u8>; N]) -> Self {
        debug_assert!(N <= usize::max_value(), "Capacity cannot exceed usize");

        Self {
            inner: storage,
        }
    }

    #[inline]
    ///Creates new instance from existing slice with panic on overflow
    pub const fn from_str(text: &str) -> Self {
        let mut idx = 0;
        let mut storage = [mem::MaybeUninit::<u8>::uninit(); N];

        debug_assert!(text.len() <= Self::CAPACITY, "Text cannot fit static storage");
        while idx < text.len() {
            storage[Self::LEN_OFFSET + idx] = mem::MaybeUninit::new(text.as_bytes()[idx]);
            idx += 1;
        }

        unsafe {
            Self::from_storage(storage).const_set_len(text.len())
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

    #[inline(always)]
    ///Reads byte at `idx`.
    pub const unsafe fn get_unchecked(&self, idx: usize) -> u8 {
        self.inner[Self::LEN_OFFSET + idx].assume_init()
    }

    #[inline]
    ///Reads byte at `idx`.
    pub const fn get(&self, idx: usize) -> Option<u8> {
        if idx < self.len() {
            unsafe {
                Some(self.get_unchecked(idx))
            }
        } else {
            None
        }
    }

    #[inline]
    ///Returns pointer  to the beginning of underlying buffer
    pub const fn as_ptr(&self) -> *const u8 {
        unsafe {
            self.inner.as_ptr().add(Self::LEN_OFFSET) as _
        }
    }

    #[inline]
    ///Returns pointer  to the beginning of underlying buffer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        unsafe {
            self.inner.as_mut_ptr().add(Self::LEN_OFFSET) as *mut u8
        }
    }

    #[inline]
    ///Returns number of bytes left (not written yet)
    pub const fn remaining(&self) -> usize {
        Self::capacity() - self.len()
    }

    #[inline]
    ///Returns reference to underlying storage as it is.
    pub const fn as_storage(&self) -> &[mem::MaybeUninit<u8>; N] {
        &self.inner
    }

    #[inline]
    ///Returns reference to underlying storage as it is.
    ///
    ///To safely modify the storage, user must guarantee to write valid UTF-8
    pub unsafe fn as_mut_storage(&mut self) -> &mut [mem::MaybeUninit<u8>; N] {
        &mut self.inner
    }

    #[inline]
    ///Returns slice to already written data.
    pub const fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.as_ptr(), self.len())
        }
    }

    #[inline]
    ///Returns mutable slice to already written data.
    ///
    ///To safely modify the slice, user must guarantee to write valid UTF-8
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.as_mut_ptr(), self.len())
    }

    #[inline]
    ///Returns mutable slice with unwritten parts of the buffer.
    pub fn as_write_slice(&mut self) -> &mut [mem::MaybeUninit<u8>] {
        let len = self.len();
        &mut self.inner[Self::LEN_OFFSET + len..]
    }

    #[inline(always)]
    ///Clears the content of buffer.
    pub fn clear(&mut self) {
        unsafe {
            self.set_len(0);
        }
    }

    #[inline(always)]
    ///Returns empty self.
    pub const fn empty(self) -> Self {
        unsafe {
            self.const_set_len(0)
        }
    }

    #[inline]
    ///Shortens the buffer, keeping the first `cursor` elements.
    ///
    ///Does nothing if new `cursor` is after current position.
    ///
    ///Unsafe as it is up to user to consider character boundary
    pub unsafe fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.set_len(len);
        }
    }

    #[inline]
    ///Returns buffer overall capacity.
    pub const fn capacity() -> usize {
        Self::CAPACITY
    }

    #[inline]
    ///Returns number of bytes written.
    pub const fn len(&self) -> usize {
        if N == 0 {
            0
        } else if N <= CAPACITY_U8 {
            unsafe {
                self.inner[0].assume_init() as _
            }
        } else if N <= CAPACITY_U16 {
            unsafe {
                u16::from_ne_bytes(*(self.inner.as_ptr() as *const [u8; mem::size_of::<u16>()])) as usize
            }
        } else {
            unsafe {
                usize::from_ne_bytes(*(self.inner.as_ptr() as *const [u8; mem::size_of::<usize>()]))
            }
        }
    }

    #[inline(always)]
    ///Sets new length of the string.
    const unsafe fn const_set_len(mut self, len: usize) -> Self {
        if N == 0 {
            //no length
        } else if N <= CAPACITY_U8 {
            self.inner[0] = mem::MaybeUninit::new(len as _);
        } else if N <= CAPACITY_U16 {
            let len = (len as u16).to_ne_bytes();
            self.inner[0] = mem::MaybeUninit::new(len[0]);
            self.inner[1] = mem::MaybeUninit::new(len[1]);
        } else {
            let len = len.to_ne_bytes();
            let mut idx = 0;
            loop {
                self.inner[idx] = mem::MaybeUninit::new(len[idx]);
                idx = idx.saturating_add(1);
                if idx >= len.len() {
                    break;
                }
            }
        }

        self
    }

    #[inline(always)]
    ///Sets new length of the string.
    pub unsafe fn set_len(&mut self, len: usize) {
        if N == 0 {
            //No length
        } else if N <= CAPACITY_U8 {
            self.inner[0] = mem::MaybeUninit::new(len as _);
        } else if N <= CAPACITY_U16 {
            let len = (len as u16).to_ne_bytes();
            self.inner[0] = mem::MaybeUninit::new(len[0]);
            self.inner[1] = mem::MaybeUninit::new(len[1]);
        } else {
            let len = len.to_ne_bytes();
            let ptr = self.inner.as_mut_ptr();
            ptr::copy_nonoverlapping(len.as_ptr(), ptr as *mut _, mem::size_of::<usize>());
        }
    }

    #[inline]
    ///Appends given string without any size checks
    pub unsafe fn push_str_unchecked(&mut self, text: &str) {
        ptr::copy_nonoverlapping(text.as_ptr(), self.as_mut_ptr().add(self.len()), text.len());
        self.set_len(self.len().saturating_add(text.len()));
    }

    #[inline]
    ///Appends given string, truncating on overflow, returning number of written bytes
    pub fn push_str(&mut self, text: &str) -> usize {
        let mut size = cmp::min(text.len(), self.remaining());

        #[cold]
        fn shift_by_char_boundary(text: &str, mut size: usize) -> usize {
            while !text.is_char_boundary(size) {
                size -= 1;
            }
            size
        }

        if !text.is_char_boundary(size) {
            //0 is always char boundary so 0 - 1 is impossible
            size = shift_by_char_boundary(text, size - 1);
        }

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
        debug_assert!(self.remaining() >= bytes.len(), "Buffer overflow");

        let mut idx = 0;
        let cursor = self.len();
        while idx < bytes.len() {
            self.inner[Self::LEN_OFFSET + cursor + idx] = mem::MaybeUninit::new(bytes[idx]);
            idx += 1;
        }
        self.const_set_len(cursor + bytes.len())
    }

    #[inline(always)]
    ///Access str from underlying storage
    ///
    ///Returns empty if nothing has been written into buffer yet.
    pub const fn as_str(&self) -> &str {
        unsafe {
            core::str::from_utf8_unchecked(self.as_slice())
        }
    }

    #[inline(always)]
    ///Modifies this string to convert all its characters into ASCII lower case equivalent
    pub const fn into_ascii_lowercase(mut self) -> Self {
        let len = Self::LEN_OFFSET + self.len();
        let mut idx = Self::LEN_OFFSET;
        loop {
            if idx >= len {
                break;
            }

            self.inner[idx] = unsafe {
                mem::MaybeUninit::new(self.inner[idx].assume_init().to_ascii_lowercase())
            };
            idx = idx.saturating_add(1);
        }
        self
    }

    #[inline(always)]
    ///Converts this string to its ASCII lower case equivalent in-place.
    pub fn make_ascii_lowercase(&mut self) {
        unsafe {
            self.as_mut_slice().make_ascii_lowercase()
        }
    }

    #[inline(always)]
    ///Modifies this string to convert all its characters into ASCII upper case equivalent
    pub const fn into_ascii_uppercase(mut self) -> Self {
        let len = Self::LEN_OFFSET + self.len();
        let mut idx = Self::LEN_OFFSET;
        loop {
            if idx >= len {
                break;
            }

            self.inner[idx] = unsafe {
                mem::MaybeUninit::new(self.inner[idx].assume_init().to_ascii_uppercase())
            };
            idx = idx.saturating_add(1);
        }
        self
    }

    #[inline(always)]
    ///Converts this string to its ASCII upper case equivalent in-place.
    pub fn make_ascii_uppercase(&mut self) {
        unsafe {
            self.as_mut_slice().make_ascii_uppercase()
        }
    }

    ///Trims of whitespaces around both ends of the string in place
    pub fn make_trim(&mut self) {
        let this = self.as_str();
        let len = this.len();
        let mut trim_left_count = 0usize;
        let mut trim_right_count = 0usize;
        let mut chars = this.chars();
        while let Some(ch) = chars.next() {
            if ch.is_whitespace() {
                trim_left_count = trim_left_count.saturating_add(ch.len_utf8());
            } else {
                break;
            }
        }

        for ch in chars.rev() {
            if ch.is_whitespace() {
                trim_right_count = trim_right_count.saturating_add(ch.len_utf8());
            } else {
                break;
            }
        }

        let new_len = len.saturating_sub(trim_left_count).saturating_sub(trim_right_count);
        if new_len != len {
            unsafe {
                //To make sure Miri doesn't complain, you have to derive both pointers from the same one otherwise Miri will 're-borrow' for no reason
                let dest = self.as_mut_ptr();
                let src = dest.add(trim_left_count) as *const _;
                ptr::copy(src, dest, new_len);
                self.set_len(new_len);
            }
        }
    }

    #[inline]
    ///Trims of whitespaces on the left in place.
    pub fn make_trim_left(&mut self) {
        let this = self.as_str();
        let len = this.len();
        let mut trim_count = 0usize;
        for ch in this.chars() {
            if ch.is_whitespace() {
                trim_count = trim_count.saturating_add(ch.len_utf8());
            } else {
                break;
            }
        }

        let new_len = len.saturating_sub(trim_count);
        unsafe {
            let dest = self.as_mut_ptr();
            let src = dest.add(trim_count);
            ptr::copy(src, dest, new_len);
            self.set_len(new_len);
        }
    }

    #[inline]
    ///Trims of whitespaces on the right in place.
    pub fn make_trim_right(&mut self) {
        let this = self.as_str();
        let len = this.len();
        let mut trim_count = 0usize;
        for ch in this.chars().rev() {
            if ch.is_whitespace() {
                trim_count = trim_count.saturating_add(ch.len_utf8());
            } else {
                break;
            }
        }

        unsafe {
            self.set_len(len.saturating_sub(trim_count))
        }
    }

    #[inline]
    ///Removes last character from the buffer, if any present
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().rev().next()?;
        let new_len = self.len() - ch.len_utf8();
        unsafe {
            self.set_len(new_len)
        }
        Some(ch)
    }
}

impl<const S: usize> AsRef<str> for StrBuf<S> {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const S: usize> fmt::Write for StrBuf<S> {
    #[inline(always)]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.push_str(s) == s.len() {
            Ok(())
        } else {
            Err(fmt::Error)
        }
    }
}

impl<const S: usize> fmt::Display for StrBuf<S> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), fmt)
    }
}

impl<const S: usize> fmt::Debug for StrBuf<S> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), fmt)
    }
}

impl<const S: usize> AsRef<[u8]> for StrBuf<S> {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
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

impl<const S: usize> core::str::FromStr for StrBuf<S> {
    type Err = StrBufError;

    #[inline(always)]
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::from_str_checked(text)
    }
}
