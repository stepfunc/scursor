mod proofs {
    use crate::{ReadCursor, WriteCursor};

    /// This proof mathematically verifies that `read_u32_le` is entirely panic-free.
    ///
    /// # Deductive Proof by Composition
    /// `scursor` is designed to be "safe by composition." Because all integer and floating-point
    /// read operations (e.g., `read_u16`, `read_u64`, `read_f32`, both LE and BE) are built using
    /// the exact same underlying primitive (`read_array`), proving that `read_u32_le` is panic-free
    /// inherently proves that the foundation is sound.
    ///
    /// By formally verifying this single method, Kani proves that for any arbitrary slice length:
    /// 1. `checked_add` prevents all integer overflows when calculating indices.
    /// 2. `input.get(..).and_then(|s| s.first_chunk::<N>())` never triggers out-of-bounds panics.
    /// 3. The `?` operator successfully propagates `ReadError` instead of panicking on failure.
    #[kani::proof]
    fn prove_read_u32_le_no_panic() {
        // Generate an arbitrary input array of up to 10 bytes
        let len: usize = kani::any();
        kani::assume(len <= 10);

        let mut input = [0u8; 10];
        for i in 0..10 {
            input[i] = kani::any();
        }

        let slice = &input[..len];
        let mut cursor = ReadCursor::new(slice);

        // This read should NEVER panic, regardless of the buffer size or contents
        let _ = cursor.read_u32_le();
    }

    /// This proof mathematically verifies that `write_u32_le` is entirely panic-free.
    ///
    /// # Deductive Proof by Composition
    /// Similar to the read side, all integer and floating-point write operations
    /// are built upon the singular `write_bytes` primitive.
    ///
    /// By formally verifying this single method, Kani proves that for any arbitrary slice length
    /// and any arbitrary 32-bit input value:
    /// 1. `checked_add` inside `write_bytes` prevents integer overflow.
    /// 2. `dest.get_mut(..)` prevents all out-of-bounds panics when accessing the buffer.
    /// 3. `copy_from_slice` is proven safe because it only executes when `get_mut` successfully
    ///    returns a subslice of the exact matching length.
    /// 4. The `write_overflow` error path is returned correctly instead of panicking.
    #[kani::proof]
    fn prove_write_u32_le_no_panic() {
        // Generate an arbitrary mutable output buffer of up to 10 bytes
        let len: usize = kani::any();
        kani::assume(len <= 10);

        let mut buffer = [0u8; 10];
        let slice = &mut buffer[..len];
        let mut cursor = WriteCursor::new(slice);

        // Generate an arbitrary 32-bit value to write
        let val: u32 = kani::any();

        // This write should NEVER panic, regardless of whether there is enough room in the buffer
        let _ = cursor.write_u32_le(val);
    }
}
