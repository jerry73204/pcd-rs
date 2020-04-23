use failure::Fallible;
use pcd_rs::{DynRecord, Reader, ReaderBuilder};

fn main() -> Fallible<()> {
    let reader: Reader<DynRecord, _> = ReaderBuilder::from_path("test_files/binary.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    println!("{} points", points.len());
    Ok(())
}
