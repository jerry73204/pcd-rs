//! Tests for PCL-style padding field (`_`) handling.

use pcd_rs::{DynReader, DynRecord, Field, ValueKind};

#[test]
fn test_padding_field_ascii_skipped() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/padding_ascii.pcd")?;

    // Metadata should still contain the padding field
    let meta = reader.meta();
    assert_eq!(meta.field_defs.len(), 3);
    assert!(meta.field_defs[1].is_padding());

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 2);

    // DynRecord should only have 2 fields (x and y), padding skipped
    assert_eq!(points[0].0.len(), 2);
    match (&points[0].0[0], &points[0].0[1]) {
        (Field::F32(x), Field::F32(y)) => {
            assert_eq!(x[0], 1.0);
            assert_eq!(y[0], 2.0);
        }
        _ => panic!("Expected F32 fields"),
    }

    Ok(())
}

#[test]
fn test_padding_field_binary_skipped() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/padding_binary.pcd")?;
    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

    assert_eq!(points.len(), 1);
    // Only x and y, padding skipped
    assert_eq!(points[0].0.len(), 2);
    match (&points[0].0[0], &points[0].0[1]) {
        (Field::F32(x), Field::F32(y)) => {
            assert_eq!(x[0], 1.5);
            assert_eq!(y[0], 2.5);
        }
        _ => panic!("Expected F32 fields"),
    }

    Ok(())
}

#[test]
fn test_multiple_padding_fields() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/multiple_padding.pcd")?;
    let meta = reader.meta();

    // All padding fields preserved in metadata
    assert_eq!(meta.field_defs.len(), 4);
    assert!(meta.field_defs[1].is_padding());
    assert!(meta.field_defs[2].is_padding());

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points[0].0.len(), 2);

    Ok(())
}

#[test]
fn test_padding_field_metadata_preserved() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/padding_metadata.pcd")?;
    let meta = reader.meta();

    // Padding field keeps `_` name in metadata
    assert_eq!(meta.field_defs[1].name, "_");
    assert_eq!(meta.field_defs[1].kind, ValueKind::U8);

    // Non-padding fields have correct names
    assert_eq!(meta.field_defs[0].name, "x");
    assert_eq!(meta.field_defs[2].name, "rgb");

    Ok(())
}
