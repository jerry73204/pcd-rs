use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};

fn main() -> pcd_rs::Result<()> {
    println!("Testing empty compressed point cloud...");

    let points: Vec<DynRecord> = vec![];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    let path = "test_files/empty_compressed_debug.pcd";

    println!("Creating writer...");
    {
        let writer = WriterInit {
            width: 0,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema.clone()),
        }
        .create::<DynRecord, _>(path)?;

        println!("Finishing writer...");
        writer.finish()?;
    }

    println!("File written. Now reading it back...");

    // Check the file contents
    let contents = std::fs::read(path)?;
    println!("File size: {} bytes", contents.len());

    // Read it back
    let reader = DynReader::open(path)?;
    println!("Reader opened successfully");
    assert_eq!(reader.meta().num_points, 0);

    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
    assert_eq!(read_points.len(), 0);

    println!("✅ Success!");
    Ok(())
}
