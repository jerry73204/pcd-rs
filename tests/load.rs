use failure::Fallible;
use pcd_rs::SeqReaderOptions;
use std::path::Path;

#[test]
fn load_ascii() -> Fallible<()> {
    // let path = Path::new("test_files/ascii.pcd");
    let reader = SeqReaderOptions::from_path("test_files/ascii.pcd")?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 213);
    Ok(())
}

#[test]
fn load_binary() -> Fallible<()> {
    let path = Path::new("test_files/binary.pcd");
    let reader = SeqReaderOptions::from_path(path)?;
    let points = reader.collect::<Fallible<Vec<_>>>()?;
    assert_eq!(points.len(), 28944);
    Ok(())
}
