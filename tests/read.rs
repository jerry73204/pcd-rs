#![cfg(feature = "derive")]

use anyhow::Result;
use itertools::Itertools as _;
use pcd_rs::{DynRecord, PcdDeserialize, Reader};

#[derive(PcdDeserialize)]
pub struct PointAscii {
    #[pcd(ignore)]
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
    #[pcd(ignore)]
    pub rgb: u32,
}

#[test]
fn load_ascii_static() -> Result<()> {
    let reader = Reader::open("test_files/ascii.pcd")?;
    let points: Vec<PointAscii> = reader.try_collect()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary_static() -> Result<()> {
    let reader = Reader::open("test_files/binary.pcd")?;
    let points: Vec<PointBinary> = reader.try_collect()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}

#[test]
fn load_ascii_untyped() -> Result<()> {
    let reader = Reader::open("test_files/ascii.pcd")?;
    let points: Vec<DynRecord> = reader.try_collect()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary_untyped() -> Result<()> {
    let reader = Reader::open("test_files/binary.pcd")?;
    let points: Vec<DynRecord> = reader.try_collect()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
