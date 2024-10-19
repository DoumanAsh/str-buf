//!Static string buffer
//!
//!Features:
//!
//!- `serde` Enables serde serialization. In case of overflow, deserialize fails.
//!- `ufmt-write` Enables ufmt `uWrite` implementation.
#![warn(missing_docs)]

#![no_std]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]
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

#[derive(Copy, Clone)]
///Stack based string.
///
///It's size is `mem::size_of::<T>() + mem::size_of::<u8>()`, but remember that it can be padded.
///Can store up to `u8::max_value()` as anything bigger makes it impractical.
///
///Storage is always capped at `u8::max_value()`, which practically means panic during creation,
///until compiler provides a better means to error.
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
        unsafe {
            Self::from_storage([mem::MaybeUninit::uninit(); N], 0)
        }
    }

    #[inline]
    ///Creates new instance from supplied storage and written size.
    ///
    ///It is unsafe, because there is no guarantee that storage is correctly initialized with UTF-8
    ///bytes.
    pub const unsafe fn from_storage(storage: [mem::MaybeUninit<u8>; N], cursor: u8) -> Self {
        debug_assert!(N <= u8::max_value() as usize, "Capacity cannot be more than 255");

        Self {
            inner: storage,
            cursor,
        }
    }

    #[inline]
    ///Creates new instance from existing slice with panic on overflow
    pub const fn from_str(text: &str) -> Self {
        let mut idx = 0;
        let mut storage = [mem::MaybeUninit::<u8>::uninit(); N];

        debug_assert!(text.len() <= storage.len(), "Text cannot fit static storage");
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

    #[inline(always)]
    ///Reads byte at `idx`.
    pub const unsafe fn get_unchecked(&self, idx: usize) -> u8 {
        self.inner[idx].assume_init()
    }

    #[inline]
    ///Reads byte at `idx`.
    pub const fn get(&self, idx: usize) -> Option<u8> {
        if idx < self.cursor as usize {
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
        self.inner.as_ptr() as _
    }

    #[inline]
    ///Returns pointer  to the beginning of underlying buffer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.inner.as_mut_ptr() as *mut u8
    }

    #[inline]
    ///Returns number of bytes left (not written yet)
    pub const fn remaining(&self) -> usize {
        Self::capacity() - self.cursor as usize
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
    #[allow(clippy::missing_transmute_annotations)]
    ///Returns slice to already written data.
    pub const fn as_slice(&self) -> &[u8] {
        //Layout is: (<ptr>, <usize>)
        //
        //Reference:
        //https://github.com/rust-lang/rust/blob/6830052c7b87217886324129bffbe096e485d415/library/core/src/ptr/metadata.rs#L145=
        #[repr(C)]
        struct RawSlice {
            ptr: *const u8,
            size: usize,
        }

        debug_assert!(unsafe {
            mem::transmute::<_, RawSlice>([3, 2, 1].as_slice()).size
        } == 3, "RawSlice layout has been changed in compiler unexpectedly");

        unsafe {
            mem::transmute(RawSlice {
                ptr: self.as_ptr(),
                size: self.len(),
            })
        }
    }

    #[inline]
    ///Returns mutable slice to already written data.
    ///
    ///To safely modify the slice, user must guarantee to write valid UTF-8
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.as_mut_ptr(), self.cursor as usize)
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
            self.set_len(0);
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
        if N > u8::max_value() as usize {
            u8::max_value() as usize
        } else {
            N
        }
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
        ptr::copy_nonoverlapping(text.as_ptr(), self.as_mut_ptr().offset(self.cursor as isize), text.len());
        self.set_len(self.cursor.saturating_add(text.len() as u8));
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
        while idx < bytes.len() {
            let cursor = self.cursor as usize + idx;
            self.inner[cursor] = mem::MaybeUninit::new(bytes[idx]);
            idx += 1;
        }
        self.cursor = self.cursor.saturating_add(bytes.len() as u8);

        self
    }

    #[inline(always)]
    ///Access str from underlying storage
    ///
    ///Returns empty if nothing has been written into buffer yet.
    pub const fn as_str(&self) -> &str {
        //You think I care?
        //Make `from_utf8_unchecked` const fn first
        unsafe {
            core::str::from_utf8_unchecked(self.as_slice())
        }
    }

    #[inline(always)]
    ///Modifies this string to convert all its characters into ASCII lower case equivalent
    pub const fn into_ascii_lowercase(mut self) -> Self {
        let len = self.len();
        let mut idx = 0;
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
        let len = self.len();
        let mut idx = 0;
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
