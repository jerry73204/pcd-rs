//! Tests for I64/U64 (8-byte integer) field type support.

use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};
use tempfile::NamedTempFile;

#[test]
fn test_i64_u64_ascii_round_trip() -> pcd_rs::Result<()> {
    let points = vec![
        DynRecord(vec![Field::I64(vec![i64::MIN]), Field::U64(vec![u64::MAX])]),
        DynRecord(vec![Field::I64(vec![42]), Field::U64(vec![0])]),
    ];

    let schema = Schema::from_iter([
        ("signed", ValueKind::I64, 1),
        ("unsigned", ValueKind::U64, 1),
    ]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer = WriterInit {
            width: 2,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
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
    match (&read_points[0].0[0], &read_points[0].0[1]) {
        (Field::I64(s), Field::U64(u)) => {
            assert_eq!(s[0], i64::MIN);
            assert_eq!(u[0], u64::MAX);
        }
        _ => panic!("Type mismatch"),
    }

    Ok(())
}

#[test]
fn test_i64_u64_binary_round_trip() -> pcd_rs::Result<()> {
    let points = vec![DynRecord(vec![
        Field::I64(vec![-999_999_999_999]),
        Field::U64(vec![12345678901234]),
    ])];

    let schema = Schema::from_iter([("a", ValueKind::I64, 1), ("b", ValueKind::U64, 1)]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
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
    match (&read_points[0].0[0], &read_points[0].0[1]) {
        (Field::I64(a), Field::U64(b)) => {
            assert_eq!(a[0], -999_999_999_999);
            assert_eq!(b[0], 12345678901234);
        }
        _ => panic!("Type mismatch"),
    }

    Ok(())
}

#[test]
fn test_i64_u64_compressed_round_trip() -> pcd_rs::Result<()> {
    let points = vec![
        DynRecord(vec![Field::I64(vec![100]), Field::U64(vec![200])]),
        DynRecord(vec![Field::I64(vec![300]), Field::U64(vec![400])]),
    ];

    let schema = Schema::from_iter([("a", ValueKind::I64, 1), ("b", ValueKind::U64, 1)]);

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
    match (&read_points[1].0[0], &read_points[1].0[1]) {
        (Field::I64(a), Field::U64(b)) => {
            assert_eq!(a[0], 300);
            assert_eq!(b[0], 400);
        }
        _ => panic!("Type mismatch"),
    }

    Ok(())
}

#[test]
fn test_i64_u64_header_parsing() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/i64_u64_ascii.pcd")?;
    let meta = reader.meta();
    assert_eq!(meta.field_defs[0].kind, ValueKind::I64);
    assert_eq!(meta.field_defs[1].kind, ValueKind::U64);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);
    match (&points[0].0[0], &points[0].0[1]) {
        (Field::I64(a), Field::U64(b)) => {
            assert_eq!(a[0], i64::MIN);
            assert_eq!(b[0], u64::MAX);
        }
        _ => panic!("Expected I64 and U64 fields"),
    }

    Ok(())
}
