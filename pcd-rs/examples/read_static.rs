use eyre::Result;
use pcd_rs::{PcdDeserialize, Reader};

#[derive(PcdDeserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

pub fn main() -> Result<()> {
    let reader = Reader::open("test_files/ascii.pcd")?;
    let points: Result<Vec<Point>, _> = reader.collect();
    println!("{} points found", points?.len());
    Ok(())
}
