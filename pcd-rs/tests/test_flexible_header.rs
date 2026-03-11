//! Tests for flexible PCD header parsing (reordered entries, aliases, defaults).

use pcd_rs::{DynReader, DynRecord};

#[test]
fn test_reordered_header() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/reordered_header.pcd")?;
    let meta = reader.meta();
    assert_eq!(meta.num_points, 2);
    assert_eq!(meta.field_defs.len(), 3);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 2);

    Ok(())
}

#[test]
fn test_columns_keyword() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/columns_keyword.pcd")?;
    let meta = reader.meta();
    assert_eq!(meta.field_defs.len(), 3);
    assert_eq!(meta.field_defs[0].name, "x");

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_missing_count_defaults_to_one() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/missing_count.pcd")?;
    let meta = reader.meta();
    for field in meta.field_defs.iter() {
        assert_eq!(field.count, 1);
    }

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_missing_viewpoint_defaults() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/missing_viewpoint.pcd")?;
    let meta = reader.meta();
    assert_eq!(meta.viewpoint.qw, 1.0);
    assert_eq!(meta.viewpoint.tx, 0.0);

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_unknown_header_entries_skipped() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/unknown_entries.pcd")?;
    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_comments_in_header() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/comments_in_header.pcd")?;
    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}

#[test]
fn test_count_zero_treated_as_one() -> pcd_rs::Result<()> {
    let reader = DynReader::open("test_files/count_zero.pcd")?;
    for field in reader.meta().field_defs.iter() {
        assert_eq!(field.count, 1, "COUNT 0 should be treated as 1");
    }

    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(points.len(), 1);

    Ok(())
}
