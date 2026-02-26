/// Secure write cursor
///
/// Provides routines for incrementally writing to a borrowed slice
#[derive(Debug)]
pub struct WriteCursor<'a> {
    dest: &'a mut [u8],
    pos: usize,
}

/// Error type returned when a seek is requested beyond the bounds of the buffer or numeric range
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WriteError {
    /// Numeric overflow occurred in a write or seek
    NumericOverflow,
    /// Attempted to write beyond the range of the underlying buffer
    WriteOverflow {
        /// number of bytes remaining to be written
        remaining: usize,
        /// number of bytes requested to be written
        written: usize,
    },
    /// Attempted to seek to a position larger than the length of the buffer
    BadSeek {
        /// length of the underling buffer
        length: usize,
        /// requested seek position
        pos: usize,
    },
}

impl<'a> WriteCursor<'a> {
    /// Construct a cursor from a borrowed mutable slice
    pub fn new(dest: &'a mut [u8]) -> WriteCursor<'a> {
        WriteCursor { dest, pos: 0 }
    }

    /// Current position of the cursor within the underlying slice
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Get a range within the underlying slice
    pub fn get(&self, range: core::ops::Range<usize>) -> Option<&[u8]> {
        self.dest.get(range)
    }

    /// Advance the cursor a count of bytes
    pub fn skip(&mut self, count: usize) -> Result<(), WriteError> {
        let new_pos = self
            .pos
            .checked_add(count)
            .ok_or(WriteError::NumericOverflow)?;
        self.seek_to(new_pos)
    }

    /// Seek the cursor to an absolute position within the underlying slice
    pub fn seek_to(&mut self, pos: usize) -> Result<(), WriteError> {
        if self.dest.len() < pos {
            return Err(WriteError::BadSeek {
                length: self.dest.len(),
                pos,
            });
        }
        self.pos = pos;
        Ok(())
    }

    /// Perform a write transaction which returns the cursor to the original
    /// position if an error occurs
    pub fn transaction<T, R>(&mut self, write: T) -> Result<R, WriteError>
    where
        T: FnOnce(&mut WriteCursor) -> Result<R, WriteError>,
    {
        let start = self.pos;
        let result = write(self);
        // if an error occurs, rollback to the starting position
        if result.is_err() {
            self.pos = start;
        }
        result
    }

    /// Perform a write transaction at particular position. The cursor is always
    /// returned to its original position regardless of the success or failure of
    /// the operation
    pub fn at_pos<T, R>(&mut self, pos: usize, write: T) -> Result<R, WriteError>
    where
        T: Fn(&mut WriteCursor) -> Result<R, WriteError>,
    {
        let start = self.pos;
        self.seek_to(pos)?;
        let result = write(self);
        // no matter what happens, go back to the starting position
        self.pos = start;
        result
    }

    /// Return the data that has been written so far as a borrowed slice
    pub fn written(&self) -> &[u8] {
        self.dest.get(0..self.pos).unwrap_or(&[])
    }

    /// Return the data that has been written since a particular write position
    pub fn written_since(&'a self, pos: usize) -> Result<&'a [u8], WriteError> {
        match self.dest.get(pos..self.pos) {
            Some(x) => Ok(x),
            None => Err(WriteError::NumericOverflow),
        }
    }

    /// Number of bytes remaining to be written
    pub fn remaining(&self) -> usize {
        self.dest.len().saturating_sub(self.pos)
    }

    /// Write a slice of bytes to the cursor
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), WriteError> {
        let new_pos = self
            .pos
            .checked_add(bytes.len())
            .ok_or(WriteError::NumericOverflow)?;
        match self.dest.get_mut(self.pos..new_pos) {
            Some(x) => x.copy_from_slice(bytes),
            None => {
                return Err(WriteError::WriteOverflow {
                    remaining: self.remaining(),
                    written: bytes.len(),
                })
            }
        }
        self.pos = new_pos;
        Ok(())
    }

    /// Write a single u8 to the cursor
    pub fn write_u8(&mut self, value: u8) -> Result<(), WriteError> {
        let new_pos = self.pos.checked_add(1).ok_or(WriteError::NumericOverflow)?;
        match self.dest.get_mut(self.pos) {
            Some(x) => {
                *x = value;
                self.pos = new_pos;
                Ok(())
            }
            None => Err(WriteError::WriteOverflow {
                remaining: 0,
                written: 1,
            }),
        }
    }
}

/// little-endian write routines
impl<'a> WriteCursor<'a> {
    /// Write a u16 in little-endian format
    pub fn write_u16_le(&mut self, value: u16) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a i16 in little-endian format
    pub fn write_i16_le(&mut self, value: i16) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a u32 in little-endian format
    pub fn write_u32_le(&mut self, value: u32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a i32 in little-endian format
    pub fn write_i32_le(&mut self, value: i32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write the lower 6-bytes of a u64 (u48) in little-endian format
    pub fn write_u48_le(&mut self, value: u64) -> Result<(), WriteError> {
        let [b0, b1, b2, b3, b4, b5, _, _] = value.to_le_bytes();
        self.write_bytes(&[b0, b1, b2, b3, b4, b5])
    }

    /// Write a u64 in little-endian format
    pub fn write_u64_le(&mut self, value: u64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a i64 in little-endian format
    pub fn write_i64_le(&mut self, value: i64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write an IEEE-754 f32 in little endian format
    pub fn write_f32_le(&mut self, value: f32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write an IEEE-754 f64 in little endian format
    pub fn write_f64_le(&mut self, value: f64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a u128 in little-endian format
    pub fn write_u128_le(&mut self, value: u128) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Write a i128 in little-endian format
    pub fn write_i128_le(&mut self, value: i128) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }
}

/// big-endian write routines
impl<'a> WriteCursor<'a> {
    /// Write a u16 in big-endian format
    pub fn write_u16_be(&mut self, value: u16) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write a u32 in big-endian format
    pub fn write_u32_be(&mut self, value: u32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write a i32 in big-endian format
    pub fn write_i32_be(&mut self, value: i32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write the lower 6-bytes of a u64 (u48) in big-endian format
    pub fn write_u48_be(&mut self, value: u64) -> Result<(), WriteError> {
        let [_, _, b2, b3, b4, b5, b6, b7] = value.to_be_bytes();
        self.write_bytes(&[b2, b3, b4, b5, b6, b7])
    }

    /// Write a u64 in big-endian format
    pub fn write_u64_be(&mut self, value: u64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write a i64 in big-endian format
    pub fn write_i64_be(&mut self, value: i64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write a u128 in big-endian format
    pub fn write_u128_be(&mut self, value: u128) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write a i128 in big-endian format
    pub fn write_i128_be(&mut self, value: i128) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write an IEEE-754 f32 in big-endian format
    pub fn write_f32_be(&mut self, value: f32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }

    /// Write an IEEE-754 f64 in big-endian format
    pub fn write_f64_be(&mut self, value: f64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn transaction_rolls_back_position_on_failure() {
        let mut buffer = [0u8; 5];
        let mut cursor = WriteCursor::new(&mut buffer);

        cursor.transaction(|cur| cur.write_u16_le(0xCAFE)).unwrap();

        let result = cursor.transaction(|cur| {
            cur.write_u16_le(0xDEAD)?;
            cur.write_u16_le(0xBEEF) // no room for this
        });

        assert_eq!(
            result,
            Err(WriteError::WriteOverflow {
                remaining: 1,
                written: 2
            })
        );
        assert_eq!(cursor.written(), &[0xFE, 0xCA]);
    }

    #[test]
    fn from_pos_seeks_back_to_original_position_on_success() {
        let mut buffer = [0u8; 3];
        let mut cursor = WriteCursor::new(&mut buffer);

        cursor.skip(2).unwrap();
        cursor.write_u8(0xFF).unwrap();

        cursor.at_pos(0, |cur| cur.write_u16_le(0xCAFE)).unwrap();

        assert_eq!(cursor.written(), &[0xFE, 0xCA, 0xFF]);
    }

    #[test]
    fn write_at_seeks_back_to_original_position_on_failure() {
        let mut buffer = [0u8; 3];
        let mut cursor = WriteCursor::new(&mut buffer);

        cursor.skip(2).unwrap();
        cursor.write_u8(0xFF).unwrap();

        assert_eq!(
            cursor.at_pos(5, |cur| cur.write_u8(0xAA)),
            Err(WriteError::BadSeek { length: 3, pos: 5 })
        );

        assert_eq!(cursor.written(), &[0x00, 0x00, 0xFF]);
    }

    #[test]
    fn can_write_u16_be() {
        let mut buffer = [0u8; 2];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_u16_be(0xCAFE).unwrap();
        assert_eq!(cursor.written(), &[0xCA, 0xFE]);
    }

    #[test]
    fn can_write_u32_be() {
        let mut buffer = [0u8; 4];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_u32_be(0xDEADBEEF).unwrap();
        assert_eq!(cursor.written(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn can_write_i32_be() {
        let mut buffer = [0u8; 4];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_i32_be(-2).unwrap();
        assert_eq!(cursor.written(), &[0xFF, 0xFF, 0xFF, 0xFE]);
    }

    #[test]
    fn can_write_u48_be() {
        let mut buffer = [0u8; 6];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_u48_be(0x00AABBCCDDEEFF).unwrap();
        assert_eq!(cursor.written(), &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn can_write_u64_be() {
        let mut buffer = [0u8; 8];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_u64_be(0x0102030405060708).unwrap();
        assert_eq!(
            cursor.written(),
            &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
    }

    #[test]
    fn can_write_f32_be() {
        let mut buffer = [0u8; 4];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_f32_be(3.1415927).unwrap();
        assert_eq!(cursor.written(), &[0x40, 0x49, 0x0F, 0xDB]);
    }

    #[test]
    fn can_write_f64_be() {
        let mut buffer = [0u8; 8];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_f64_be(3.141592653589793).unwrap();
        assert_eq!(
            cursor.written(),
            &[0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]
        );
    }

    #[test]
    fn can_write_u128_le() {
        let mut buffer = [0u8; 16];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor
            .write_u128_le(0x0F0E0D0C0B0A09080706050403020100)
            .unwrap();
        assert_eq!(
            cursor.written(),
            &[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F
            ]
        );
    }

    #[test]
    fn can_write_u128_be() {
        let mut buffer = [0u8; 16];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor
            .write_u128_be(0x000102030405060708090A0B0C0D0E0F)
            .unwrap();
        assert_eq!(
            cursor.written(),
            &[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
                0x0E, 0x0F
            ]
        );
    }

    #[test]
    fn can_write_u8() {
        let mut buffer = [0u8; 2];
        let mut cursor = WriteCursor::new(&mut buffer);
        cursor.write_u8(0xCA).unwrap();
        cursor.write_u8(0xFE).unwrap();
        assert_eq!(cursor.written(), &[0xCA, 0xFE]);
    }

    #[test]
    fn write_overflow_returns_error() {
        let mut buffer = [0u8; 3];
        let mut cursor = WriteCursor::new(&mut buffer);
        assert_eq!(
            cursor.write_u32_le(0xDEADBEEF),
            Err(WriteError::WriteOverflow {
                remaining: 3,
                written: 4
            })
        );
    }
}
