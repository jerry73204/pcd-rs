# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

pcd-rs is a Rust library for reading and writing PCD (Point Cloud Data) file format. It consists of two crates:
- `pcd-rs`: Main library for PCD file parsing and writing
- `pcd-rs-derive`: Procedural macros for deriving PcdSerialize and PcdDeserialize traits

## Common Development Commands

### Building
```bash
cargo build                    # Build all workspace members
cargo build --all-features     # Build with all features including derive
cargo build -p pcd-rs          # Build only pcd-rs crate
cargo build -p pcd-rs-derive   # Build only pcd-rs-derive crate
```

### Testing
```bash
cargo test                              # Run all tests in workspace
cargo test -p pcd-rs                    # Test only pcd-rs crate
cargo test -p pcd-rs-derive             # Test only pcd-rs-derive crate
cargo test --all-features               # Test with all features enabled
cargo test test_name                    # Run specific test by name
cargo test --test test_file_name        # Run specific test file
```

### Code Quality
```bash
cargo fmt                      # Format code using rustfmt
cargo fmt --check              # Check formatting without modifying
cargo clippy                   # Run clippy linter
cargo clippy --all-features    # Run clippy with all features
```

### Examples
```bash
cargo run --example read_dynamic         # Run dynamic reader example
cargo run --example write_dynamic        # Run dynamic writer example
cargo run --example read_static --features derive   # Run static reader (requires derive feature)
cargo run --example write_static --features derive  # Run static writer (requires derive feature)
```

## Architecture

### Core Components

**pcd-rs crate:**
- `reader.rs`: Contains `DynReader` (dynamic schema) and `Reader` (static schema with derive feature)
- `writer.rs`: Contains `DynWriter` and `Writer` implementations with `WriterInit` builder
- `record.rs`: Defines `DynRecord` and `Field` types for dynamic point representation
- `metas.rs`: PCD metadata structures (`Schema`, `ValueKind`, `DataKind`, etc.)
- `traits.rs`: Core traits like `PcdSerialize`, `PcdDeserialize`, and `Value`
- `error.rs`: Error types and result aliases

**pcd-rs-derive crate:**
- Procedural macros for `PcdSerialize` and `PcdDeserialize` derive implementations
- Supports field attributes: `#[pcd(rename = "...")]` and `#[pcd(ignore)]`

### Key Design Patterns

1. **Dual API Design**: Library provides both dynamic (any schema) and static (derive-based) APIs
2. **Builder Pattern**: `WriterInit` configures and creates writers
3. **Iterator Pattern**: Readers implement Iterator trait for streaming point data
4. **Type Safety**: Static API uses derive macros for compile-time schema validation

### Test Files Location
- Test PCD files are located in `test_files/` directory
- Tests read from and write to this directory for validation