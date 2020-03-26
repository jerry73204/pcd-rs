use failure::Fallible;
use pcd_rs::{DynRecord, PcdDeserialize, Reader, ReaderBuilder};
use std::path::Path;

#[derive(PcdDeserialize)]
pub struct PointAscii {
    #[pcd_ignore_name]
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

#[derive(PcdDeserialize)]
pub struct PointBinary {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[pcd_ignore_name]
    pub rgb: u32,
}

#[test]
fn load_ascii_static() -> Fallible<()> {
    let reader: Reader<PointAscii, _> = ReaderBuilder::from_path("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary_static() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader: Reader<PointBinary, _> = ReaderBuilder::from_path(path)?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}

#[test]
fn load_ascii_untyped() -> Fallible<()> {
    let reader: Reader<DynRecord, _> = ReaderBuilder::from_path("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary_untyped() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader: Reader<DynRecord, _> = ReaderBuilder::from_path(path)?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
