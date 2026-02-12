# Building from Source

## Prerequisites

- Rust 1.82+
- Protobuf codegen vendored (handled by build.rs)
- System packages for serial support (`libudev` headers + `pkg-config`)

### Automated dependency install (Linux)

Use the helper script to install build dependencies on common distros:

```bash
./scripts/install_build_deps.sh
```

This installs the packages needed for `serialport`/`libudev-sys` (enabled by default features), then you can validate with:

```bash
pkg-config --libs --cflags libudev
```

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

## CI automation

GitHub Actions CI (`.github/workflows/ci.yml`) automatically installs `pkg-config` and `libudev-dev` before running `cargo check` and `cargo test`.
