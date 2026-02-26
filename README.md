# scursor

![CI](https://github.com/stepfunc/scursor/workflows/CI/badge.svg)

Secure cursor library with support for read and write transactions.

## Panic-free design

`scursor` is designed to be strictly panic-free. This makes it suitable for parsing untrusted input
in security-sensitive contexts, embedded systems with `panic = "abort"`, or anywhere predictable
failure handling is required.

The `ReadCursor` uses a consumption model where each read operation advances an internal position
within a borrowed byte slice. The key insight is that all operations use inherently safe methods.
For example, the core `read_array` routine is implemented as:

```rust
pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], ReadError> {
    let chunk = self.input.get(self.pos..)
        .and_then(|s| s.first_chunk::<N>()) // non-panicking bounds check
        .ok_or(ReadError)?;
    
    self.pos = self.pos.checked_add(N).ok_or(ReadError)?; // overflow-safe arithmetic
    Ok(*chunk)
}
```

Higher-level routines are composed from these primitives. There are no direct slice indexing 
operations (`slice[i]`), no `.unwrap()` or `.expect()` calls, and no arithmetic that could overflow. 
Every failure path returns a `Result`.

### Safety by Composition

The core philosophy of `scursor` is that complexity should be built from verified, safe foundations.
You can build complex, multi-step parsers that inherit the library's panic-free guarantees:

```rust
use scursor::{ReadCursor, ReadError};

struct Packet {
    id: u32,
    payload: [u8; 4],
    checksum: u16,
}

fn parse_packet(cursor: &mut ReadCursor) -> Result<Packet, ReadError> {
    // All of these calls are panic-free and bounds-checked
    Ok(Packet {
        id: cursor.read_u32_le()?,
        payload: cursor.read_array()?,
        checksum: cursor.read_u16_le()?,
    })
}

fn main() {
    let data = [0x01, 0x02, 0x03, 0x04, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let mut cursor = ReadCursor::new(&data);
    
    // The transaction API rolls back the cursor if any part of the parser fails
    let packet = cursor.transaction(|cur| parse_packet(cur));
}
```

## Formal Verification

This library is formally verified to be panic-free using the [Kani Rust Verifier](https://model-checking.github.io/kani/). 
Kani uses bit-precise model checking to mathematically prove the absence of panics, out-of-bounds 
accesses, and overflows across all possible execution paths and inputs.

To run the mathematical proofs yourself:

1. **Install Kani**:
   ```bash
   cargo install --locked kani-verifier
   cargo kani setup
   ```

2. **Run Verification**:
   ```bash
   cargo kani
   ```

## License
Licensed under the terms of the MIT or Apache v2 licenses at your choice.
