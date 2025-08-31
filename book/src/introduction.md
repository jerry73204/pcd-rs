# Introduction

Welcome to the pcd-rs developer guide! This book provides comprehensive documentation for the pcd-rs library, a pure Rust implementation for reading and writing Point Cloud Data (PCD) files.

## What is pcd-rs?

pcd-rs is a Rust library that enables you to work with PCD files, the primary file format used by the Point Cloud Library (PCL) for storing 3D point cloud data. Whether you're working with LiDAR data, 3D scanning results, or computer vision applications, pcd-rs provides the tools you need to efficiently parse and generate PCD files.

## Key Features

- **Pure Rust Implementation**: No external dependencies on PCL or other C++ libraries
- **Dual API Design**: Choose between dynamic (runtime) or static (compile-time) schemas
- **Zero-Copy Parsing**: Efficient binary data handling with minimal allocations
- **Type Safety**: Leverage Rust's type system with derive macros for compile-time validation
- **Format Support**: Handles ASCII, binary, and compressed binary PCD formats
- **Streaming API**: Iterator-based design for memory-efficient processing of large point clouds

## Who Should Read This Book?

This book is intended for:

- **Rust developers** working with 3D point cloud data
- **Robotics engineers** needing to process sensor data
- **Computer vision researchers** handling 3D datasets
- **Contributors** looking to understand and improve the pcd-rs codebase

## How This Book is Organized

The book is divided into several sections:

1. **User Guide**: Learn how to use pcd-rs in your projects
2. **Architecture**: Understand the library's design and structure
3. **Design Documentation**: Deep dive into implementation details
4. **PCD Format Reference**: Complete specification of the PCD file format
5. **Development**: Contributing guidelines and development workflow
6. **Roadmap**: Current status and future plans

## Quick Example

Here's a simple example to get you started:

```rust
use pcd_rs::{DynReader, DynWriter, WriterInit, DataKind};

// Read a PCD file
let reader = DynReader::open("input.pcd")?;
let points: Vec<_> = reader.collect::<Result<_, _>>()?;

// Write a PCD file
let mut writer = WriterInit {
    width: points.len() as u64,
    height: 1,
    viewpoint: Default::default(),
    data_kind: DataKind::Binary,
    schema: None,
}
.create("output.pcd")?;

for point in points {
    writer.push(&point)?;
}
writer.finish()?;
```

## Getting Help

- **API Documentation**: [docs.rs/pcd-rs](https://docs.rs/pcd-rs/)
- **GitHub Repository**: [github.com/jerry73204/pcd-rs](https://github.com/jerry73204/pcd-rs)
- **Issues & Feature Requests**: [GitHub Issues](https://github.com/jerry73204/pcd-rs/issues)

Let's get started with [Getting Started](./getting_started.md)!