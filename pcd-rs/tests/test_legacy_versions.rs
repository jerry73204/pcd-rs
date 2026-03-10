//! Tests for legacy PCD version support (0.5 and 0.6)

use pcd_rs::{DataKind, DynReader, DynRecord, Field};

#[test]
fn test_read_pcd_v05_ascii() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/v05_ascii.pcd")?;
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
    let reader = DynReader::open("test_files/v06_binary.pcd")?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.6");
    assert_eq!(meta.data, DataKind::Binary);
    assert_eq!(meta.num_points, 1);

    // Version 0.6 should have default viewpoint
    assert_eq!(meta.viewpoint.tx, 0.0);
    assert_eq!(meta.viewpoint.qw, 1.0);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

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
    let result = DynReader::open("test_files/unsupported_v04.pcd");
    assert!(result.is_err());

    if let Err(e) = result {
        let error_msg = format!("{}", e);
        assert!(error_msg.contains("Unsupported version"));
        assert!(error_msg.contains("0.5, 0.6, 0.7"));
    }
}

#[test]
fn test_v05_no_viewpoint_expected() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/v05_no_viewpoint.pcd")?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.5");
    assert_eq!(meta.viewpoint.qw, 1.0); // Should use defaults

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_legacy_versions_reject_binary_compressed() {
    for path in [
        "test_files/v05_binary_compressed.pcd",
        "test_files/v06_binary_compressed.pcd",
    ] {
        let result = DynReader::open(path);
        assert!(result.is_err(), "{path} should be rejected");

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
    let reader = DynReader::open("test_files/v06_mixed_types_binary.pcd")?;
    let meta = reader.meta();

    assert_eq!(meta.version, "0.6");
    assert_eq!(meta.field_defs.len(), 4);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

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
