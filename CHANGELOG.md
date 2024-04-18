### 0.2.0 ###
* Specify lints in Cargo.toml instead of lib.rs.
* :star: Add method to `ReadCursor` to retrieve the position.
* :star: Argument to read/write transaction are now `FnOnce` per [#1](https://github.com/stepfunc/scursor/issues/1).