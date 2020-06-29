#[cfg(feature = "derive")]
mod example {
    use anyhow::Result;
    use pcd_rs::{PcdDeserialize, Reader, ReaderBuilder};

    #[derive(PcdDeserialize)]
    pub struct Point {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub rgb: f32,
    }

    pub fn main() -> Result<()> {
        let reader: Reader<Point, _> = ReaderBuilder::from_path("test_files/ascii.pcd")?;
        let points = reader.collect::<Result<Vec<_>>>()?;
        println!("{} points found", points.len());
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
