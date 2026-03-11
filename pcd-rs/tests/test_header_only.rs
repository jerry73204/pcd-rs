//! Tests for header-only (metadata) reading without loading point data.

use pcd_rs::{DataKind, PcdMeta, ValueKind};
use tempfile::NamedTempFile;

#[test]
fn test_meta_from_reader_ascii() -> pcd_rs::Result<()> {
    let meta = PcdMeta::from_path("test_files/header_only_ascii.pcd")?;

    assert_eq!(meta.version, "0.7");
    assert_eq!(meta.num_points, 100);
    assert_eq!(meta.data, DataKind::Ascii);
    assert_eq!(meta.field_defs.len(), 3);
    assert_eq!(meta.field_defs[0].name, "x");
    assert_eq!(meta.field_defs[0].kind, ValueKind::F32);

    Ok(())
}

#[test]
fn test_meta_from_reader_binary_compressed() -> pcd_rs::Result<()> {
    let meta = PcdMeta::from_path("test_files/header_only_compressed.pcd")?;

    assert_eq!(meta.data, DataKind::BinaryCompressed);
    assert_eq!(meta.num_points, 500);
    assert_eq!(meta.field_defs.len(), 4);
    assert_eq!(meta.field_defs[3].name, "intensity");
    assert_eq!(meta.field_defs[3].kind, ValueKind::U8);

    Ok(())
}

#[test]
fn test_meta_from_path() -> pcd_rs::Result<()> {
    use pcd_rs::{DynRecord, DynWriter, Schema, WriterInit};

    let schema = Schema::from_iter([("x", ValueKind::F32, 1), ("y", ValueKind::F32, 1)]);

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer: DynWriter<_> = WriterInit {
            width: 5,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema),
            version: None,
        }
        .create(file.path())?;

        for i in 0..5 {
            writer.push(&DynRecord(vec![
                pcd_rs::Field::F32(vec![i as f32]),
                pcd_rs::Field::F32(vec![i as f32 * 2.0]),
            ]))?;
        }
        writer.finish()?;
    }

    // Read only metadata, no point data
    let meta = PcdMeta::from_path(file.path())?;

    assert_eq!(meta.field_defs.len(), 2);
    assert_eq!(meta.data, DataKind::Binary);
    assert_eq!(meta.field_defs[0].name, "x");

    Ok(())
}

#[test]
fn test_meta_from_path_all_versions() -> pcd_rs::Result<()> {
    let cases = [
        ("test_files/header_only_v05.pcd", "0.5"),
        ("test_files/header_only_v06.pcd", "0.6"),
        ("test_files/header_only_v07.pcd", "0.7"),
    ];

    for (path, expected_version) in cases {
        let meta = PcdMeta::from_path(path)?;
        assert_eq!(meta.version, expected_version);
    }

    Ok(())
}
