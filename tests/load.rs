use std::path::Path;
use pcd_rs::ReaderOptions;

#[test]
fn load_ascii() {
    let path = Path::new("test_files/ascii.pcd");
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

#[test]
fn load_binary() {
    let path = Path::new("test_files/binary.pcd");
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
