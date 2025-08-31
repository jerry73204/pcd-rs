use pcd_rs::DynReader;

fn main() -> pcd_rs::Result<()> {
    println!("Testing real PCL bunny.pcd file (version 0.5)...\n");

    let reader = DynReader::open("test_files/bunny_v05.pcd")?;
    let meta = reader.meta().clone();

    println!("Version: {}", meta.version);
    println!("Data format: {:?}", meta.data);
    println!("Points: {}", meta.num_points);
    println!("Width: {}, Height: {}", meta.width, meta.height);
    println!("Fields: {}", meta.field_defs.len());

    for (i, field) in meta.field_defs.iter().enumerate() {
        println!(
            "  Field {}: {} ({:?}, count={})",
            i, field.name, field.kind, field.count
        );
    }

    println!(
        "Viewpoint (should be defaults for v0.5): tx={} ty={} tz={} qw={} qx={} qy={} qz={}",
        meta.viewpoint.tx,
        meta.viewpoint.ty,
        meta.viewpoint.tz,
        meta.viewpoint.qw,
        meta.viewpoint.qx,
        meta.viewpoint.qy,
        meta.viewpoint.qz
    );

    println!("\nReading points...");
    let points: Vec<_> = reader.collect::<Result<_, _>>()?;
    println!("Successfully read {} points", points.len());

    // Print first few points
    for (i, point) in points.iter().take(3).enumerate() {
        println!("Point {}:", i);
        for (j, field) in point.0.iter().enumerate() {
            println!("  {}: {:?}", meta.field_defs[j].name, field);
        }
    }

    println!("\n✅ Successfully read PCL bunny.pcd file with legacy v0.5 support!");
    Ok(())
}
