use anyhow::Result;
use pcd_rs::{DataKind, DynRecord, Field, ValueKind, Writer, WriterBuilder};

fn main() -> Result<()> {
    // output path
    let path = "test_files/dump_ascii_untyped.pcd";

    // point data
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

    // serialize points
    let schema = vec![
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::U8, 3),
        ("z", ValueKind::I32, 1),
    ];

    let mut writer: Writer<DynRecord, _> =
        WriterBuilder::new(300, 1, Default::default(), DataKind::ASCII)?
            .schema(schema)?
            .create(path)?;

    for point in dump_points.iter() {
        writer.push(&point)?;
    }

    writer.finish()?;

    Ok(())
}
