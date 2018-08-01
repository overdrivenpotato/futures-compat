#![deny(warnings)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! # futures-compat
//!
//! A compatibility layer between different versions of [Future][futures].
//!
//! [futures]: https://crates.io/crates/futures

extern crate futures_v01x;
extern crate futures_v02x;
extern crate tokio_io;

pub mod futures_01;
pub mod futures_02;
