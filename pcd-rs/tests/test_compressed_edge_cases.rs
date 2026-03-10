//! Edge case tests for binary_compressed format

use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_empty_point_cloud() -> pcd_rs::Result<()> {
    // Test writing and reading an empty compressed point cloud
    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    let file = NamedTempFile::new().unwrap();

    {
        let writer = WriterInit {
            width: 0,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
            version: None,
        }
        .create::<DynRecord, _>(file.path())?;

        writer.finish()?;
    }

    // Read it back
    let reader = DynReader::open(file.path())?;
    assert_eq!(reader.meta().num_points, 0);

    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(read_points.len(), 0);

    Ok(())
}

#[test]
fn test_single_point() -> pcd_rs::Result<()> {
    let points = vec![DynRecord(vec![
        Field::F32(vec![1.0]),
        Field::F32(vec![2.0]),
        Field::F32(vec![3.0]),
    ])];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
            version: None,
        }
        .create::<DynRecord, _>(file.path())?;

        for point in &points {
            writer.push(point)?;
        }

        writer.finish()?;
    }

    let reader = DynReader::open(file.path())?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points.len(), 1);

    match (&points[0].0[0], &read_points[0].0[0]) {
        (Field::F32(o), Field::F32(r)) => assert_eq!(o[0], r[0]),
        _ => panic!("Type mismatch"),
    }

    Ok(())
}

#[test]
fn test_highly_compressible_data() -> pcd_rs::Result<()> {
    // Test with highly repetitive data that should compress well
    let mut points = Vec::new();
    for _ in 0..1000 {
        points.push(DynRecord(vec![
            Field::F32(vec![1.0]),
            Field::F32(vec![1.0]),
            Field::F32(vec![1.0]),
        ]));
    }

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    let compressed_file = NamedTempFile::new().unwrap();
    let uncompressed_file = NamedTempFile::new().unwrap();

    // Write compressed version
    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema.clone()),
            version: None,
        }
        .create(compressed_file.path())?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Write uncompressed version for comparison
    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema),
            version: None,
        }
        .create(uncompressed_file.path())?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    let compressed_size = std::fs::metadata(compressed_file.path())?.len();
    let uncompressed_size = std::fs::metadata(uncompressed_file.path())?.len();

    // Should compress to less than 10% of original size for this repetitive data
    assert!(compressed_size < uncompressed_size / 10);

    // Verify we can read it back correctly
    let reader = DynReader::open(compressed_file.path())?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(read_points.len(), points.len());

    Ok(())
}

#[test]
fn test_mixed_data_types() -> pcd_rs::Result<()> {
    let points = vec![
        DynRecord(vec![
            Field::F32(vec![1.0]),
            Field::F64(vec![2.0]),
            Field::U8(vec![255]),
            Field::I32(vec![-42]),
        ]),
        DynRecord(vec![
            Field::F32(vec![3.25]),
            Field::F64(vec![2.75]),
            Field::U8(vec![128]),
            Field::I32(vec![12345]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F64, 1),
        ("intensity", ValueKind::U8, 1),
        ("label", ValueKind::I32, 1),
    ]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
            version: None,
        }
        .create::<DynRecord, _>(file.path())?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    let reader = DynReader::open(file.path())?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points.len(), points.len());

    for (orig, read) in points.iter().zip(read_points.iter()) {
        for (orig_field, read_field) in orig.0.iter().zip(read.0.iter()) {
            match (orig_field, read_field) {
                (Field::F32(o), Field::F32(r)) => assert_eq!(o[0], r[0]),
                (Field::F64(o), Field::F64(r)) => assert_eq!(o[0], r[0]),
                (Field::U8(o), Field::U8(r)) => assert_eq!(o[0], r[0]),
                (Field::I32(o), Field::I32(r)) => assert_eq!(o[0], r[0]),
                _ => panic!("Type mismatch"),
            }
        }
    }

    Ok(())
}

#[test]
fn test_large_point_cloud() -> pcd_rs::Result<()> {
    let mut points = Vec::new();
    for i in 0..10000 {
        let x = (i as f32) * 0.1;
        let y = ((i as f32) * 0.2).sin();
        let z = ((i as f32) * 0.3).cos();

        points.push(DynRecord(vec![
            Field::F32(vec![x]),
            Field::F32(vec![y]),
            Field::F32(vec![z]),
        ]));
    }

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
            version: None,
        }
        .create::<DynRecord, _>(file.path())?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    let reader = DynReader::open(file.path())?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points.len(), points.len());

    for i in [0, 1000, 5000, 9999] {
        match (&points[i].0[0], &read_points[i].0[0]) {
            (Field::F32(o), Field::F32(r)) => {
                assert!((o[0] - r[0]).abs() < 1e-6, "Mismatch at point {}", i);
            }
            _ => panic!("Type mismatch"),
        }
    }

    Ok(())
}

#[test]
fn test_corrupt_compressed_file() {
    let mut file = NamedTempFile::new().unwrap();

    writeln!(file, "# .PCD v.7 - Point Cloud Data file format").unwrap();
    writeln!(file, "VERSION .7").unwrap();
    writeln!(file, "FIELDS x y z").unwrap();
    writeln!(file, "SIZE 4 4 4").unwrap();
    writeln!(file, "TYPE F F F").unwrap();
    writeln!(file, "COUNT 1 1 1").unwrap();
    writeln!(file, "WIDTH 3").unwrap();
    writeln!(file, "HEIGHT 1").unwrap();
    writeln!(file, "VIEWPOINT 0 0 0 1 0 0 0").unwrap();
    writeln!(file, "POINTS 3").unwrap();
    writeln!(file, "DATA binary_compressed").unwrap();

    // Write invalid compressed data (sizes don't match actual data)
    file.write_all(&100u32.to_le_bytes()).unwrap(); // compressed size
    file.write_all(&200u32.to_le_bytes()).unwrap(); // uncompressed size
    file.write_all(b"invalid data").unwrap(); // Not enough data

    let result = DynReader::open(file.path());
    assert!(
        result.is_err(),
        "Should fail to read corrupted compressed file"
    );
}
