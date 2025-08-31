//! Example to write and read back a compressed PCD file

use pcd_rs::{DataKind, DynReader, DynRecord, Field, Schema, ValueKind, WriterInit};
use std::fs;

fn main() -> pcd_rs::Result<()> {
    println!("Testing write and read of binary_compressed PCD files");

    // Create some test data
    let points = vec![
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

    // Write compressed file
    {
        println!("\nWriting compressed PCD file...");
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema.clone()),
        }
        .create("test_files/output_compressed.pcd")?;

        for point in &points {
            writer.push(point)?;
        }

        writer.finish()?;
        println!(
            "Written {} points to test_files/output_compressed.pcd",
            points.len()
        );
    }

    // Read it back
    {
        println!("\nReading back the compressed file...");
        let reader = DynReader::open("test_files/output_compressed.pcd")?;

        let meta = reader.meta();
        println!("  Data format: {:?}", meta.data);
        println!("  Points: {}", meta.num_points);

        let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

        println!("  Read {} points", read_points.len());

        // Verify the data
        for (i, (original, read)) in points.iter().zip(read_points.iter()).enumerate() {
            println!("\nPoint {}:", i);
            for (j, (orig_field, read_field)) in original.0.iter().zip(read.0.iter()).enumerate() {
                match (orig_field, read_field) {
                    (Field::F32(o), Field::F32(r)) => {
                        println!("  Field {}: {} == {}", j, o[0], r[0]);
                        assert_eq!(o[0], r[0], "Field mismatch at point {} field {}", i, j);
                    }
                    _ => panic!("Type mismatch"),
                }
            }
        }
    }

    // Check file sizes
    {
        println!("\nFile size comparison:");

        // Write uncompressed version for comparison
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema),
        }
        .create("test_files/output_binary.pcd")?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;

        let compressed_size = fs::metadata("test_files/output_compressed.pcd")?.len();
        let binary_size = fs::metadata("test_files/output_binary.pcd")?.len();

        println!("  Binary size: {} bytes", binary_size);
        println!("  Compressed size: {} bytes", compressed_size);
        println!(
            "  Compression ratio: {:.2}%",
            (compressed_size as f64 / binary_size as f64) * 100.0
        );
    }

    println!("\n✅ Test passed! Binary compressed format is working correctly.");

    Ok(())
}
