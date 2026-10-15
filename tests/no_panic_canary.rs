//! Canary for the link-time panic check: this test must fail to link.
//!
//! `canary` panics when its argument is zero. If the `no_panic` check is
//! working, linking this test fails with "detected panic in function `canary`",
//! and CI requires that failure. If this test ever links, the check has stopped
//! detecting panics, and a passing `tests/no_panic.rs` would mean nothing.
//!
//! ```text
//! cargo test --profile no-panic --features no-panic --test no_panic_canary
//! ```

use no_panic::no_panic;
use std::hint::black_box;

#[no_panic]
fn canary(divisor: u8) -> u8 {
    100 / divisor
}

#[test]
fn canary_must_not_link() {
    black_box::<fn(_) -> _>(canary);
}
