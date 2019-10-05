use failure::Fallible;
use pcd_rs::{
    record::UntypedRecord,
    seq_reader::{SeqReaderBuilder, SeqReaderBuilderEx},
    PCDRecordRead,
};
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
fn load_ascii_static() -> Fallible<()> {
    let reader = SeqReaderBuilder::<PointAscii, _>::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary_static() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader = SeqReaderBuilder::<PointBinary, _>::open(path)?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}

#[test]
fn load_ascii_dynamic() -> Fallible<()> {
    let reader = SeqReaderBuilder::<UntypedRecord, _>::open("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary_dynamic() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader = SeqReaderBuilder::<UntypedRecord, _>::open(path)?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
