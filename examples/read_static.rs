use anyhow::Result;
use pcd_rs::{PcdDeserialize, ReaderBuilder};

#[derive(PcdDeserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

pub fn main() -> Result<()> {
    let reader = ReaderBuilder::open::<Point, _>("test_files/ascii.pcd")?;
    let points: Result<Vec<_>> = reader.collect();
    println!("{} points found", points?.len());
    Ok(())
}
