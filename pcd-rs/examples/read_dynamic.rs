use eyre::Result;
use pcd_rs::DynReader;

fn main() -> Result<()> {
    let reader = DynReader::open("pcd-rs/test_files/binary.pcd")?;
    let points: Result<Vec<_>, _> = reader.collect();
    println!("{} points", points?.len());
    Ok(())
}
