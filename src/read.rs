/// custom read-only cursor
#[derive(Copy, Clone, Debug)]
pub struct ReadCursor<'a> {
    pos: usize,
    input: &'a [u8],
}

/// error returned when insufficient data exists to deserialize requested type
#[derive(Copy, Clone, Debug)]
pub struct ReadError;

impl<'a> ReadCursor<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { pos: 0, input }
    }

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

    pub fn remaining(&self) -> usize {
        self.input.len().saturating_sub(self.pos)
    }

    pub fn transaction<T, R, E>(&mut self, mut read: T) -> Result<R, E>
    where
        T: FnMut(&mut ReadCursor) -> Result<R, E>,
    {
        let start = self.pos;
        let result = read(self);
        // if an error occurs, rollback to the starting position
        if result.is_err() {
            self.pos = start;
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn read_all(&mut self) -> &'a [u8] {
        match self.input.get(self.pos..) {
            None => &[],
            Some(x) => {
                self.pos = self.input.len();
                x
            }
        }
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ReadError> {
        let end = self.pos.checked_add(count).ok_or(ReadError)?;
        let ret = self.input.get(self.pos..end).ok_or(ReadError)?;
        self.pos = end;
        Ok(ret)
    }
}

/// little-endian read routines
impl<'a> ReadCursor<'a> {
    pub fn read_u16_le(&mut self) -> Result<u16, ReadError> {
        Ok((self.read_u8()? as u16) | (self.read_u8()? as u16) << 8)
    }

    pub fn read_i16_le(&mut self) -> Result<i16, ReadError> {
        self.read_u16_le().map(|x| x as i16)
    }

    pub fn read_u32_le(&mut self) -> Result<u32, ReadError> {
        Ok((self.read_u16_le()?) as u32 | ((self.read_u16_le()? as u32) << 16))
    }

    pub fn read_i32_le(&mut self) -> Result<i32, ReadError> {
        self.read_u32_le().map(|x| x as i32)
    }

    pub fn read_u48_le(&mut self) -> Result<u64, ReadError> {
        let low = self.read_u32_le()?;
        let high = self.read_u16_le()?;

        Ok((high as u64) << 32 | (low as u64))
    }

    pub fn read_u64_le(&mut self) -> Result<u64, ReadError> {
        let low = self.read_u32_le()?;
        let high = self.read_u32_le()?;

        Ok((high as u64) << 32 | (low as u64))
    }

    pub fn read_i64_le(&mut self) -> Result<i64, ReadError> {
        self.read_u64_le().map(|x| x as i64)
    }

    pub fn read_f32_le(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_bits(self.read_u32_le()?))
    }

    pub fn read_f64_le(&mut self) -> Result<f64, ReadError> {
        Ok(f64::from_bits(self.read_u64_le()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_read_u8() {
        let mut cursor = ReadCursor::new(&[0xCA, 0xFE]);

        assert_eq!(cursor.remaining(), 2);
        assert_eq!(cursor.read_u8().unwrap(), 0xCA);
        assert_eq!(cursor.remaining(), 1);
        assert_eq!(cursor.read_u8().unwrap(), 0xFE);
        assert_eq!(cursor.remaining(), 0);
        assert!(cursor.read_u8().is_err());
        assert_eq!(cursor.remaining(), 0);
    }

    #[test]
    fn can_read_u16_le() {
        let mut cursor = ReadCursor::new(&[0xCA, 0xFE]);
        assert_eq!(cursor.read_u16_le().unwrap(), 0xFECA);
        assert_eq!(cursor.remaining(), 0);
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
    }

    #[test]
    fn can_read_u64_le() {
        let mut cursor = ReadCursor::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x01]);
        assert_eq!(cursor.read_u64_le().unwrap(), 0x0100FFEEDDCCBBAA);
        assert_eq!(cursor.remaining(), 0);
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
}
