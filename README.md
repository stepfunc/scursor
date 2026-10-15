# scursor

Secure cursor library with support for read and write transactions.

## Panic-free design

`scursor` is designed to be strictly panic-free. This makes it suitable for parsing untrusted input
in security-sensitive contexts, embedded systems with `panic = "abort"`, or anywhere predictable
failure handling is required.

The `ReadCursor` uses a consumption model where each read operation advances an internal position
within a borrowed byte slice. The key insight is that all operations use inherently safe methods.
For example, the core `read_array` routine is implemented as:

```rust,ignore
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

## Panic-freedom checks

When the `no-panic` feature is enabled, every public routine is annotated with
[`no_panic`](https://github.com/dtolnay/no-panic). If the compiled code for a routine still contains a
path that could panic, the check fails at link time. CI runs it with:

```bash
cargo test --profile no-panic --features no-panic --test no_panic
```

What the check does and doesn't tell you:

* Every public method is checked, for every input. `tests/no_panic.rs` takes the address of each
  one, which forces the compiler to keep a standalone copy that has to handle any arguments. That
  copy is what the linker checks. `tests/no_panic_coverage.rs` runs with the normal tests and fails
  if a public method is missing the annotation or isn't listed in `tests/no_panic.rs`.
* CI also links `tests/no_panic_canary.rs`, which contains a function that can panic, and
  requires that link to fail. If a toolchain change ever stopped `no_panic` from detecting
  panics, CI would fail rather than pass silently.
* It applies to the configuration that was checked: an optimized build (`opt-level = 3`) with
  `overflow-checks = true` and `panic = "unwind"`. Debug builds and size-optimized builds keep
  some unreachable panic paths, so the check reports false failures there. Those builds are
  still panic-free; they just can't be checked this way.
* Outside the check:
  * Callbacks passed to `transaction` and `at_pos`. Those routines add no panics of their own,
    but a panic in your callback still propagates.
  * Derived trait impls (`Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`), which can't be annotated.
    Derived `Debug` writes to the formatter you supply.
  * `read_array` is checked for a few sizes (0, 3 and 32). Every size runs the same code.
* The `no-panic` feature exists for this crate's CI. Don't enable it in your own builds.

## License
Licensed under the terms of the MIT or Apache v2 licenses at your choice.
