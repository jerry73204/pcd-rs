use failure::Fallible;
use pcd_rs::{
    meta::{DataKind, ValueKind},
    record::{Field, UntypedRecord},
    seq_reader::{SeqReaderBuilder, SeqReaderBuilderEx},
    seq_writer::{SeqWriterBuilder, SeqWriterBuilderEx, SeqWriterEx},
    PCDRecordRead, PCDRecordWrite,
};

#[derive(Debug, PCDRecordRead, PCDRecordWrite, PartialEq)]
pub struct Point {
    #[pcd_rename("new_x")]
    x: f32,
    y: [u8; 3],
    z: i32,
}

#[test]
fn write_ascii_static() -> Fallible<()> {
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

    let mut writer =
        SeqWriterBuilder::<Point, _>::new(300, 1, Default::default(), DataKind::ASCII)?
            .create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    let reader = SeqReaderBuilder::<Point, _>::open(path)?;
    let load_points = reader.collect::<Fallible<Vec<_>>>()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn write_binary_static() -> Fallible<()> {
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

    let mut writer =
        SeqWriterBuilder::<Point, _>::new(300, 1, Default::default(), DataKind::Binary)?
            .create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    let reader = SeqReaderBuilder::<Point, _>::open(path)?;
    let load_points = reader.collect::<Fallible<Vec<_>>>()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn write_ascii_untyped() -> Fallible<()> {
    let path = "test_files/dump_ascii_untyped.pcd";
    let dump_points = vec![
        UntypedRecord(vec![
            Field::F32(vec![3.14159]),
            Field::U8(vec![2, 1, 7]),
            Field::I32(vec![-5]),
        ]),
        UntypedRecord(vec![
            Field::F32(vec![-0.0]),
            Field::U8(vec![254, 6, 98]),
            Field::I32(vec![7]),
        ]),
        UntypedRecord(vec![
            Field::F32(vec![5.6]),
            Field::U8(vec![4, 0, 111]),
            Field::I32(vec![-100000]),
        ]),
    ];

    let schema = vec![
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::U8, 3),
        ("z", ValueKind::I32, 1),
    ];

    let mut writer = SeqWriterBuilder::<UntypedRecord, _>::new(
        300,
        1,
        Default::default(),
        DataKind::ASCII,
        schema,
    )?
    .create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    let reader = SeqReaderBuilder::<UntypedRecord, _>::open(path)?;
    let load_points = reader.collect::<Fallible<Vec<_>>>()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}

#[test]
fn write_binary_untyped() -> Fallible<()> {
    let path = "test_files/dump_binary_untyped.pcd";

    let dump_points = vec![
        UntypedRecord(vec![
            Field::F32(vec![3.14159]),
            Field::U8(vec![2, 1, 7]),
            Field::I32(vec![-5]),
        ]),
        UntypedRecord(vec![
            Field::F32(vec![-0.0]),
            Field::U8(vec![254, 6, 98]),
            Field::I32(vec![7]),
        ]),
        UntypedRecord(vec![
            Field::F32(vec![5.6]),
            Field::U8(vec![4, 0, 111]),
            Field::I32(vec![-100000]),
        ]),
    ];

    let schema = vec![
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::U8, 3),
        ("z", ValueKind::I32, 1),
    ];

    let mut writer = SeqWriterBuilder::<UntypedRecord, _>::new(
        300,
        1,
        Default::default(),
        DataKind::Binary,
        schema,
    )?
    .create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    let reader = SeqReaderBuilder::<UntypedRecord, _>::open(path)?;
    let load_points = reader.collect::<Fallible<Vec<_>>>()?;

    assert_eq!(dump_points, load_points);
    std::fs::remove_file(path)?;

    Ok(())
}
