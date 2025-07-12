# rjprof

- Java profiler written in Rust, similar to [async-profiler](https://github.com/async-profiler/async-profiler).

## Usage

```
cargo clean && RUSTFLAGS="-Awarnings" cargo build --release && RUSTFLAGS="-Awarnings" cargo run --bin rjprof -- \
  --jar examples/HelloApp.jar \
  --agent-path $(pwd)/target/release/librjprof.dylib
```

## Current State

- It "works" for now. Obviously, it's pretty early.


