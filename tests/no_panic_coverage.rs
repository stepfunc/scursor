//! Ensures every public routine is annotated with `#[no_panic]` and exercised by
//! `tests/no_panic.rs`. Functions that are never called are not checked by the
//! linker, so a missing call would silently skip the check.

const DRIVER: &str = include_str!("no_panic.rs");
const ATTRIBUTE: &str = "#[cfg_attr(feature = \"no-panic\", no_panic)]";

/// Names of all `pub fn` items in `source`, each paired with whether it is
/// immediately preceded by the no_panic attribute.
fn public_fns(source: &str) -> Vec<(String, bool)> {
    let lines: Vec<&str> = source.lines().map(str::trim).collect();
    lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| {
            let rest = line.strip_prefix("pub fn ")?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let annotated = i > 0 && lines[i - 1] == ATTRIBUTE;
            Some((name, annotated))
        })
        .collect()
}

/// The body of the driver function `name`, up to the next driver function.
fn driver_body(name: &str) -> &'static str {
    let start = DRIVER
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("driver function {name} not found"));
    let body = &DRIVER[start..];
    let end = body[1..].find("\nfn ").map_or(body.len(), |i| i + 1);
    &body[..end]
}

fn check(source: &str, driver_fn: &str, type_name: &str) {
    let body = driver_body(driver_fn);
    let fns = public_fns(source);
    assert!(!fns.is_empty(), "no public functions found for {type_name}");

    for (name, annotated) in fns {
        assert!(
            annotated,
            "{type_name}::{name} is missing the no_panic attribute"
        );
        let called = body.contains(&format!(".{name}("))
            || body.contains(&format!(".{name}::<"))
            || body.contains(&format!("{type_name}::{name}("));
        assert!(
            called,
            "{type_name}::{name} is not exercised by {driver_fn} in tests/no_panic.rs"
        );
    }
}

#[test]
fn every_read_cursor_routine_is_checked() {
    check(
        include_str!("../src/read.rs"),
        "exercise_read_cursor",
        "ReadCursor",
    );
}

#[test]
fn every_write_cursor_routine_is_checked() {
    check(
        include_str!("../src/write.rs"),
        "exercise_write_cursor",
        "WriteCursor",
    );
}
