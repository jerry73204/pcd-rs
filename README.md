# pcd-rs: Read point cloud data from **PCD** file format

`pcd-rs` allows you to parse PCD point cloud data from a file,
a path, or a binary buffer. The reader implements `Iterator` to
let you iterate over points with ease.

## Usage

Add pcd-rs to your `Cargo.toml`.

```toml
pcd_rs = "~0"
```

## Example

```rust
use failure::Fallible;
use pcd_rs::SeqReaderOptions;
use std::path::Path;

fn main() -> Fallible<()> {
    let path = Path::new("test_files/ascii.pcd");
    let reader = SeqReaderOptions::from_path(path)?;

    // Get meta data
    let meta = reader.meta();

    // Scan all points
    let points = reader.collect::<Fallible<Vec<_>>>()?;

    Ok(())
}
```

## License

[MIT](LICENSE)
