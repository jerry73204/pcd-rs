# Getting Started

This chapter will help you get started with pcd-rs in your Rust project.

## Installation

Add pcd-rs to your `Cargo.toml`:

```toml
[dependencies]
pcd-rs = "0.12"
```

To use the derive macros for static typing:

```toml
[dependencies]
pcd-rs = { version = "0.12", features = ["derive"] }
```

## Basic Concepts

pcd-rs provides two main APIs for working with PCD files:

### Dynamic API

The dynamic API works with any PCD schema at runtime:

- `DynReader`: Reads PCD files with runtime schema discovery
- `DynWriter`: Writes PCD files with runtime-defined schemas
- `DynRecord`: Represents a point with dynamic fields
- `Field`: Represents field data (F32, U8, I32, etc.)

### Static API (with `derive` feature)

The static API uses Rust's type system for compile-time validation:

- `Reader<T>`: Reads PCD files into structs implementing `PcdDeserialize`
- `Writer<T>`: Writes structs implementing `PcdSerialize` to PCD files
- Derive macros: `#[derive(PcdSerialize, PcdDeserialize)]`

## Your First PCD Reader

Here's a simple example reading a PCD file with dynamic schema:

```rust
use pcd_rs::{DynReader, DynRecord};

fn main() -> pcd_rs::Result<()> {
    // Open a PCD file
    let reader = DynReader::open("pointcloud.pcd")?;
    
    // Get metadata
    let meta = reader.meta();
    println!("Points: {}", meta.num_points());
    println!("Width: {}, Height: {}", meta.width, meta.height);
    
    // Iterate through points
    for (idx, point) in reader.enumerate() {
        let point = point?;
        println!("Point {}: {:?}", idx, point);
        if idx >= 10 { break; } // Just show first 10 points
    }
    
    Ok(())
}
```

## Your First PCD Writer

Creating a PCD file with known schema:

```rust
use pcd_rs::{DynWriter, DynRecord, Field, WriterInit, DataKind, Schema, ValueKind};

fn main() -> pcd_rs::Result<()> {
    // Define the schema
    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);
    
    // Create writer
    let mut writer = WriterInit {
        width: 3,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Ascii,
        schema: Some(schema),
    }
    .create("output.pcd")?;
    
    // Write points
    let points = vec![
        DynRecord(vec![
            Field::F32(vec![1.0]),
            Field::F32(vec![2.0]),
            Field::F32(vec![3.0]),
        ]),
        DynRecord(vec![
            Field::F32(vec![4.0]),
            Field::F32(vec![5.0]),
            Field::F32(vec![6.0]),
        ]),
        DynRecord(vec![
            Field::F32(vec![7.0]),
            Field::F32(vec![8.0]),
            Field::F32(vec![9.0]),
        ]),
    ];
    
    for point in points {
        writer.push(&point)?;
    }
    
    writer.finish()?;
    Ok(())
}
```

## Using Static Types with Derive

For better type safety and performance, use the derive macros:

```rust
use pcd_rs::{Reader, Writer, WriterInit, DataKind, PcdSerialize, PcdDeserialize};

#[derive(Debug, PcdSerialize, PcdDeserialize)]
struct PointXYZRGB {
    x: f32,
    y: f32,
    z: f32,
    rgb: f32,
}

fn main() -> pcd_rs::Result<()> {
    // Read typed points
    let reader: Reader<PointXYZRGB> = Reader::open("pointcloud.pcd")?;
    let points: Vec<PointXYZRGB> = reader.collect::<Result<_, _>>()?;
    
    // Process points
    for point in &points {
        println!("Point at ({}, {}, {})", point.x, point.y, point.z);
    }
    
    // Write typed points
    let mut writer = WriterInit {
        width: points.len() as u64,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: None, // Schema inferred from type
    }
    .create("output.pcd")?;
    
    for point in points {
        writer.push(&point)?;
    }
    
    writer.finish()?;
    Ok(())
}
```

## Next Steps

Now that you understand the basics, explore:

- [Reading PCD Files](./reading_pcd.md) for advanced reading techniques
- [Writing PCD Files](./writing_pcd.md) for output options
- [Static vs Dynamic API](./static_vs_dynamic.md) to choose the right approach