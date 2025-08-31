use pcd_rs::{DynReader, Field};

fn main() -> pcd_rs::Result<()> {
    println!("Testing legacy PCD version support...\n");

    // Test version 0.5 (ASCII)
    println!("=== Testing PCD version 0.5 ===");
    {
        let reader = DynReader::open("test_files/legacy_v05.pcd")?;
        let meta = reader.meta();

        println!("Version: {}", meta.version);
        println!("Data format: {:?}", meta.data);
        println!("Points: {}", meta.num_points);
        println!("Fields: {}", meta.field_defs.len());

        // Check that viewpoint uses defaults for v0.5
        println!(
            "Viewpoint: tx={} ty={} tz={} qw={} qx={} qy={} qz={}",
            meta.viewpoint.tx,
            meta.viewpoint.ty,
            meta.viewpoint.tz,
            meta.viewpoint.qw,
            meta.viewpoint.qx,
            meta.viewpoint.qy,
            meta.viewpoint.qz
        );

        let points: Vec<_> = reader.collect::<Result<_, _>>()?;
        println!("Successfully read {} points", points.len());

        // Print first point
        if let Some(point) = points.first() {
            println!("First point:");
            for (i, field) in point.0.iter().enumerate() {
                match field {
                    Field::F32(values) => println!("  Field {}: {:?}", i, values),
                    _ => println!("  Field {}: {:?}", i, field),
                }
            }
        }
    }

    println!();

    // Test version 0.6 (Binary)
    println!("=== Testing PCD version 0.6 ===");
    {
        let reader = DynReader::open("test_files/legacy_v06.pcd")?;
        let meta = reader.meta();

        println!("Version: {}", meta.version);
        println!("Data format: {:?}", meta.data);
        println!("Points: {}", meta.num_points);
        println!("Fields: {}", meta.field_defs.len());

        // Check that viewpoint uses defaults for v0.6
        println!(
            "Viewpoint: tx={} ty={} tz={} qw={} qx={} qy={} qz={}",
            meta.viewpoint.tx,
            meta.viewpoint.ty,
            meta.viewpoint.tz,
            meta.viewpoint.qw,
            meta.viewpoint.qx,
            meta.viewpoint.qy,
            meta.viewpoint.qz
        );

        let points: Vec<_> = reader.collect::<Result<_, _>>()?;
        println!("Successfully read {} points", points.len());

        // Print first point
        if let Some(point) = points.first() {
            println!("First point:");
            for (i, field) in point.0.iter().enumerate() {
                match field {
                    Field::F32(values) => println!("  Field {}: {:?}", i, values),
                    Field::U8(values) => println!("  Field {}: {:?}", i, values),
                    _ => println!("  Field {}: {:?}", i, field),
                }
            }
        }
    }

    println!("\n✅ All legacy version tests passed!");
    Ok(())
}
