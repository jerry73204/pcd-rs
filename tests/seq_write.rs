use failure::Fallible;
use pcd_rs::{DataKind, PCDRecordRead, PCDRecordWrite, SeqReaderBuilder, SeqWriterBuilder};
use std::path::Path;

#[derive(Debug, PCDRecordRead, PCDRecordWrite, PartialEq)]
pub struct Point {
    #[pcd_rename("new_x")]
    x: f32,
    y: [u8; 3],
    z: i32,
}

#[test]
fn dump_ascii() -> Fallible<()> {
    let path = "test_files/dump_ascii.pcd";
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

    {
        let mut writer =
            SeqWriterBuilder::<Point>::new(300, 1, Default::default(), DataKind::ASCII)?
                .create(path)?;

        for point in dump_points.iter() {
            writer.push(&point)?;
        }
    }

    {
        let reader = SeqReaderBuilder::open(path)?;
        let load_points = reader.collect::<Fallible<Vec<Point>>>()?;
        assert_eq!(dump_points, load_points);
    }

    Ok(())
}

#[test]
fn dump_binary() -> Fallible<()> {
    let path = "test_files/dump_binary.pcd";

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

    {
        let mut writer =
            SeqWriterBuilder::<Point>::new(300, 1, Default::default(), DataKind::Binary)?
                .create(path)?;

        for point in dump_points.iter() {
            writer.push(&point)?;
        }
    }

    {
        let reader = SeqReaderBuilder::open(path)?;
        let load_points = reader.collect::<Fallible<Vec<Point>>>()?;
        assert_eq!(dump_points, load_points);
    }

    Ok(())
}
