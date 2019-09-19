# pcd-rs: Read point cloud data from **PCD** file format

`pcd-rs` allows you to parse PCD point cloud data from a file or a binary buffer

## Usage

Add pcd-rs to your `Cargo.toml`.

```toml
pcd_rs = "*"
```

Checkout [docs.rs](https://docs.rs/pcd-rs/) to see detailed usage.

## Examples

### Load PCD file into typed records

```rust
use failure::Fallible;
use pcd_rs::{seq_reader::SeqReaderBuilder, PCDRecordRead};
use std::path::Path;

#[derive(PCDRecordRead)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
    rgb: f32,
}

fn main() -> Fallible<()> {
    let reader = SeqReaderBuilder::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<Point>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}
```

### Load PCD file into untyped records

```rust
use failure::Fallible;
use pcd_rs::{seq_reader::SeqReaderBuilder, PCDRecordRead, record::{Record, Field}};
use std::path::Path;

fn main() -> Fallible<()> {
    let reader = SeqReaderBuilder::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<Record>>>()?;

    for point in points.iter() {
        for field in point.iter() {
            match field {
                Field::I8(values) => {...},
                Field::U8(values) => {...},
                Field::F32(values) => {...},
                ...
            }
        }
    }

    assert_eq!(points.len(), 213);
    Ok(())
}
```

### Write to PCD file

```rust
use failure::Fallible;
use pcd_rs::{DataKind, seq_writer::SeqWriterBuilder, PCDRecordWrite};
use std::path::Path;

#[derive(PCDRecordWrite)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Fallible<()> {
    let viewpoint = Default::default();
    let kind = DataKind::ASCII;
    let mut writer = SeqWriterBuilder::<Point>::new(300, 1, viewpoint, kind)?
        .create("test_files/dump.pcd")?;

    let point = Point {
        x: 3.14159,
        y: 2.71828,
        z: -5.0,
    };

    writer.push(&point)?;

    Ok(())
}
```

## License

[MIT](LICENSE)
