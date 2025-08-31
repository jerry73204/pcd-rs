//! Tests for legacy PCD version support (0.5 and 0.6)

use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};
use std::{fs, io::Write};

#[test]
fn test_read_pcd_v05_ascii() -> pcd_rs::Result<()> {
    // Test reading PCD version 0.5 with ASCII data
    let path = "test_files/test_v05_ascii.pcd";

    // Create a test file
    let mut file = fs::File::create(path)?;
    writeln!(file, "VERSION .5")?;
    writeln!(file, "FIELDS x y z")?;
    writeln!(file, "SIZE 4 4 4")?;
    writeln!(file, "TYPE F F F")?;
    writeln!(file, "COUNT 1 1 1")?;
    writeln!(file, "WIDTH 2")?;
    writeln!(file, "HEIGHT 1")?;
    writeln!(file, "POINTS 2")?;
    writeln!(file, "DATA ascii")?;
    writeln!(file, "1.0 2.0 3.0")?;
    writeln!(file, "4.0 5.0 6.0")?;

    // Read and verify
    let reader = DynReader::open(path)?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.5");
    assert_eq!(meta.data, DataKind::Ascii);
    assert_eq!(meta.num_points, 2);

    // Version 0.5 should have default viewpoint
    assert_eq!(meta.viewpoint.tx, 0.0);
    assert_eq!(meta.viewpoint.ty, 0.0);
    assert_eq!(meta.viewpoint.tz, 0.0);
    assert_eq!(meta.viewpoint.qw, 1.0);
    assert_eq!(meta.viewpoint.qx, 0.0);
    assert_eq!(meta.viewpoint.qy, 0.0);
    assert_eq!(meta.viewpoint.qz, 0.0);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 2);

    // Verify first point
    match (&points[0].0[0], &points[0].0[1], &points[0].0[2]) {
        (Field::F32(x), Field::F32(y), Field::F32(z)) => {
            assert_eq!(x[0], 1.0);
            assert_eq!(y[0], 2.0);
            assert_eq!(z[0], 3.0);
        }
        _ => panic!("Unexpected field types"),
    }

    Ok(())
}

#[test]
fn test_read_pcd_v06_binary() -> pcd_rs::Result<()> {
    // Test reading PCD version 0.6 with binary data
    let path = "test_files/test_v06_binary.pcd";

    // Create header
    let mut file = fs::File::create(path)?;
    writeln!(file, "VERSION .6")?;
    writeln!(file, "FIELDS x y z intensity")?;
    writeln!(file, "SIZE 4 4 4 1")?;
    writeln!(file, "TYPE F F F U")?;
    writeln!(file, "COUNT 1 1 1 1")?;
    writeln!(file, "WIDTH 1")?;
    writeln!(file, "HEIGHT 1")?;
    writeln!(file, "POINTS 1")?;
    writeln!(file, "DATA binary")?;

    // Add binary data
    drop(file);
    use std::io::Write;
    let mut file = fs::OpenOptions::new().append(true).open(path)?;

    // Write binary data: x=1.5, y=2.5, z=3.5, intensity=100
    file.write_all(&1.5f32.to_le_bytes())?;
    file.write_all(&2.5f32.to_le_bytes())?;
    file.write_all(&3.5f32.to_le_bytes())?;
    file.write_all(&[100u8])?;

    // Read and verify
    let reader = DynReader::open(path)?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.6");
    assert_eq!(meta.data, DataKind::Binary);
    assert_eq!(meta.num_points, 1);

    // Version 0.6 should have default viewpoint
    assert_eq!(meta.viewpoint.tx, 0.0);
    assert_eq!(meta.viewpoint.qw, 1.0);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    // Verify point data
    match (
        &points[0].0[0],
        &points[0].0[1],
        &points[0].0[2],
        &points[0].0[3],
    ) {
        (Field::F32(x), Field::F32(y), Field::F32(z), Field::U8(intensity)) => {
            assert_eq!(x[0], 1.5);
            assert_eq!(y[0], 2.5);
            assert_eq!(z[0], 3.5);
            assert_eq!(intensity[0], 100);
        }
        _ => panic!("Unexpected field types"),
    }

    Ok(())
}

#[test]
fn test_unsupported_version() {
    // Test that unsupported versions are rejected
    let path = "test_files/test_unsupported_version.pcd";

    let mut file = fs::File::create(path).unwrap();
    writeln!(file, "VERSION .4").unwrap(); // Unsupported version
    writeln!(file, "FIELDS x y z").unwrap();
    writeln!(file, "SIZE 4 4 4").unwrap();
    writeln!(file, "TYPE F F F").unwrap();
    writeln!(file, "COUNT 1 1 1").unwrap();
    writeln!(file, "WIDTH 1").unwrap();
    writeln!(file, "HEIGHT 1").unwrap();
    writeln!(file, "POINTS 1").unwrap();
    writeln!(file, "DATA ascii").unwrap();
    writeln!(file, "1.0 2.0 3.0").unwrap();

    let result = DynReader::open(path);
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("Unsupported version"));
        assert!(error_msg.contains("0.5, 0.6, 0.7"));
    }
}

#[test]
fn test_v05_v06_no_viewpoint_expected() -> pcd_rs::Result<()> {
    // Test that versions 0.5 and 0.6 don't expect VIEWPOINT line
    let path = "test_files/test_v05_no_viewpoint.pcd";

    let mut file = fs::File::create(path)?;
    writeln!(file, "VERSION .5")?;
    writeln!(file, "FIELDS x")?;
    writeln!(file, "SIZE 4")?;
    writeln!(file, "TYPE F")?;
    writeln!(file, "COUNT 1")?;
    writeln!(file, "WIDTH 1")?;
    writeln!(file, "HEIGHT 1")?;
    // No VIEWPOINT line - should work for v0.5
    writeln!(file, "POINTS 1")?;
    writeln!(file, "DATA ascii")?;
    writeln!(file, "42.0")?;

    let reader = DynReader::open(path)?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.5");
    assert_eq!(meta.viewpoint.qw, 1.0); // Should use defaults

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_legacy_versions_reject_binary_compressed() {
    // Test that versions 0.5 and 0.6 reject binary_compressed format
    for version in &[".5", ".6"] {
        let path = format!(
            "test_files/test_{}_compressed.pcd",
            version.replace(".", "v")
        );

        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "VERSION {}", version).unwrap();
        writeln!(file, "FIELDS x y z").unwrap();
        writeln!(file, "SIZE 4 4 4").unwrap();
        writeln!(file, "TYPE F F F").unwrap();
        writeln!(file, "COUNT 1 1 1").unwrap();
        writeln!(file, "WIDTH 1").unwrap();
        writeln!(file, "HEIGHT 1").unwrap();
        writeln!(file, "POINTS 1").unwrap();
        writeln!(file, "DATA binary_compressed").unwrap(); // Should be rejected

        let result = DynReader::open(&path);
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("binary_compressed format is only supported in PCD version 0.7")
            );
        }
    }
}

#[test]
fn test_legacy_mixed_data_types() -> pcd_rs::Result<()> {
    // Test legacy versions with various data types
    let path = "test_files/test_v06_mixed_types.pcd";

    // Create header
    let mut file = fs::File::create(path)?;
    writeln!(file, "VERSION 0.6")?; // Test "0.6" format (not ".6")
    writeln!(file, "FIELDS pos normal_x rgb label")?;
    writeln!(file, "SIZE 4 4 2 4")?;
    writeln!(file, "TYPE F F U I")?;
    writeln!(file, "COUNT 1 1 1 1")?;
    writeln!(file, "WIDTH 1")?;
    writeln!(file, "HEIGHT 1")?;
    writeln!(file, "POINTS 1")?;
    writeln!(file, "DATA binary")?;

    // Add binary data
    drop(file);
    let mut file = fs::OpenOptions::new().append(true).open(path)?;

    // pos=10.5, normal_x=0.5, rgb=65535, label=-42
    file.write_all(&10.5f32.to_le_bytes())?;
    file.write_all(&0.5f32.to_le_bytes())?;
    file.write_all(&65535u16.to_le_bytes())?;
    file.write_all(&(-42i32).to_le_bytes())?;

    // Read and verify
    let reader = DynReader::open(path)?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.6");
    assert_eq!(meta.field_defs.len(), 4);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    // Verify mixed data types
    match (
        &points[0].0[0],
        &points[0].0[1],
        &points[0].0[2],
        &points[0].0[3],
    ) {
        (Field::F32(pos), Field::F32(normal), Field::U16(rgb), Field::I32(label)) => {
            assert_eq!(pos[0], 10.5);
            assert_eq!(normal[0], 0.5);
            assert_eq!(rgb[0], 65535);
            assert_eq!(label[0], -42);
        }
        _ => panic!("Unexpected field types: {:?}", points[0]),
    }

    Ok(())
}
