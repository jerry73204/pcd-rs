use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};

fn main() -> pcd_rs::Result<()> {
    println!("Demonstrating Format Support Features\n");

    // 1. Legacy PCD Version Support
    println!("=== Legacy PCD Version Support ===");

    // Test v0.5
    println!("Reading PCD v0.5 (ASCII):");
    let reader_v05 = DynReader::open("pcd-rs/test_files/legacy_v05.pcd")?;
    let meta_v05 = reader_v05.meta().clone();
    println!(
        "  Version: {} | Format: {:?} | Points: {}",
        meta_v05.version, meta_v05.data, meta_v05.num_points
    );
    let points_v05: Vec<_> = reader_v05.collect::<Result<_, _>>()?;
    println!("  Successfully read {} points", points_v05.len());

    // Test v0.6
    println!("Reading PCD v0.6 (Binary):");
    let reader_v06 = DynReader::open("pcd-rs/test_files/legacy_v06.pcd")?;
    let meta_v06 = reader_v06.meta().clone();
    println!(
        "  Version: {} | Format: {:?} | Points: {}",
        meta_v06.version, meta_v06.data, meta_v06.num_points
    );
    let points_v06: Vec<_> = reader_v06.collect::<Result<_, _>>()?;
    println!("  Successfully read {} points", points_v06.len());

    // Test real PCL file
    println!("Reading real PCL bunny.pcd (v0.5, 397 points):");
    let reader_bunny = DynReader::open("pcd-rs/test_files/bunny_v05.pcd")?;
    let meta_bunny = reader_bunny.meta().clone();
    println!(
        "  Version: {} | Format: {:?} | Points: {}",
        meta_bunny.version, meta_bunny.data, meta_bunny.num_points
    );
    let points_bunny: Vec<_> = reader_bunny.collect::<Result<_, _>>()?;
    println!(
        "  Successfully read {} points from PCL repository",
        points_bunny.len()
    );

    println!();

    // 2. Binary Compressed Format Support
    println!("=== Binary Compressed Format Support (LZF) ===");

    // Create test data for compression
    let test_points = vec![
        DynRecord(vec![
            Field::F32(vec![1.0]),
            Field::F32(vec![2.0]),
            Field::F32(vec![3.0]),
            Field::F32(vec![0.5]),
        ]),
        DynRecord(vec![
            Field::F32(vec![4.0]),
            Field::F32(vec![5.0]),
            Field::F32(vec![6.0]),
            Field::F32(vec![0.8]),
        ]),
        DynRecord(vec![
            Field::F32(vec![7.0]),
            Field::F32(vec![8.0]),
            Field::F32(vec![9.0]),
            Field::F32(vec![1.0]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
        ("intensity", ValueKind::F32, 1),
    ]);

    // Write compressed PCD v0.7
    let compressed_path = "pcd-rs/test_files/demo_compressed.pcd";
    println!("Writing binary_compressed PCD v0.7...");
    {
        let mut writer = WriterInit {
            width: test_points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
            version: None,
        }
        .create::<DynRecord, _>(compressed_path)?;

        for point in &test_points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Read it back
    println!("Reading binary_compressed PCD v0.7:");
    let reader_compressed = DynReader::open(compressed_path)?;
    let meta_compressed = reader_compressed.meta().clone();
    println!(
        "  Version: {} | Format: {:?} | Points: {}",
        meta_compressed.version, meta_compressed.data, meta_compressed.num_points
    );
    let points_compressed: Vec<_> = reader_compressed.collect::<Result<_, _>>()?;
    println!(
        "  Successfully read {} compressed points",
        points_compressed.len()
    );

    // Verify data integrity
    for (i, (original, read)) in test_points.iter().zip(points_compressed.iter()).enumerate() {
        for (j, (orig_field, read_field)) in original.0.iter().zip(read.0.iter()).enumerate() {
            match (orig_field, read_field) {
                (Field::F32(o), Field::F32(r)) => {
                    if (o[0] - r[0]).abs() > 1e-6 {
                        panic!(
                            "Data mismatch at point {} field {}: {} != {}",
                            i, j, o[0], r[0]
                        );
                    }
                }
                _ => panic!("Type mismatch"),
            }
        }
    }
    println!("  Data integrity verified - perfect round-trip compression");

    println!();

    // 3. Format Compatibility Matrix
    println!("=== Format Compatibility Matrix ===");
    println!("Supported combinations:");
    println!("  PCD v0.5: ASCII [yes], Binary [yes], Compressed [no]");
    println!("  PCD v0.6: ASCII [yes], Binary [yes], Compressed [no]");
    println!("  PCD v0.7: ASCII [yes], Binary [yes], Compressed [yes]");

    Ok(())
}
