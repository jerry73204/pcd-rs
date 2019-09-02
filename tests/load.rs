use failure::Fallible;
use pcd_rs::{PCDRecord, SeqReaderBuilder};
use std::path::Path;

#[derive(PCDRecord)]
pub struct PointAscii {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(PCDRecord)]
pub struct PointBinary {
    x: f32,
    y: f32,
    z: f32,
    timestamp: u32,
}

#[test]
fn load_ascii() -> Fallible<()> {
    let reader = SeqReaderBuilder::open_path("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<PointAscii>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader = SeqReaderBuilder::open_path(path)?;
    let points = reader.collect::<Fallible<Vec<PointBinary>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
