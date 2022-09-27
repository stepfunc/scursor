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
    /// Attempted to write or seek beyond the range of the underlying buffer
    Overflow { length: usize, pos: usize },
}

impl<'a> WriteCursor<'a> {
    pub fn new(dest: &'a mut [u8]) -> WriteCursor<'a> {
        WriteCursor { dest, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn get(&self, range: core::ops::Range<usize>) -> Option<&[u8]> {
        self.dest.get(range)
    }

    pub fn skip(&mut self, count: usize) -> Result<(), WriteError> {
        let new_pos = self
            .pos
            .checked_add(count)
            .ok_or(WriteError::NumericOverflow)?;
        if new_pos > self.dest.len() {
            return Err(WriteError::Overflow {
                length: self.dest.len(),
                pos: new_pos,
            });
        }
        self.pos = new_pos;
        Ok(())
    }

    pub fn seek_to(&mut self, pos: usize) -> Result<(), WriteError> {
        if self.dest.len() < pos {
            return Err(WriteError::Overflow {
                length: self.dest.len(),
                pos,
            });
        }
        self.pos = pos;
        Ok(())
    }

    pub fn transaction<T, R>(&mut self, write: T) -> Result<R, WriteError>
    where
        T: Fn(&mut WriteCursor) -> Result<R, WriteError>,
    {
        let start = self.pos;
        let result = write(self);
        // if an error occurs, rollback to the starting position
        if result.is_err() {
            self.pos = start;
        }
        result
    }

    pub fn at_pos<T, R>(&mut self, pos: usize, write: T) -> Result<R, WriteError>
    where
        T: Fn(&mut WriteCursor) -> Result<R, WriteError>,
    {
        let start = self.pos;
        self.pos = pos;
        let result = write(self);
        // no matter what happens, go back to the starting position
        self.pos = start;
        result
    }

    pub fn written(&self) -> &[u8] {
        self.dest.get(0..self.pos).unwrap_or(&[])
    }

    pub fn written_since(&'a self, pos: usize) -> Result<&'a [u8], WriteError> {
        match self.dest.get(pos..self.pos) {
            Some(x) => Ok(x),
            None => Err(WriteError::NumericOverflow),
        }
    }

    pub fn remaining(&self) -> usize {
        self.dest.len().saturating_sub(self.pos)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), WriteError> {
        let new_pos = self
            .pos
            .checked_add(bytes.len())
            .ok_or(WriteError::NumericOverflow)?;
        match self.dest.get_mut(self.pos..new_pos) {
            Some(x) => x.copy_from_slice(bytes),
            None => {
                return Err(WriteError::Overflow {
                    length: self.dest.len(),
                    pos: new_pos,
                })
            }
        }
        self.pos = new_pos;
        Ok(())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), WriteError> {
        let new_pos = self.pos.checked_add(1).ok_or(WriteError::NumericOverflow)?;
        match self.dest.get_mut(self.pos) {
            Some(x) => {
                *x = value;
                self.pos = new_pos;
                Ok(())
            }
            None => Err(WriteError::Overflow {
                length: self.dest.len(),
                pos: self.pos,
            }),
        }
    }
}

/// little-endian write routines
impl<'a> WriteCursor<'a> {
    pub fn write_u16_le(&mut self, value: u16) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_i16_le(&mut self, value: i16) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_u32_le(&mut self, value: u32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_i32_le(&mut self, value: i32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_u48_le(&mut self, value: u64) -> Result<(), WriteError> {
        let bytes = value.to_le_bytes();
        self.write_bytes(&bytes[0..6])
    }

    pub fn write_f32_le(&mut self, value: f32) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub fn write_f64_le(&mut self, value: f64) -> Result<(), WriteError> {
        self.write_bytes(&value.to_le_bytes())
    }
}

/// big-endian write routines
impl<'a> WriteCursor<'a> {
    pub fn write_u16_be(&mut self, value: u16) -> Result<(), WriteError> {
        self.write_bytes(&value.to_be_bytes())
    }
}

#[cfg(test)]
mod test {

    mod write {
        use super::super::*;

        #[test]
        fn transaction_rolls_back_position_on_failure() {
            let mut buffer = [0u8; 5];
            let mut cursor = WriteCursor::new(&mut buffer);

            cursor.transaction(|cur| cur.write_u16_le(0xCAFE)).unwrap();

            let result = cursor.transaction(|cur| {
                cur.write_u16_le(0xDEAD)?;
                cur.write_u16_le(0xBEEF) // no room for this
            });

            assert_eq!(result, Err(WriteError::Overflow { length: 5, pos: 6 }));
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
                Err(WriteError::Overflow { length: 3, pos: 5 })
            );

            assert_eq!(cursor.written(), &[0x00, 0x00, 0xFF]);
        }
    }
}
