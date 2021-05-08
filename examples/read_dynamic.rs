use anyhow::Result;
use pcd_rs::{DynReader, ReaderBuilder};

fn main() -> Result<()> {
    let reader: DynReader<_> = ReaderBuilder::open("test_files/binary.pcd")?;
    let points: Result<Vec<_>> = reader.collect();
    println!("{} points", points?.len());
    Ok(())
}
