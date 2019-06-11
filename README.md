# pcd-rs: Read point cloud data from **PCD** file format

## Usage

Add pcd-rs to your `Cargo.toml`.

```
pcd_rs = "^0"
```

## Example

```rust
extern crate pcd_rs;
use pcd_rs::ReaderOptions;

fn main() {
    let path = Path::new("/path/to/your.pcd");
    let mut reader = ReaderOptions::from_path(path).unwrap();

    let mut point_count = 0;

    for _ in 0..20 {
        let _point = match reader.read_point().unwrap() {
            Some(point) => point,
            None => break,
        };

        point_count += 1;
    }

    let remaining_points = reader.read_all().unwrap();
    point_count += remaining_points.len() as u64;

    assert!(point_count == reader.meta.num_points);
}
```

## License

MIT
