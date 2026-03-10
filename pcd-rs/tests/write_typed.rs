#![cfg(feature = "derive")]

use eyre::Result;
use itertools::Itertools as _;
use pcd_rs::{DataKind, PcdDeserialize, PcdSerialize, Reader, WriterInit};
use tempfile::NamedTempFile;

#[derive(Debug, PcdDeserialize, PcdSerialize, PartialEq)]
pub struct Point {
    #[pcd(rename = "new_x")]
    x: f32,
    y: [u8; 3],
    z: i32,
}

#[test]
fn write_ascii_typed() -> Result<()> {
    let file = NamedTempFile::new()?;
    let dump_points = vec![
        Point {
            x: 3.5,
            y: [2, 1, 7],
            z: -5,
        },
        Point {
            x: -0.0,
            y: [254, 6, 98],
            z: 7,
        },
        Point {
            x: 5.6,
            y: [4, 0, 111],
            z: -100000,
        },
    ];

    let mut writer = WriterInit {
        width: 300,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Ascii,
        schema: None,
        version: None,
    }
    .create(file.path())?;

    for point in &dump_points {
        writer.push(point)?;
    }

    writer.finish()?;

    let reader = Reader::open(file.path())?;
    let load_points: Vec<Point> = reader.try_collect()?;

    assert_eq!(dump_points, load_points);

    Ok(())
}

#[test]
fn write_binary_typed() -> Result<()> {
    let file = NamedTempFile::new()?;

    let dump_points = vec![
        Point {
            x: 3.5,
            y: [2, 1, 7],
            z: -5,
        },
        Point {
            x: -0.0,
            y: [254, 6, 98],
            z: 7,
        },
        Point {
            x: 5.6,
            y: [4, 0, 111],
            z: -100000,
        },
    ];

    let mut writer = WriterInit {
        width: 300,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: None,
        version: None,
    }
    .create(file.path())?;

    for point in &dump_points {
        writer.push(point)?;
    }

    writer.finish()?;

    let reader = Reader::open(file.path())?;
    let load_points: Vec<Point> = reader.try_collect()?;

    assert_eq!(dump_points, load_points);

    Ok(())
}
