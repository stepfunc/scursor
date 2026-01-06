# scursor

![CI](https://github.com/stepfunc/oo_bindgen/workflows/CI/badge.svg)

Secure cursor library with support for read and write transactions.

## Panic-free design

`scursor` is designed to be strictly panic-free. This makes it suitable for parsing untrusted input
in security-sensitive contexts, embedded systems with `panic = "abort"`, or anywhere predictable
failure handling is required.

The `ReadCursor` uses a consumption model where each read operation advances an internal position
within a borrowed byte slice. The key insight is that all operations use inherently safe methods:

```rust
pub fn read_u8(&mut self) -> Result<u8, ReadError> {
    match self.input.get(self.pos) {          // .get() returns Option, never panics
        Some(x) => {
            let pos = self.pos.checked_add(1) // checked_add() returns Option on overflow
                .ok_or(ReadError)?;
            self.pos = pos;
            Ok(*x)
        }
        None => Err(ReadError),
    }
}
```

Larger types are composed from smaller reads. For example, `read_u32_le()` performs two `read_u16_le()`
calls, which each perform two `read_u8()` calls. This hierarchical approach means panic-freedom is
established at the leaf operations and preserved through composition.

There are no direct slice indexing operations (`slice[i]`), no `.unwrap()` or `.expect()` calls,
and no arithmetic that could overflow. Every failure path returns a `Result`.

## License
Licensed under the terms of the MIT or Apache v2 licenses at your choice.

