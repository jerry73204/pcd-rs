# pcd-rs: Read point cloud data from **PCD** file format

`pcd-rs` allows you to parse PCD point cloud data from a file or a binary buffer.

## Usage

Add pcd-rs to your `Cargo.toml`.

```toml
pcd_rs = "^0.6.0"
```

Please visit [docs.rs](https://docs.rs/pcd-rs/) to see detailed usage.

## Examples

### How to run examples

Example programs are available in `examples` directory.
Pleaase run `cargo run --example` to list all available example names.
Then, try the example by `cargo run --example EXAMPLE_NAME`.


### Deserialize a PCD file into a type

```rust
use pcd_rs::{PcdDeserialize, Reader, ReaderBuilder};
use anyhow::Result;

#[derive(PcdDeserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

fn main() -> Result<()> {
    let reader: Reader<Point, _> = ReaderBuilder::from_path("test_files/ascii.pcd")?;
    let points = reader.collect::<Result<Vec<_>>>()?;
    println!("{} points found", points.len());
    Ok(())
}
```

### Deserialize a PCD file dynamically

```rust
use anyhow::Result;
use pcd_rs::{DynRecord, Reader, ReaderBuilder};

fn main() -> Result<()> {
    let reader: Reader<DynRecord, _> = ReaderBuilder::from_path("test_files/binary.pcd")?;
    let points = reader.collect::<Result<Vec<_>>>()?;
    println!("{} points", points.len());
    Ok(())
}
```

### Serialize a type to a PCD file

```rust
use anyhow::Result;
use pcd_rs::{DataKind, PcdDeserialize, PcdSerialize, Writer, WriterBuilder};

#[derive(Debug, PcdDeserialize, PcdSerialize, PartialEq)]
pub struct Point {
    #[pcd_rename("new_x")]
    x: f32,
    y: [u8; 3],
    z: i32,
}

fn main() -> Result<()> {
    // output path
    let path = "test_files/dump_ascii_static.pcd";

    // point data
    let dump_points = vec![
        Point {
            x: 3.14159,
            y: [2, 1, 7],
            z: -5,
        },
        Point {
            x: -0.0,
            y: [254, 6, 98],
            z: 7,
        },
        Point {
            x: 5.6,
            y: [4, 0, 111],
            z: -100000,
        },
    ];

    // serialize points
    let mut writer: Writer<Point, _> =
        WriterBuilder::new(300, 1, Default::default(), DataKind::ASCII)?.create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    Ok(())
}
```

### Serialize points to a PCD file with dynamic schema

```rust
use anyhow::Result;
use pcd_rs::{DataKind, DynRecord, Field, ValueKind, Writer, WriterBuilder};

fn main() -> Result<()> {
    // output path
    let path = "test_files/dump_ascii_untyped.pcd";

    // point data
    let dump_points = vec![
        DynRecord(vec![
            Field::F32(vec![3.14159]),
            Field::U8(vec![2, 1, 7]),
            Field::I32(vec![-5]),
        ]),
        DynRecord(vec![
            Field::F32(vec![-0.0]),
            Field::U8(vec![254, 6, 98]),
            Field::I32(vec![7]),
        ]),
        DynRecord(vec![
            Field::F32(vec![5.6]),
            Field::U8(vec![4, 0, 111]),
            Field::I32(vec![-100000]),
        ]),
    ];

    // serialize points
    let schema = vec![
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::U8, 3),
        ("z", ValueKind::I32, 1),
    ];

    let mut writer: Writer<DynRecord, _> =
        WriterBuilder::new(300, 1, Default::default(), DataKind::ASCII)?
            .schema(schema)?
            .create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    Ok(())
}
```

## License

[MIT](LICENSE) license
