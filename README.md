# pcd-rs: Read point cloud data from **PCD** file format

`pcd-rs` allows you to parse PCD point cloud data from a file or a binary buffer

## Usage

Add pcd-rs to your `Cargo.toml`.

```toml
pcd_rs = "*"
```

Checkout [docs.rs](https://docs.rs/pcd-rs/) to see detailed usage.

## Examples

### Load PCD file into structs

```rust
use failure::Fallible;
use pcd_rs::{prelude::*, seq_reader::SeqReaderBuilder, PCDRecordRead};

#[derive(PCDRecordRead)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

fn main() -> Fallible<()> {
    let reader = SeqReaderBuilder::<Point, _>::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}
```

### Load PCD file into untyped records

```rust
use failure::Fallible;
use pcd_rs::{
    prelude::*,
    record::{Field, UntypedRecord},
    seq_reader::SeqReaderBuilder,
};

fn main() -> Fallible<()> {
    let reader = SeqReaderBuilder::<UntypedRecord, _>::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;

    for point in points.iter() {
        for field in point.0.iter() {
            match field {
                Field::I8(values) => {
                    println!("i8 values: {:?}", values);
                }
                Field::U8(values) => {
                    println!("u8 values: {:?}", values);
                }
                Field::F32(values) => {
                    println!("f32 values: {:?}", values);
                }
                _ => {
                    println!("other kinds of values");
                }
            }
        }
    }

    Ok(())
}
```

### Write struct to PCD file

```rust
use failure::Fallible;
use pcd_rs::{meta::DataKind, prelude::*, seq_writer::SeqWriterBuilder, PCDRecordWrite};

#[derive(PCDRecordWrite)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Fallible<()> {
    let viewpoint = Default::default();
    let kind = DataKind::ASCII;
    let mut writer = SeqWriterBuilder::<Point, _>::new(300, 1, viewpoint, kind)?
        .create("test_files/dump.pcd")?;

    let point = Point {
        x: 3.14159,
        y: 2.71828,
        z: -5.0,
    };

    writer.push(&point)?;
    writer.finish()?;

    Ok(())
}
```

### Write untyped record to PCD file

```rust
use failure::Fallible;
use pcd_rs::{
    meta::{DataKind, ValueKind},
    prelude::*,
    record::{Field, UntypedRecord},
    seq_writer::SeqWriterBuilder,
};

fn main() -> Fallible<()> {
    let viewpoint = Default::default();
    let kind = DataKind::ASCII;
    let schema = vec![
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ];
    let mut writer = SeqWriterBuilder::<UntypedRecord, _>::new(300, 1, viewpoint, kind, schema)?
        .create("test_files/dump.pcd")?;

    let point = UntypedRecord(vec![
        Field::F32(vec![3.14159]),
        Field::F32(vec![2.71828]),
        Field::F32(vec![-5.0]),
    ]);

    writer.push(&point)?;
    writer.finish()?;

    Ok(())
}
```

## License

[MIT](LICENSE)
