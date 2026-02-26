//! Library for secure reading and writing of binary data:
//!
//! * forbid(unsafe_code)
//! * recursion-free
//! * no_std
//! * panic-free API
//! * support for transactions
#![no_std]

mod read;
mod write;

#[cfg(kani)]
mod proofs;

pub use read::*;
pub use write::*;
