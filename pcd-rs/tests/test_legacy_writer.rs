use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};
use std::io::Cursor;

#[test]
fn test_write_pcd_v05() -> pcd_rs::Result<()> {
    // Create test data
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
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    // Write to buffer with v0.5
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 2,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema),
            version: Some("0.5".to_string()),
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Read back and verify
    let data = buffer.into_inner();
    let content = String::from_utf8_lossy(&data);

    // Check that VERSION is .5
    assert!(content.contains("VERSION .5"), "Should contain VERSION .5");

    // Check that VIEWPOINT is NOT present (v0.5 doesn't have it)
    assert!(
        !content.contains("VIEWPOINT"),
        "v0.5 should not contain VIEWPOINT"
    );

    // Check header comment
    assert!(
        content.contains("# .PCD v.5"),
        "Should contain v.5 header comment"
    );

    // Verify we can read it back
    let reader = DynReader::from_reader(Cursor::new(data))?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(read_points.len(), 2);

    Ok(())
}

#[test]
fn test_write_pcd_v06() -> pcd_rs::Result<()> {
    // Create test data
    let points = vec![DynRecord(vec![
        Field::F32(vec![1.0]),
        Field::F32(vec![2.0]),
        Field::F32(vec![3.0]),
        Field::U8(vec![255]),
    ])];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
        ("intensity", ValueKind::U8, 1),
    ]);

    // Write to buffer with v0.6
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema),
            version: Some("0.6".to_string()),
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Read back and verify
    let data = buffer.into_inner();
    let content_end = std::cmp::min(data.len(), 500);
    let header = String::from_utf8_lossy(&data[..content_end]);

    // Check that VERSION is .6
    assert!(header.contains("VERSION .6"), "Should contain VERSION .6");

    // Check that VIEWPOINT is NOT present (v0.6 doesn't have it)
    assert!(
        !header.contains("VIEWPOINT"),
        "v0.6 should not contain VIEWPOINT"
    );

    // Check header comment
    assert!(
        header.contains("# .PCD v.6"),
        "Should contain v.6 header comment"
    );

    // Verify we can read it back
    let reader = DynReader::from_reader(Cursor::new(data))?;
    let read_points: Vec<DynRecord> = reader.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(read_points.len(), 1);

    Ok(())
}

#[test]
fn test_write_pcd_v07_explicit() -> pcd_rs::Result<()> {
    // Create test data
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

    // Write to buffer with explicit v0.7
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema),
            version: Some("0.7".to_string()),
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Read back and verify
    let data = buffer.into_inner();
    let content = String::from_utf8_lossy(&data);

    // Check that VERSION is .7
    assert!(content.contains("VERSION .7"), "Should contain VERSION .7");

    // Check that VIEWPOINT IS present (v0.7 has it)
    assert!(
        content.contains("VIEWPOINT"),
        "v0.7 should contain VIEWPOINT"
    );

    // Check header comment
    assert!(
        content.contains("# .PCD v.7"),
        "Should contain v.7 header comment"
    );

    Ok(())
}

#[test]
fn test_write_pcd_default_version() -> pcd_rs::Result<()> {
    // Create test data
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

    // Write to buffer without specifying version (should default to 0.7)
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema),
            version: None, // No version specified
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Read back and verify
    let data = buffer.into_inner();
    let content = String::from_utf8_lossy(&data);

    // Check that VERSION defaults to .7
    assert!(
        content.contains("VERSION .7"),
        "Should default to VERSION .7"
    );

    // Check that VIEWPOINT IS present (default v0.7 has it)
    assert!(
        content.contains("VIEWPOINT"),
        "Default v0.7 should contain VIEWPOINT"
    );

    Ok(())
}

#[test]
fn test_legacy_version_rejects_compressed() {
    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    // Try to write compressed with v0.5
    let mut buffer = Cursor::new(Vec::new());
    let result = WriterInit {
        width: 1,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::BinaryCompressed,
        schema: Some(schema.clone()),
        version: Some("0.5".to_string()),
    }
    .build_from_writer::<DynRecord, _>(&mut buffer);

    assert!(result.is_err());
    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("binary_compressed") || error_msg.contains("v0.7"));
    }

    // Try with v0.6
    let mut buffer2 = Cursor::new(Vec::new());
    let result = WriterInit {
        width: 1,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::BinaryCompressed,
        schema: Some(schema),
        version: Some("0.6".to_string()),
    }
    .build_from_writer::<DynRecord, _>(&mut buffer2);

    assert!(result.is_err());
}

#[test]
fn test_alternative_version_formats() -> pcd_rs::Result<()> {
    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    let points = vec![DynRecord(vec![
        Field::F32(vec![1.0]),
        Field::F32(vec![2.0]),
        Field::F32(vec![3.0]),
    ])];

    // Test ".5" format
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema.clone()),
            version: Some(".5".to_string()),
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    let data = buffer.into_inner();
    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("VERSION .5"));
    assert!(!content.contains("VIEWPOINT"));

    // Test ".6" format
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema.clone()),
            version: Some(".6".to_string()),
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    let data = buffer.into_inner();
    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("VERSION .6"));
    assert!(!content.contains("VIEWPOINT"));

    // Test ".7" format
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema),
            version: Some(".7".to_string()),
        }
        .build_from_writer(&mut buffer)?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    let data = buffer.into_inner();
    let content = String::from_utf8_lossy(&data);
    assert!(content.contains("VERSION .7"));
    assert!(content.contains("VIEWPOINT"));

    Ok(())
}
