//! Link-time panic check for every public routine.
//!
//! Every public function is annotated with `#[no_panic]` when the `no-panic`
//! feature is enabled. The check happens when this test binary is linked, so
//! each routine must be called here. `tests/no_panic_coverage.rs` verifies that
//! this file stays in sync with the public API.
//!
//! Run with:
//!
//! ```text
//! cargo test --profile no-panic --features no-panic --test no_panic
//! ```

use scursor::{ReadCursor, WriteCursor};
use std::hint::black_box;

#[inline(never)]
fn exercise_read_cursor(input: &[u8], count: usize) {
    let mut cursor = ReadCursor::new(input);

    black_box(cursor.read_u8().ok());
    black_box(cursor.read_bytes(count).ok());
    black_box(cursor.read_array::<0>().ok());
    black_box(cursor.read_array::<3>().ok());
    black_box(cursor.read_array::<32>().ok());

    black_box(cursor.read_u16_le().ok());
    black_box(cursor.read_i16_le().ok());
    black_box(cursor.read_u32_le().ok());
    black_box(cursor.read_i32_le().ok());
    black_box(cursor.read_u48_le().ok());
    black_box(cursor.read_u64_le().ok());
    black_box(cursor.read_i64_le().ok());
    black_box(cursor.read_u128_le().ok());
    black_box(cursor.read_i128_le().ok());
    black_box(cursor.read_f32_le().ok());
    black_box(cursor.read_f64_le().ok());

    black_box(cursor.read_u16_be().ok());
    black_box(cursor.read_i16_be().ok());
    black_box(cursor.read_u32_be().ok());
    black_box(cursor.read_i32_be().ok());
    black_box(cursor.read_u48_be().ok());
    black_box(cursor.read_u64_be().ok());
    black_box(cursor.read_i64_be().ok());
    black_box(cursor.read_u128_be().ok());
    black_box(cursor.read_i128_be().ok());
    black_box(cursor.read_f32_be().ok());
    black_box(cursor.read_f64_be().ok());

    black_box(cursor.transaction(|cur| cur.read_bytes(count)).ok());
    black_box(cursor.expect_empty().ok());
    black_box(cursor.remaining());
    black_box(cursor.position());
    black_box(cursor.is_empty());
    black_box(cursor.read_all());
}

#[inline(never)]
fn exercise_write_cursor(dest: &mut [u8], value: u128, pos: usize) {
    let mut cursor = WriteCursor::new(dest);

    black_box(cursor.write_u8(value as u8).ok());
    black_box(cursor.write_bytes(black_box(&[0xCA, 0xFE])).ok());
    black_box(cursor.skip(pos).ok());
    black_box(cursor.seek_to(pos).ok());

    black_box(cursor.write_u16_le(value as u16).ok());
    black_box(cursor.write_i16_le(value as i16).ok());
    black_box(cursor.write_u32_le(value as u32).ok());
    black_box(cursor.write_i32_le(value as i32).ok());
    black_box(cursor.write_u48_le(value as u64).ok());
    black_box(cursor.write_u64_le(value as u64).ok());
    black_box(cursor.write_i64_le(value as i64).ok());
    black_box(cursor.write_u128_le(value).ok());
    black_box(cursor.write_i128_le(value as i128).ok());
    black_box(cursor.write_f32_le(value as f32).ok());
    black_box(cursor.write_f64_le(value as f64).ok());

    black_box(cursor.write_u16_be(value as u16).ok());
    black_box(cursor.write_i16_be(value as i16).ok());
    black_box(cursor.write_u32_be(value as u32).ok());
    black_box(cursor.write_i32_be(value as i32).ok());
    black_box(cursor.write_u48_be(value as u64).ok());
    black_box(cursor.write_u64_be(value as u64).ok());
    black_box(cursor.write_i64_be(value as i64).ok());
    black_box(cursor.write_u128_be(value).ok());
    black_box(cursor.write_i128_be(value as i128).ok());
    black_box(cursor.write_f32_be(value as f32).ok());
    black_box(cursor.write_f64_be(value as f64).ok());

    black_box(
        cursor
            .transaction(|cur| cur.write_u16_le(value as u16))
            .ok(),
    );
    black_box(
        cursor
            .at_pos(pos, |cur| cur.write_u32_be(value as u32))
            .ok(),
    );
    black_box(cursor.written_since(pos).map(<[u8]>::len).ok());
    black_box(cursor.get(pos..pos.wrapping_add(4)).map(<[u8]>::len));
    black_box(cursor.written().len());
    black_box(cursor.remaining());
    black_box(cursor.position());
}

#[test]
fn public_api_links_without_panic_paths() {
    let input: Vec<u8> = (0..=255).collect();
    exercise_read_cursor(black_box(&input), black_box(7));

    let mut dest = [0u8; 256];
    exercise_write_cursor(black_box(&mut dest), black_box(u128::MAX), black_box(5));
}
