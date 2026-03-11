//! Tests for Rgb/Rgba wrapper types with PcdSerialize/PcdDeserialize derive.

use pcd_rs::{DataKind, PcdDeserialize, PcdSerialize, Reader, Rgb, Rgba, Writer, WriterInit};
use tempfile::NamedTempFile;

#[derive(Debug, PcdSerialize, PcdDeserialize, PartialEq)]
struct PointRgb {
    x: f32,
    y: f32,
    z: f32,
    rgb: Rgb,
}

#[derive(Debug, PcdSerialize, PcdDeserialize, PartialEq)]
struct PointRgba {
    x: f32,
    y: f32,
    z: f32,
    rgba: Rgba,
}

#[test]
fn test_rgb_derive_ascii_round_trip() -> pcd_rs::Result<()> {
    let points = vec![
        PointRgb {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            rgb: Rgb::new(255, 0, 0),
        },
        PointRgb {
            x: 4.0,
            y: 5.0,
            z: 6.0,
            rgb: Rgb::new(0, 128, 255),
        },
    ];

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer: Writer<PointRgb, _> = WriterInit {
            width: 2,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: None,
            version: None,
        }
        .create(file.path())?;

        for p in &points {
            writer.push(p)?;
        }
        writer.finish()?;
    }

    let reader = Reader::open(file.path())?;
    let read_points: Vec<PointRgb> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points, points);

    Ok(())
}

#[test]
fn test_rgb_derive_binary_round_trip() -> pcd_rs::Result<()> {
    let points = vec![
        PointRgb {
            x: 1.5,
            y: 2.5,
            z: 3.5,
            rgb: Rgb::new(10, 20, 30),
        },
        PointRgb {
            x: -1.0,
            y: 0.0,
            z: 100.0,
            rgb: Rgb::new(200, 100, 50),
        },
    ];

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer: Writer<PointRgb, _> = WriterInit {
            width: 2,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: None,
            version: None,
        }
        .create(file.path())?;

        for p in &points {
            writer.push(p)?;
        }
        writer.finish()?;
    }

    let reader = Reader::open(file.path())?;
    let read_points: Vec<PointRgb> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points, points);

    Ok(())
}

#[test]
fn test_rgba_derive_binary_round_trip() -> pcd_rs::Result<()> {
    let points = vec![
        PointRgba {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            rgba: Rgba::new(255, 128, 64, 200),
        },
        PointRgba {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            rgba: Rgba::new(0, 0, 0, 0),
        },
    ];

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer: Writer<PointRgba, _> = WriterInit {
            width: 2,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: None,
            version: None,
        }
        .create(file.path())?;

        for p in &points {
            writer.push(p)?;
        }
        writer.finish()?;
    }

    let reader = Reader::open(file.path())?;
    let read_points: Vec<PointRgba> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points, points);

    Ok(())
}

#[test]
fn test_rgb_derive_compressed_round_trip() -> pcd_rs::Result<()> {
    let points = vec![
        PointRgb {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            rgb: Rgb::new(255, 0, 0),
        },
        PointRgb {
            x: 4.0,
            y: 5.0,
            z: 6.0,
            rgb: Rgb::new(0, 255, 0),
        },
        PointRgb {
            x: 7.0,
            y: 8.0,
            z: 9.0,
            rgb: Rgb::new(0, 0, 255),
        },
    ];

    let file = NamedTempFile::new().unwrap();

    {
        let mut writer: Writer<PointRgb, _> = WriterInit {
            width: 3,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: None,
            version: None,
        }
        .create(file.path())?;

        for p in &points {
            writer.push(p)?;
        }
        writer.finish()?;
    }

    let reader = Reader::open(file.path())?;
    let read_points: Vec<PointRgb> = reader.collect::<Result<_, _>>()?;

    assert_eq!(read_points, points);

    Ok(())
}

#[test]
fn test_rgb_schema_reports_f32() {
    // Verify the write_spec reports F32 for the rgb field
    let spec = <PointRgb as pcd_rs::PcdSerialize>::write_spec();
    assert_eq!(spec.len(), 4);
    assert_eq!(spec[3].kind, pcd_rs::ValueKind::F32);
    assert_eq!(spec[3].name, "rgb");
}
