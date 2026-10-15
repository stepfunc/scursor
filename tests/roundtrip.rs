//! Property tests: every value round-trips through the write and read routines
//! with the expected byte layout, and failed operations change nothing.

use proptest::prelude::*;
use scursor::{ReadCursor, WriteCursor, WriteError};

macro_rules! numeric {
    ($($name:ident: $ty:ty => $write:ident, $read:ident, $to_bytes:ident;)*) => {
        $(
            mod $name {
                use super::*;

                const SIZE: usize = core::mem::size_of::<$ty>();

                proptest! {
                    #[test]
                    fn round_trips(value: $ty, prefix in 0usize..8, spare in 0usize..8) {
                        let mut buffer = vec![0xAA; prefix + SIZE + spare];
                        let mut writer = WriteCursor::new(&mut buffer);
                        writer.skip(prefix).unwrap();
                        writer.$write(value).unwrap();
                        prop_assert_eq!(writer.position(), prefix + SIZE);
                        prop_assert_eq!(writer.written_since(prefix).unwrap(), &value.$to_bytes()[..]);

                        let mut reader = ReadCursor::new(&buffer);
                        reader.read_bytes(prefix).unwrap();
                        // compare bytes so that NaN payloads are checked too
                        prop_assert_eq!(reader.$read().unwrap().to_le_bytes(), value.to_le_bytes());
                        prop_assert_eq!(reader.remaining(), spare);
                    }

                    #[test]
                    fn short_buffer_changes_nothing(value: $ty, prefix in 0usize..8, short in 0usize..SIZE, fill: u8) {
                        let mut buffer = vec![fill; prefix + short];
                        let original = buffer.clone();
                        let mut writer = WriteCursor::new(&mut buffer);
                        writer.skip(prefix).unwrap();
                        prop_assert_eq!(
                            writer.$write(value),
                            Err(WriteError::WriteOverflow { remaining: short, written: SIZE })
                        );
                        prop_assert_eq!(writer.position(), prefix);
                        prop_assert_eq!(&buffer, &original);

                        let mut reader = ReadCursor::new(&original);
                        reader.read_bytes(prefix).unwrap();
                        prop_assert!(reader.$read().is_err());
                        prop_assert_eq!(reader.position(), prefix);
                    }
                }
            }
        )*
    };
}

numeric! {
    u16_le: u16 => write_u16_le, read_u16_le, to_le_bytes;
    u16_be: u16 => write_u16_be, read_u16_be, to_be_bytes;
    i16_le: i16 => write_i16_le, read_i16_le, to_le_bytes;
    i16_be: i16 => write_i16_be, read_i16_be, to_be_bytes;
    u32_le: u32 => write_u32_le, read_u32_le, to_le_bytes;
    u32_be: u32 => write_u32_be, read_u32_be, to_be_bytes;
    i32_le: i32 => write_i32_le, read_i32_le, to_le_bytes;
    i32_be: i32 => write_i32_be, read_i32_be, to_be_bytes;
    u64_le: u64 => write_u64_le, read_u64_le, to_le_bytes;
    u64_be: u64 => write_u64_be, read_u64_be, to_be_bytes;
    i64_le: i64 => write_i64_le, read_i64_le, to_le_bytes;
    i64_be: i64 => write_i64_be, read_i64_be, to_be_bytes;
    u128_le: u128 => write_u128_le, read_u128_le, to_le_bytes;
    u128_be: u128 => write_u128_be, read_u128_be, to_be_bytes;
    i128_le: i128 => write_i128_le, read_i128_le, to_le_bytes;
    i128_be: i128 => write_i128_be, read_i128_be, to_be_bytes;
    f32_le: f32 => write_f32_le, read_f32_le, to_le_bytes;
    f32_be: f32 => write_f32_be, read_f32_be, to_be_bytes;
    f64_le: f64 => write_f64_le, read_f64_le, to_le_bytes;
    f64_be: f64 => write_f64_be, read_f64_be, to_be_bytes;
}

const U48_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

proptest! {
    #[test]
    fn u48_le_round_trips_lower_48_bits(value: u64) {
        let mut buffer = [0u8; 6];
        let mut writer = WriteCursor::new(&mut buffer);
        writer.write_u48_le(value).unwrap();
        prop_assert_eq!(writer.written(), &value.to_le_bytes()[..6]);
        prop_assert_eq!(ReadCursor::new(&buffer).read_u48_le().unwrap(), value & U48_MASK);
    }

    #[test]
    fn u48_be_round_trips_lower_48_bits(value: u64) {
        let mut buffer = [0u8; 6];
        let mut writer = WriteCursor::new(&mut buffer);
        writer.write_u48_be(value).unwrap();
        prop_assert_eq!(writer.written(), &value.to_be_bytes()[2..]);
        prop_assert_eq!(ReadCursor::new(&buffer).read_u48_be().unwrap(), value & U48_MASK);
    }

    #[test]
    fn read_transaction_is_all_or_nothing(
        input in proptest::collection::vec(any::<u8>(), 0..32),
        start in 0usize..32,
        counts in proptest::collection::vec(0usize..16, 0..4),
    ) {
        let mut cursor = ReadCursor::new(&input);
        let _ = cursor.read_bytes(start);
        let before = cursor.position();
        let result = cursor.transaction(|cur| {
            counts.iter().try_for_each(|&count| cur.read_bytes(count).map(|_| ()))
        });
        let total: usize = counts.iter().sum();
        if before + total <= input.len() {
            prop_assert!(result.is_ok());
            prop_assert_eq!(cursor.position(), before + total);
        } else {
            prop_assert!(result.is_err());
            prop_assert_eq!(cursor.position(), before);
        }
    }

    #[test]
    fn write_transaction_restores_position_on_failure(
        len in 0usize..32,
        start in 0usize..32,
        values in proptest::collection::vec(any::<u32>(), 0..8),
    ) {
        let mut buffer = vec![0u8; len];
        let mut cursor = WriteCursor::new(&mut buffer);
        let _ = cursor.seek_to(start);
        let before = cursor.position();
        let result = cursor.transaction(|cur| {
            values.iter().try_for_each(|&value| cur.write_u32_le(value))
        });
        if before + 4 * values.len() <= len {
            prop_assert!(result.is_ok());
            prop_assert_eq!(cursor.position(), before + 4 * values.len());
        } else {
            prop_assert!(result.is_err());
            prop_assert_eq!(cursor.position(), before);
        }
    }
}
