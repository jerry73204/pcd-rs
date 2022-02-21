#![cfg(feature = "derive")]

use anyhow::Result;
use itertools::Itertools as _;
use pcd_rs::{
    DataKind, DynRecord, Field, PcdDeserialize, PcdSerialize, Reader, Schema, ValueKind, WriterInit,
};

#[derive(Debug, PcdDeserialize, PcdSerialize, PartialEq)]
pub struct Point {
    #[pcd(rename = "new_x")]
    x: f32,
    y: [u8; 3],
    z: i32,
}

#[test]
fn write_ascii_static() -> Result<()> {
    let path = "test_files/dump_ascii_static.pcd";
    let dump_points = vec![
        Point {
            x: 3.14159,
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
    }
    .create(path)?;

    for point in &dump_points {
        writer.push(point)?;
    }

    writer.finish()?;

    let reader = Reader::open(path)?;
    let load_points: Vec<Point> = reader.try_collect()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn write_binary_static() -> Result<()> {
    let path = "test_files/dump_binary_static.pcd";

    let dump_points = vec![
        Point {
            x: 3.14159,
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
    }
    .create(path)?;

    for point in &dump_points {
        writer.push(point)?;
    }

    writer.finish()?;

    let reader = Reader::open(path)?;
    let load_points: Vec<Point> = reader.try_collect()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn write_ascii_untyped() -> Result<()> {
    let path = "test_files/dump_ascii_untyped.pcd";
    let dump_points = vec![
        DynRecord(vec![
            Field::F32(vec![3.14159]),
            Field::U8(vec![2, 1, 7]),
            Field::I32(vec![-5]),
        ]),
        DynRecord(vec![
            Field::F32(vec![-0.0]),
            Field::U8(vec![254, 6, 98]),
            Field::I32(vec![7]),
        ]),
        DynRecord(vec![
            Field::F32(vec![5.6]),
            Field::U8(vec![4, 0, 111]),
            Field::I32(vec![-100000]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::U8, 3),
        ("z", ValueKind::I32, 1),
    ]);

    let mut writer = WriterInit {
        width: 300,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Ascii,
        schema: Some(schema),
    }
    .create(path)?;

    for point in &dump_points {
        writer.push(point)?;
    }

    writer.finish()?;

    let reader: Reader<DynRecord, _> = Reader::open(path)?;
    let load_points = reader.collect::<Result<Vec<_>>>()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn write_binary_untyped() -> Result<()> {
    let path = "test_files/dump_binary_untyped.pcd";

    let dump_points = vec![
        DynRecord(vec![
            Field::F32(vec![3.14159]),
            Field::U8(vec![2, 1, 7]),
            Field::I32(vec![-5]),
        ]),
        DynRecord(vec![
            Field::F32(vec![-0.0]),
            Field::U8(vec![254, 6, 98]),
            Field::I32(vec![7]),
        ]),
        DynRecord(vec![
            Field::F32(vec![5.6]),
            Field::U8(vec![4, 0, 111]),
            Field::I32(vec![-100000]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::U8, 3),
        ("z", ValueKind::I32, 1),
    ]);

    let mut writer = WriterInit {
        width: 300,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: Some(schema),
    }
    .create(path)?;

    for point in &dump_points {
        writer.push(point)?;
    }

    writer.finish()?;

    let reader = Reader::open(path)?;
    let load_points: Vec<DynRecord> = reader.try_collect()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}
