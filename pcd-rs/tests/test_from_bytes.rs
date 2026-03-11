//! Tests for reading PCD from in-memory byte slices.

use pcd_rs::{DynReader, DynRecord, Field};

#[test]
fn test_dyn_reader_from_bytes_ascii() -> pcd_rs::Result<()> {
    let pcd = std::fs::read("test_files/xyz_2pts.pcd").unwrap();
    let reader = DynReader::from_bytes(&pcd)?;
    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(points.len(), 2);
    match &points[0].0[0] {
        Field::F32(v) => assert_eq!(v[0], 1.0),
        _ => panic!("Expected F32"),
    }

    Ok(())
}

#[test]
fn test_dyn_reader_from_bytes_binary() -> pcd_rs::Result<()> {
    // Write a binary PCD to bytes, then read it back
    use pcd_rs::{DataKind, DynWriter, Schema, ValueKind, WriterInit};
    use std::io::Cursor;

    let schema = Schema::from_iter([("x", ValueKind::F32, 1)]);

    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);

    {
        let mut writer: DynWriter<_> = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema),
            version: None,
        }
        .build_from_writer(cursor)?;

        writer.push(&DynRecord(vec![Field::F32(vec![42.0])]))?;
        writer.finish()?;
    }

    let reader = DynReader::from_bytes(&buf)?;
    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(points.len(), 1);
    match &points[0].0[0] {
        Field::F32(v) => assert_eq!(v[0], 42.0),
        _ => panic!("Expected F32"),
    }

    Ok(())
}
