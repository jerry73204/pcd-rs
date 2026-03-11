use pcd_rs::{DataKind, DynRecord, Field, Schema, ValueKind, WriterInit};

fn main() -> pcd_rs::Result<()> {
    println!("Demonstrating PCD Legacy Writer Support\n");

    // Create test data
    let points = vec![
        DynRecord(vec![
            Field::F32(vec![1.0]),
            Field::F32(vec![2.0]),
            Field::F32(vec![3.0]),
        ]),
        DynRecord(vec![
            Field::F32(vec![4.0]),
            Field::F32(vec![5.0]),
            Field::F32(vec![6.0]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    // Write PCD v0.5 file (no VIEWPOINT field)
    println!("Writing PCD v0.5 file (ASCII format)...");
    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Ascii,
            schema: Some(schema.clone()),
            version: Some("0.5".to_string()),
        }
        .create("pcd-rs/test_files/demo_v05.pcd")?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Write PCD v0.6 file (no VIEWPOINT field, binary format)
    println!("Writing PCD v0.6 file (Binary format)...");
    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema.clone()),
            version: Some("0.6".to_string()),
        }
        .create("pcd-rs/test_files/demo_v06.pcd")?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Write PCD v0.7 file (includes VIEWPOINT field, can use compressed)
    println!("Writing PCD v0.7 file (Binary format with VIEWPOINT)...");
    {
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema.clone()),
            version: Some("0.7".to_string()),
        }
        .create("pcd-rs/test_files/demo_v07.pcd")?;

        for point in &points {
            writer.push(point)?;
        }
        writer.finish()?;
    }

    // Try to write compressed with legacy version (should fail)
    println!("\nTrying to write compressed PCD with legacy version (should fail)...");
    let result = WriterInit {
        width: 1,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::BinaryCompressed,
        schema: Some(schema),
        version: Some("0.5".to_string()),
    }
    .create::<DynRecord, _>("pcd-rs/test_files/demo_v05_compressed_fail.pcd");

    match result {
        Ok(_) => println!("ERROR: Should have failed!"),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("\nLegacy writer demo completed!");
    println!("Created files:");
    println!("  - test_files/demo_v05.pcd (v0.5, ASCII, no VIEWPOINT)");
    println!("  - test_files/demo_v06.pcd (v0.6, Binary, no VIEWPOINT)");
    println!("  - test_files/demo_v07.pcd (v0.7, Binary, with VIEWPOINT)");

    Ok(())
}
