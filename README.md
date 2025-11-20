# str-buf

[![Rust](https://github.com/DoumanAsh/str-buf/actions/workflows/rust.yml/badge.svg)](https://github.com/DoumanAsh/str-buf/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/str-buf.svg)](https://crates.io/crates/str-buf)
[![Documentation](https://docs.rs/str-buf/badge.svg)](https://docs.rs/crate/str-buf/)

Static string buffer

## Requirements

- Rust 1.64

## Features:

- `serde` Enables serde serialization. In case of overflow, deserialize fails.
- `ufmt-write` Enables ufmt `uWrite` implementation.
