//! Tests for binary_compressed SoA (column-major) layout correctness.

use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};
use tempfile::NamedTempFile;

#[test]
fn test_compressed_soa_round_trip() -> pcd_rs::Result<()> {
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
    for (orig, read) in points.iter().zip(read_points.iter()) {
        for (of, rf) in orig.0.iter().zip(read.0.iter()) {
            match (of, rf) {
                (Field::F32(o), Field::F32(r)) => assert_eq!(o[0], r[0]),
                _ => panic!("Type mismatch"),
            }
        }
    }

    Ok(())
}

#[test]
fn test_compressed_soa_single_point() -> pcd_rs::Result<()> {
    // Edge case: single point
    let schema = Schema::from_iter([("x", ValueKind::F32, 1), ("y", ValueKind::F32, 1)]);

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

        writer.push(&DynRecord(vec![
            Field::F32(vec![42.0]),
            Field::F32(vec![99.0]),
        ]))?;
        writer.finish()?;
    }

    let reader = DynReader::open(file.path())?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points.len(), 1);
    match (&read_points[0].0[0], &read_points[0].0[1]) {
        (Field::F32(x), Field::F32(y)) => {
            assert_eq!(x[0], 42.0);
            assert_eq!(y[0], 99.0);
        }
        _ => panic!("Type mismatch"),
    }

    Ok(())
}

#[test]
fn test_compressed_soa_mixed_types() -> pcd_rs::Result<()> {
    let points = vec![
        DynRecord(vec![
            Field::F32(vec![1.5]),
            Field::U8(vec![10]),
            Field::I32(vec![-100]),
        ]),
        DynRecord(vec![
            Field::F32(vec![2.5]),
            Field::U8(vec![20]),
            Field::I32(vec![200]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("intensity", ValueKind::U8, 1),
        ("label", ValueKind::I32, 1),
    ]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer = WriterInit {
            width: 2,
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

    assert_eq!(read_points.len(), 2);
    match (
        &read_points[0].0[0],
        &read_points[0].0[1],
        &read_points[0].0[2],
    ) {
        (Field::F32(x), Field::U8(i), Field::I32(l)) => {
            assert_eq!(x[0], 1.5);
            assert_eq!(i[0], 10);
            assert_eq!(l[0], -100);
        }
        _ => panic!("Type mismatch"),
    }

    Ok(())
}
