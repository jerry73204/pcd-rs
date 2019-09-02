# pcd-rs: Read point cloud data from **PCD** file format

`pcd-rs` allows you to parse PCD point cloud data from a file or
a binary buffer. The reader implements `Iterator` to
let you iterate over points with ease.

## Usage

Add pcd-rs to your `Cargo.toml`.

```toml
pcd_rs = "*"
```

## Example

```rust
use failure::Fallible;
use pcd_rs::{PCDRecord, SeqReaderBuilder};
use std::path::Path;

#[derive(PCDRecord)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
    timestamp: u32,
}

#[test]
fn load_binary() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader = SeqReaderBuilder::open_path(path)?;
    let points = reader.collect::<Fallible<Vec<Point>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
```

You may visit [tests directory](tests) for more examples.

## License

[MIT](LICENSE)
