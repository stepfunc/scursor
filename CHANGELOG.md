### 0.5.0 ###
* :star: Optimized all integer and floating point read operations using the stabilized `slice::first_chunk` (Rust 1.77+).
* :star: Added missing symmetric big-endian read/write routines for `u32`, `i32`, `f32`, `f64`, and `u48`.
* :star: Integrated formal verification using Kani to mathematically prove panic-freedom.
* Strictly enforced clippy lints for `panic`, `unwrap`, `expect`, and `indexing_slicing`.
* Updated documentation and CI badges.

### 0.4.0 ###
* :star: Add `read_array` method for const-generic fixed-size arrays.
* :star: Add u64/i64 write methods (little-endian and big-endian).

### 0.3.0 ###
* :star: Add u128/i128 read and write support (little-endian and big-endian).
* Document panic-free design in README.
* Use `into()` instead of `as` for widening conversions.

### 0.2.0 ###
* Specify lints in Cargo.toml instead of lib.rs.
* :star: Add method to `ReadCursor` to retrieve the position.
* :star: Argument to read/write transaction are now `FnOnce` per [#1](https://github.com/stepfunc/scursor/issues/1).