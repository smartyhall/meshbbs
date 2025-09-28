# Building from Source

## Prerequisites

- Rust 1.82+
- Protobuf codegen vendored (handled by build.rs)

## Build

```bash
cargo build --release
```

## Tests

```bash
cargo test
```

## Docs

```bash
cargo doc --no-deps --all-features
open target/doc/meshbbs/index.html
```
