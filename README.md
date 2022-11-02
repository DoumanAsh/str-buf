# str-buf

![Rust](https://github.com/DoumanAsh/str-buf/workflows/Rust/badge.svg?branch=master)
[![Crates.io](https://img.shields.io/crates/v/str-buf.svg)](https://crates.io/crates/str-buf)
[![Documentation](https://docs.rs/str-buf/badge.svg)](https://docs.rs/crate/str-buf/)

Static string buffer

## Requirements

- Rust 1.59

## Features:

- `serde` Enables serde serialization. In case of overflow, deserialize fails.
- `ufmt-write` Enables ufmt `uWrite` implementation.
