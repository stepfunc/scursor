#![doc = include_str!("../README.md")]
#![no_std]

mod read;
mod write;

#[cfg(kani)]
mod proofs;

pub use read::*;
pub use write::*;
