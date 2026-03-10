use pcd_rs::{ChunkedReaderBuilder, Field};
use std::collections::HashMap;

fn main() -> pcd_rs::Result<()> {
    println!("Demonstrating Chunked Streaming API\n");

    // Example 1: Basic chunked processing
    println!("=== Basic Chunked Processing ===");
    {
        let mut reader = ChunkedReaderBuilder::new()
            .chunk_size(100)
            .open("test_files/ascii.pcd")?;

        println!("File info:");
        println!("  Points: {}", reader.meta().num_points);
        println!("  Fields: {}", reader.meta().field_defs.len());

        let mut total_points = 0;
        let mut chunk_count = 0;

        reader.for_each_chunk(|chunk| {
            total_points += chunk.points.len();
            chunk_count += 1;

            println!(
                "  Chunk {}: {} points (start_index: {}, is_last: {})",
                chunk.index,
                chunk.points.len(),
                chunk.start_index,
                chunk.is_last
            );

            Ok(())
        })?;

        println!(
            "Processed {} points in {} chunks",
            total_points, chunk_count
        );
    }

    println!();

    // Example 2: Statistical analysis with progress
    println!("=== Statistical Analysis with Progress ===");
    {
        let mut reader = ChunkedReaderBuilder::new()
            .chunk_size(50)
            .open("test_files/ascii.pcd")?;

        // Set progress callback
        reader.set_progress_callback(|progress| {
            if progress.current_chunk % 2 == 0 {
                // Report every 2nd chunk
                println!(
                    "  Progress: {:.1}% ({}/{} chunks, {}/{} points)",
                    progress.percentage,
                    progress.current_chunk,
                    progress.total_chunks,
                    progress.points_processed,
                    progress.total_points
                );
            }
        });

        let mut stats = Statistics::new();

        reader.for_each_chunk(|chunk| {
            // Calculate statistics for this chunk
            for point in &chunk.points {
                if let (Field::F32(x), Field::F32(y), Field::F32(z)) =
                    (&point.0[0], &point.0[1], &point.0[2])
                {
                    stats.update(x[0], y[0], z[0]);
                }
            }
            Ok(())
        })?;

        println!("Final statistics:");
        println!(
            "  Min: ({:.3}, {:.3}, {:.3})",
            stats.min.0, stats.min.1, stats.min.2
        );
        println!(
            "  Max: ({:.3}, {:.3}, {:.3})",
            stats.max.0, stats.max.1, stats.max.2
        );
        println!(
            "  Avg: ({:.3}, {:.3}, {:.3})",
            stats.avg.0, stats.avg.1, stats.avg.2
        );
    }

    println!();

    // Example 3: Filtering and transformation
    println!("=== Filtering and Transformation ===");
    {
        let mut reader = ChunkedReaderBuilder::new()
            .chunk_size(75)
            .open("test_files/ascii.pcd")?;

        let mut filtered_count = 0;
        let mut transformed_points = Vec::new();

        reader.for_each_chunk(|chunk| {
            // Filter points based on some criteria (e.g., z > 0)
            // Transform coordinates (e.g., scale by 2)
            for point in chunk.points {
                if let (Field::F32(x), Field::F32(y), Field::F32(z)) =
                    (&point.0[0], &point.0[1], &point.0[2])
                {
                    if z[0] > 0.0 {
                        // Filter condition
                        filtered_count += 1;

                        // Transform: scale by 2 and shift
                        let transformed = (x[0] * 2.0 + 1.0, y[0] * 2.0 + 1.0, z[0] * 2.0 + 1.0);

                        if transformed_points.len() < 5 {
                            // Keep only first 5 for demo
                            transformed_points.push(transformed);
                        }
                    }
                }
            }
            Ok(())
        })?;

        println!("Filter results:");
        println!(
            "  Filtered points: {} / {}",
            filtered_count,
            reader.meta().num_points
        );
        println!("  Sample transformed points:");
        for (i, point) in transformed_points.iter().enumerate() {
            println!(
                "    {}: ({:.3}, {:.3}, {:.3})",
                i + 1,
                point.0,
                point.1,
                point.2
            );
        }
    }

    println!();

    // Example 4: Histogram/Binning
    println!("=== Spatial Binning (Histogram) ===");
    {
        let mut reader = ChunkedReaderBuilder::new()
            .chunk_size(100)
            .open("test_files/ascii.pcd")?;

        let mut z_histogram: HashMap<i32, usize> = HashMap::new();

        reader.for_each_chunk(|chunk| {
            for point in chunk.points {
                if let Field::F32(z) = &point.0[2] {
                    // Bin z values into 0.01 intervals
                    let bin = (z[0] * 100.0) as i32;
                    *z_histogram.entry(bin).or_insert(0) += 1;
                }
            }
            Ok(())
        })?;

        println!("Z-coordinate histogram (showing top 5 bins):");
        let mut sorted_bins: Vec<_> = z_histogram.iter().collect();
        sorted_bins.sort_by(|a, b| b.1.cmp(a.1));

        for (bin, count) in sorted_bins.iter().take(5) {
            let z_value = **bin as f32 / 100.0;
            println!("  z~={:.2}: {} points", z_value, count);
        }
    }

    println!();

    // Example 5: Chunk mapping (collect results)
    println!("=== Chunk Mapping (Collecting Results) ===");
    {
        let mut reader = ChunkedReaderBuilder::new()
            .chunk_size(100)
            .open("test_files/ascii.pcd")?;

        let chunk_summaries = reader.map_chunks(|chunk| {
            // Calculate summary for each chunk
            let mut min_z = f32::INFINITY;
            let mut max_z = f32::NEG_INFINITY;
            let mut sum_z = 0.0;

            for point in &chunk.points {
                if let Field::F32(z) = &point.0[2] {
                    min_z = min_z.min(z[0]);
                    max_z = max_z.max(z[0]);
                    sum_z += z[0];
                }
            }

            let avg_z = sum_z / chunk.points.len() as f32;

            Ok(ChunkSummary {
                index: chunk.index,
                count: chunk.points.len(),
                min_z,
                max_z,
                avg_z,
            })
        })?;

        println!("Per-chunk summaries:");
        for summary in chunk_summaries.iter().take(5) {
            // Show first 5
            println!(
                "  Chunk {}: {} pts, z in [{:.3}, {:.3}], avg_z={:.3}",
                summary.index, summary.count, summary.min_z, summary.max_z, summary.avg_z
            );
        }
        println!("  ... (total {} chunks)", chunk_summaries.len());
    }

    println!("\nChunked streaming examples completed!");
    println!("Key benefits demonstrated:");
    println!("  - Memory-bounded processing (configurable chunk size)");
    println!("  - Progress callbacks for long operations");
    println!("  - Works with ALL PCD formats (ASCII, binary, compressed)");
    println!("  - Natural parallelism boundaries");
    println!("  - Streaming transformations and filtering");

    Ok(())
}

#[derive(Debug)]
struct Statistics {
    min: (f32, f32, f32),
    max: (f32, f32, f32),
    sum: (f64, f64, f64),
    count: usize,
    avg: (f32, f32, f32),
}

impl Statistics {
    fn new() -> Self {
        Self {
            min: (f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: (f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            sum: (0.0, 0.0, 0.0),
            count: 0,
            avg: (0.0, 0.0, 0.0),
        }
    }

    fn update(&mut self, x: f32, y: f32, z: f32) {
        self.min.0 = self.min.0.min(x);
        self.min.1 = self.min.1.min(y);
        self.min.2 = self.min.2.min(z);

        self.max.0 = self.max.0.max(x);
        self.max.1 = self.max.1.max(y);
        self.max.2 = self.max.2.max(z);

        self.sum.0 += x as f64;
        self.sum.1 += y as f64;
        self.sum.2 += z as f64;

        self.count += 1;

        let _count_f = self.count as f32;
        self.avg.0 = (self.sum.0 / self.count as f64) as f32;
        self.avg.1 = (self.sum.1 / self.count as f64) as f32;
        self.avg.2 = (self.sum.2 / self.count as f64) as f32;
    }
}

#[derive(Debug)]
struct ChunkSummary {
    index: usize,
    count: usize,
    min_z: f32,
    max_z: f32,
    avg_z: f32,
}
