//! Example program to read binary_compressed PCD files

use pcd_rs::{DataKind, DynReader};
use std::path::Path;

fn main() -> pcd_rs::Result<()> {
    // Path to the compressed PCD file
    let path = Path::new("test_files/test_compressed.pcd");

    println!("Reading compressed PCD file: {:?}", path);

    // Open the reader
    let reader = DynReader::open(path)?;

    // Print metadata
    let meta = reader.meta();
    println!("\nFile Metadata:");
    println!("  Version: {}", meta.version);
    println!("  Data format: {:?}", meta.data);
    println!("  Dimensions: {} x {}", meta.width, meta.height);
    println!("  Total points: {}", meta.num_points);
    println!(
        "  Fields: {:?}",
        meta.field_defs
            .fields
            .iter()
            .map(|f| &f.name)
            .collect::<Vec<_>>()
    );

    // Verify it's compressed
    assert_eq!(
        meta.data,
        DataKind::BinaryCompressed,
        "File should be binary_compressed"
    );

    // Read first few points
    println!("\nReading first 5 points...");
    let mut count = 0;
    for (i, point_result) in reader.enumerate() {
        match point_result {
            Ok(point) => {
                if i < 5 {
                    println!("Point {}: {} fields", i, point.0.len());
                    // Print first 3 fields (usually x, y, z)
                    for (j, field) in point.0.iter().take(3).enumerate() {
                        print!("  Field {}: ", j);
                        match field {
                            pcd_rs::Field::F32(vals) => println!("F32 = {:?}", vals),
                            pcd_rs::Field::F64(vals) => println!("F64 = {:?}", vals),
                            pcd_rs::Field::I8(vals) => println!("I8 = {:?}", vals),
                            pcd_rs::Field::I16(vals) => println!("I16 = {:?}", vals),
                            pcd_rs::Field::I32(vals) => println!("I32 = {:?}", vals),
                            pcd_rs::Field::U8(vals) => println!("U8 = {:?}", vals),
                            pcd_rs::Field::U16(vals) => println!("U16 = {:?}", vals),
                            pcd_rs::Field::U32(vals) => println!("U32 = {:?}", vals),
                        }
                    }
                }
                count += 1;
            }
            Err(e) => {
                eprintln!("Error reading point {}: {}", i, e);
                break;
            }
        }

        // Stop after 100 points for testing
        if i >= 99 {
            break;
        }
    }

    println!("\nSuccessfully read {} points", count);

    Ok(())
}
