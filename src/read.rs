/// Secure read-only cursor
#[derive(Copy, Clone, Debug)]
pub struct ReadCursor<'a> {
    pos: usize,
    input: &'a [u8],
}

/// Error returned when insufficient data exists to deserialize requested type
#[derive(Copy, Clone, Debug)]
pub struct ReadError;

/// Error when asserting that there are no remaining bytes
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TrailingBytes {
    /// number of non-zero trailing bytes
    pub count: core::num::NonZeroUsize,
}

impl<'a> ReadCursor<'a> {
    /// Construct a cursor from a borrowed slice
    pub fn new(input: &'a [u8]) -> Self {
        Self { pos: 0, input }
    }

    /// Read a single unsigned byte from the cursor
    pub fn read_u8(&mut self) -> Result<u8, ReadError> {
        match self.input.get(self.pos) {
            Some(x) => {
                let pos = self.pos.checked_add(1).ok_or(ReadError)?;
                self.pos = pos;
                Ok(*x)
            }
            None => Err(ReadError),
        }
    }

    /// Expect the cursor to be empty or return and error indicating how many trailing
    /// bytes are present
    pub fn expect_empty(&self) -> Result<(), TrailingBytes> {
        let remaining = self.remaining();
        match core::num::NonZeroUsize::new(remaining) {
            None => Ok(()),
            Some(x) => Err(TrailingBytes { count: x }),
        }
    }

    /// Return the number of bytes remaining to be read
    pub fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.pos)
    }

    /// Return the position of the cursor within the original input slice
    ///
    /// This is synonymous with the number of bytes consumed by the cursor
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Perform a transaction on the buffer, returning it to its initial
    /// state if an error occurs
    pub fn transaction<T, R, E>(&mut self, read: T) -> Result<R, E>
    where
        T: FnOnce(&mut ReadCursor) -> Result<R, E>,
    {
        let start = self.pos;
        let result = read(self);
        // if an error occurs, rollback to the starting position
        if result.is_err() {
            self.pos = start;
        }
        result
    }

    /// Return true if there are no more bytes remaining to be read
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// Read the rest of the buffer as a borrowed slice
    pub fn read_all(&mut self) -> &'a [u8] {
        match self.input.get(self.pos..) {
            None => &[],
            Some(x) => {
                self.pos = self.input.len();
                x
            }
        }
    }

    /// Read a count of bytes as a borrowed slice
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ReadError> {
        let end = self.pos.checked_add(count).ok_or(ReadError)?;
        let ret = self.input.get(self.pos..end).ok_or(ReadError)?;
        self.pos = end;
        Ok(ret)
    }

    /// Read a fixed-size array of bytes
    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ReadError> {
        let chunk = self
            .input
            .get(self.pos..)
            .and_then(|s| s.first_chunk::<N>())
            .ok_or(ReadError)?;
        self.pos = self.pos.checked_add(N).ok_or(ReadError)?;
        Ok(*chunk)
    }
}

/// little-endian read routines
impl<'a> ReadCursor<'a> {
    /// Read a u16 from a little-endian representation
    pub fn read_u16_le(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes(self.read_array()?))
    }

    /// Read a i16 from a little-endian representation
    pub fn read_i16_le(&mut self) -> Result<i16, ReadError> {
        Ok(i16::from_le_bytes(self.read_array()?))
    }

    /// Read a u32 from a little-endian representation
    pub fn read_u32_le(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.read_array()?))
    }

    /// Read a i32 from a little-endian representation
    pub fn read_i32_le(&mut self) -> Result<i32, ReadError> {
        Ok(i32::from_le_bytes(self.read_array()?))
    }

    /// Read a 48-bit unsigned number from a little-endian representation, store it in the first 6 bytes of a u64
    pub fn read_u48_le(&mut self) -> Result<u64, ReadError> {
        let [b0, b1, b2, b3, b4, b5] = self.read_array::<6>()?;
        Ok(u64::from_le_bytes([b0, b1, b2, b3, b4, b5, 0, 0]))
    }

    /// Read a u64 number from a little-endian representation
    pub fn read_u64_le(&mut self) -> Result<u64, ReadError> {
        Ok(u64::from_le_bytes(self.read_array()?))
    }

    /// Read a i64 number from a little-endian representation
    pub fn read_i64_le(&mut self) -> Result<i64, ReadError> {
        Ok(i64::from_le_bytes(self.read_array()?))
    }

    /// Read a u128 number from a little-endian representation
    pub fn read_u128_le(&mut self) -> Result<u128, ReadError> {
        Ok(u128::from_le_bytes(self.read_array()?))
    }

    /// Read a i128 number from a little-endian representation
    pub fn read_i128_le(&mut self) -> Result<i128, ReadError> {
        Ok(i128::from_le_bytes(self.read_array()?))
    }

    /// Read an IEEE-754 f32 from a little-endian representation
    pub fn read_f32_le(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_le_bytes(self.read_array()?))
    }

    /// Read an IEEE-754 f64 from a little-endian representation
    pub fn read_f64_le(&mut self) -> Result<f64, ReadError> {
        Ok(f64::from_le_bytes(self.read_array()?))
    }
}

/// big-endian read routines
impl<'a> ReadCursor<'a> {
    /// Read a u16 from a big-endian representation
    pub fn read_u16_be(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_be_bytes(self.read_array()?))
    }

    /// Read a i16 from a big-endian representation
    pub fn read_i16_be(&mut self) -> Result<i16, ReadError> {
        Ok(i16::from_be_bytes(self.read_array()?))
    }

    /// Read a u32 from a big-endian representation
    pub fn read_u32_be(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_be_bytes(self.read_array()?))
    }

    /// Read a i32 from a big-endian representation
    pub fn read_i32_be(&mut self) -> Result<i32, ReadError> {
        Ok(i32::from_be_bytes(self.read_array()?))
    }

    /// Read a 48-bit unsigned number from a big-endian representation, store it in the first 6 bytes of a u64
    pub fn read_u48_be(&mut self) -> Result<u64, ReadError> {
        let [b0, b1, b2, b3, b4, b5] = self.read_array::<6>()?;
        Ok(u64::from_be_bytes([0, 0, b0, b1, b2, b3, b4, b5]))
    }

    /// Read a u64 from a big-endian representation
    pub fn read_u64_be(&mut self) -> Result<u64, ReadError> {
        Ok(u64::from_be_bytes(self.read_array()?))
    }

    /// Read a i64 from a big-endian representation
    pub fn read_i64_be(&mut self) -> Result<i64, ReadError> {
        Ok(i64::from_be_bytes(self.read_array()?))
    }

    /// Read a u128 from a big-endian representation
    pub fn read_u128_be(&mut self) -> Result<u128, ReadError> {
        Ok(u128::from_be_bytes(self.read_array()?))
    }

    /// Read a i128 from a big-endian representation
    pub fn read_i128_be(&mut self) -> Result<i128, ReadError> {
        Ok(i128::from_be_bytes(self.read_array()?))
    }

    /// Read an IEEE-754 f32 from a big-endian representation
    pub fn read_f32_be(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_be_bytes(self.read_array()?))
    }

    /// Read an IEEE-754 f64 from a big-endian representation
    pub fn read_f64_be(&mut self) -> Result<f64, ReadError> {
        Ok(f64::from_be_bytes(self.read_array()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_u8() {
        let mut cursor = ReadCursor::new(&[0xCA, 0xFE]);
        assert_eq!(cursor.position(), 0);

        assert_eq!(cursor.remaining(), 2);
        assert_eq!(cursor.read_u8().unwrap(), 0xCA);
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.remaining(), 1);
        assert_eq!(cursor.read_u8().unwrap(), 0xFE);
        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.remaining(), 0);
        assert!(cursor.read_u8().is_err());
        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn can_read_u16_le() {
        let mut cursor = ReadCursor::new(&[0xCA, 0xFE]);
        assert_eq!(cursor.read_u16_le().unwrap(), 0xFECA);
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(cursor.position(), 2);
    }

    #[test]
    fn can_read_u32_le() {
        let mut cursor = ReadCursor::new(&[0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(cursor.read_u32_le().unwrap(), 0xDDCCBBAA);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn can_read_u48_le() {
        let mut cursor = ReadCursor::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(cursor.read_u48_le().unwrap(), 0x00FFEEDDCCBBAA);
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    fn can_read_u64_le() {
        let mut cursor = ReadCursor::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01]);
        assert_eq!(cursor.read_u64_le().unwrap(), 0x0100FFEEDDCCBBAA);
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(cursor.position(), 8);
    }

    #[test]
    fn can_read_f64_le() {
        let tests: [(f64, [u8; 8]); 2] = [
            (0.0, [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            (f64::MAX, [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xEF, 0x7F]),
        ];

        for (value, bytes) in tests {
            let mut cursor = ReadCursor::new(&bytes);
            assert_eq!(cursor.read_f64_le().unwrap(), value);
            assert_eq!(cursor.remaining(), 0);
        }
    }

    #[test]
    fn can_read_f64_le_nan() {
        let mut cursor = ReadCursor::new(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x7F]);
        let value = cursor.read_f64_le().unwrap();
        assert!(value.is_nan());
    }

    #[test]
    fn can_read_u48_be() {
        let mut cursor = ReadCursor::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(cursor.read_u48_be().unwrap(), 0x00AABBCCDDEEFF);
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    fn can_read_f32_be() {
        let mut cursor = ReadCursor::new(&[0x40, 0x49, 0x0F, 0xDB]); // approx pi
        let val = cursor.read_f32_be().unwrap();
        assert!((val - 3.14159265).abs() < 1e-6);
    }

    #[test]
    fn can_read_f64_be() {
        let mut cursor = ReadCursor::new(&[0x40, 0x09, 0x21, 0xFB, 0x54, 0x44, 0x2D, 0x18]); // approx pi
        let val = cursor.read_f64_be().unwrap();
        assert!((val - 3.141592653589793).abs() < 1e-15);
    }

    #[test]
    fn can_read_i32_be() {
        let mut cursor = ReadCursor::new(&[0xFF, 0xFF, 0xFF, 0xFE]);
        assert_eq!(cursor.read_i32_be().unwrap(), -2);
    }

    #[test]
    fn can_read_i16_be() {
        let mut cursor = ReadCursor::new(&[0xFF, 0xFD]);
        assert_eq!(cursor.read_i16_be().unwrap(), -3);
    }

    #[test]
    fn can_read_u16_be() {
        let mut cursor = ReadCursor::new(&[0xCA, 0xFE]);
        assert_eq!(cursor.read_u16_be().unwrap(), 0xCAFE);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn can_read_u32_lb() {
        let mut cursor = ReadCursor::new(&[0xDD, 0xCC, 0xBB, 0xAA]);
        assert_eq!(cursor.read_u32_be().unwrap(), 0xDDCCBBAA);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn can_read_u64_be() {
        let mut cursor = ReadCursor::new(&[0x01, 0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA]);
        assert_eq!(cursor.read_u64_be().unwrap(), 0x0100FFEEDDCCBBAA);
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn can_read_u128_le() {
        let mut cursor = ReadCursor::new(&[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ]);
        assert_eq!(
            cursor.read_u128_le().unwrap(),
            0x0F0E0D0C0B0A09080706050403020100
        );
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(cursor.position(), 16);
    }

    #[test]
    fn can_read_u128_be() {
        let mut cursor = ReadCursor::new(&[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F,
        ]);
        assert_eq!(
            cursor.read_u128_be().unwrap(),
            0x000102030405060708090A0B0C0D0E0F
        );
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(cursor.position(), 16);
    }

    #[test]
    fn can_read_array() {
        let mut cursor = ReadCursor::new(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00]);
        assert_eq!(cursor.read_array::<4>().unwrap(), [0xCA, 0xFE, 0xBA, 0xBE]);
        assert_eq!(cursor.remaining(), 1);
        assert_eq!(cursor.position(), 4);

        // insufficient bytes
        assert!(cursor.read_array::<2>().is_err());
    }
}
