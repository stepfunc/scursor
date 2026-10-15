//! Link-time panic check for every public routine.
//!
//! Every public function is annotated with `#[no_panic]` when the `no-panic`
//! feature is enabled. The check happens when this test binary is linked, and
//! the linker only examines code that is linked, so this file takes the address
//! of every public routine. Passing the address through `black_box` forces a
//! standalone copy of each routine that knows nothing about its arguments, so
//! the check covers every input rather than particular values.
//!
//! `tests/no_panic_coverage.rs` verifies that every public routine is listed.
//!
//! Run with:
//!
//! ```text
//! cargo test --profile no-panic --features no-panic --test no_panic
//! ```

use scursor::{ReadCursor, ReadError, WriteCursor, WriteError};
use std::hint::black_box;

/// Callback for the `transaction` and `at_pos` instances. Those routines are
/// panic-free only if the caller's callback is, so they are checked with a
/// callback that is.
fn read_callback(cursor: &mut ReadCursor<'_>) -> Result<u8, ReadError> {
    cursor.read_u8()
}

fn write_callback(cursor: &mut WriteCursor<'_>) -> Result<(), WriteError> {
    cursor.write_u8(0)
}

fn read_transaction<'a, T>(_: T) -> fn(&mut ReadCursor<'a>, T) -> Result<u8, ReadError>
where
    T: FnOnce(&mut ReadCursor<'a>) -> Result<u8, ReadError>,
{
    ReadCursor::transaction
}

fn write_transaction<'a, T>(_: T) -> fn(&mut WriteCursor<'a>, T) -> Result<(), WriteError>
where
    T: FnOnce(&mut WriteCursor<'_>) -> Result<(), WriteError>,
{
    WriteCursor::transaction
}

fn write_at_pos<'a, T>(_: T) -> fn(&mut WriteCursor<'a>, usize, T) -> Result<(), WriteError>
where
    T: FnOnce(&mut WriteCursor<'_>) -> Result<(), WriteError>,
{
    WriteCursor::at_pos
}

#[test]
fn public_api_links_without_panic_paths() {
    black_box::<fn(_) -> _>(ReadCursor::new);
    black_box::<fn(_) -> _>(ReadCursor::read_u8);
    black_box::<fn(_) -> _>(ReadCursor::expect_empty);
    black_box::<fn(_) -> _>(ReadCursor::remaining);
    black_box::<fn(_) -> _>(ReadCursor::position);
    black_box(read_transaction(read_callback));
    black_box::<fn(_) -> _>(ReadCursor::is_empty);
    black_box::<fn(_) -> _>(ReadCursor::read_all);
    black_box::<fn(_, _) -> _>(ReadCursor::read_bytes);
    black_box::<fn(_) -> _>(ReadCursor::read_array::<0>);
    black_box::<fn(_) -> _>(ReadCursor::read_array::<3>);
    black_box::<fn(_) -> _>(ReadCursor::read_array::<32>);

    black_box::<fn(_) -> _>(ReadCursor::read_u16_le);
    black_box::<fn(_) -> _>(ReadCursor::read_i16_le);
    black_box::<fn(_) -> _>(ReadCursor::read_u32_le);
    black_box::<fn(_) -> _>(ReadCursor::read_i32_le);
    black_box::<fn(_) -> _>(ReadCursor::read_u48_le);
    black_box::<fn(_) -> _>(ReadCursor::read_u64_le);
    black_box::<fn(_) -> _>(ReadCursor::read_i64_le);
    black_box::<fn(_) -> _>(ReadCursor::read_u128_le);
    black_box::<fn(_) -> _>(ReadCursor::read_i128_le);
    black_box::<fn(_) -> _>(ReadCursor::read_f32_le);
    black_box::<fn(_) -> _>(ReadCursor::read_f64_le);

    black_box::<fn(_) -> _>(ReadCursor::read_u16_be);
    black_box::<fn(_) -> _>(ReadCursor::read_i16_be);
    black_box::<fn(_) -> _>(ReadCursor::read_u32_be);
    black_box::<fn(_) -> _>(ReadCursor::read_i32_be);
    black_box::<fn(_) -> _>(ReadCursor::read_u48_be);
    black_box::<fn(_) -> _>(ReadCursor::read_u64_be);
    black_box::<fn(_) -> _>(ReadCursor::read_i64_be);
    black_box::<fn(_) -> _>(ReadCursor::read_u128_be);
    black_box::<fn(_) -> _>(ReadCursor::read_i128_be);
    black_box::<fn(_) -> _>(ReadCursor::read_f32_be);
    black_box::<fn(_) -> _>(ReadCursor::read_f64_be);

    black_box::<fn(_) -> _>(WriteCursor::new);
    black_box::<fn(_) -> _>(WriteCursor::position);
    black_box::<fn(_, _) -> _>(WriteCursor::get);
    black_box::<fn(_, _) -> _>(WriteCursor::skip);
    black_box::<fn(_, _) -> _>(WriteCursor::seek_to);
    black_box(write_transaction(write_callback));
    black_box(write_at_pos(write_callback));
    black_box::<fn(_) -> _>(WriteCursor::written);
    black_box::<fn(_, _) -> _>(WriteCursor::written_since);
    black_box::<fn(_) -> _>(WriteCursor::remaining);
    black_box::<fn(_, _) -> _>(WriteCursor::write_bytes);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u8);

    black_box::<fn(_, _) -> _>(WriteCursor::write_u16_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i16_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u32_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i32_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u48_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u64_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i64_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_f32_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_f64_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u128_le);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i128_le);

    black_box::<fn(_, _) -> _>(WriteCursor::write_u16_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i16_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u32_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i32_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u48_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u64_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i64_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_u128_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_i128_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_f32_be);
    black_box::<fn(_, _) -> _>(WriteCursor::write_f64_be);
}
