use anyhow::Result;
use itertools::Itertools as _;
use pcd_rs::{DynRecord, Reader};

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
