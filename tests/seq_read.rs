use failure::Fallible;
use pcd_rs::{PCDRecordRead, SeqReaderBuilder};
use std::path::Path;

#[derive(PCDRecordRead)]
pub struct PointAscii {
    #[pcd_ignore_name]
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

#[derive(PCDRecordRead)]
pub struct PointBinary {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[pcd_ignore_name]
    pub rgb: u32,
}

#[test]
fn load_ascii() -> Fallible<()> {
    let reader = SeqReaderBuilder::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<PointAscii>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader = SeqReaderBuilder::open(path)?;
    let points = reader.collect::<Fallible<Vec<PointBinary>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
