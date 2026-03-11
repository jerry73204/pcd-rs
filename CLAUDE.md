# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

pcd-rs is a Rust library for reading and writing PCD (Point Cloud Data) file format. It consists of two crates:
- `pcd-rs`: Main library for PCD file parsing and writing
- `pcd-rs-derive`: Procedural macros for deriving PcdSerialize and PcdDeserialize traits

## Common Development Commands

Use `just` to run common tasks:

```bash
just build     # Build all targets with dev-release profile
just format    # Format code with nightly rustfmt
just check     # Check formatting and run clippy
just test      # Run tests with nextest
just ci        # Run check + test (used in CI)
just clean     # Remove build artifacts
```

### Running specific tests
```bash
cargo nextest run --cargo-profile dev-release --all-features test_name       # Run specific test by name
cargo nextest run --cargo-profile dev-release --all-features --test test_file # Run specific test file
```

### Running examples
```bash
cargo run --example read_dynamic                        # Dynamic reader example
cargo run --example write_dynamic                       # Dynamic writer example
cargo run --example read_static --features derive       # Static reader (requires derive feature)
cargo run --example write_static --features derive      # Static writer (requires derive feature)
```

## Architecture

### Core Components

**pcd-rs crate:**
- `reader.rs`: `DynReader` (dynamic schema) and `Reader` (static schema with derive feature)
- `writer.rs`: `DynWriter` and `Writer` implementations with `WriterInit` builder
- `record.rs`: `DynRecord` and `Field` types for dynamic point representation
- `metas.rs`: PCD metadata structures (`Schema`, `ValueKind`, `DataKind`, etc.)
- `rgb.rs`: `Rgb`/`Rgba` wrapper types and PCL-style packed float conversion helpers
- `lzf.rs`: LZF compression/decompression for `binary_compressed` format
- `traits.rs`: Core traits like `PcdSerialize`, `PcdDeserialize`, and `Value`
- `error.rs`: Error types and result aliases

**pcd-rs-derive crate:**
- Procedural macros for `PcdSerialize` and `PcdDeserialize` derive implementations
- Supports field attributes: `#[pcd(rename = "...")]` and `#[pcd(ignore)]`
- Supports types: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `Rgb`, `Rgba`

### Key Design Patterns

1. **Dual API Design**: Library provides both dynamic (any schema) and static (derive-based) APIs
2. **Builder Pattern**: `WriterInit` configures and creates writers
3. **Iterator Pattern**: Readers implement Iterator trait for streaming point data
4. **Type Safety**: Static API uses derive macros for compile-time schema validation
5. **PCD Version Support**: Supports v0.5, v0.6, and v0.7 formats (binary_compressed is v0.7 only)

### Test Files Location
- Test PCD fixture files are in `pcd-rs/test_files/`
- Tests and doctests run from the crate directory (`pcd-rs/`), so paths use `"test_files/..."`
- Examples run from the workspace root, so paths use `"pcd-rs/test_files/..."`

### CI/CD
- GitHub Actions workflow runs `just ci` on pushes to master and PRs
- Publishing is triggered by `pcd-rs@x.y.z` or `pcd-rs-derive@x.y.z` tags
- Requires `CARGO_REGISTRY_TOKEN` secret for crates.io publication
