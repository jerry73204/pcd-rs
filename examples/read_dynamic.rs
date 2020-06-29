use anyhow::Result;
use pcd_rs::{DynRecord, Reader, ReaderBuilder};

fn main() -> Result<()> {
    let reader: Reader<DynRecord, _> = ReaderBuilder::from_path("test_files/binary.pcd")?;
    let points = reader.collect::<Result<Vec<_>>>()?;
    println!("{} points", points.len());
    Ok(())
}
