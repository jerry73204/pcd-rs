#[cfg(feature = "derive")]
mod example {
    use anyhow::Result;
    use pcd_rs::{DataKind, PcdDeserialize, PcdSerialize, Writer, WriterBuilder};

    #[derive(Debug, PcdDeserialize, PcdSerialize, PartialEq)]
    pub struct Point {
        #[pcd_rename("new_x")]
        x: f32,
        y: [u8; 3],
        z: i32,
    }

    pub fn main() -> Result<()> {
        // output path
        let path = "test_files/dump_ascii_static.pcd";

        // point data
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

        // serialize points
        let mut writer: Writer<Point, _> =
            WriterBuilder::new(300, 1, Default::default(), DataKind::Ascii)?.create(path)?;

        for point in dump_points.iter() {
            writer.push(&point)?;
        }

        writer.finish()?;

        Ok(())
    }
}

#[cfg(feature = "derive")]
fn main() -> anyhow::Result<()> {
    example::main()
}

#[cfg(not(feature = "derive"))]
fn main() {
    panic!(r#"please enable "derive" feature to run this example"#);
}
