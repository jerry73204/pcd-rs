use pcd_rs::{ChunkedReaderBuilder, Field};
use std::sync::{Arc, Mutex};

#[test]
fn test_basic_chunked_reading() -> pcd_rs::Result<()> {
    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(100)
        .open("test_files/ascii.pcd")?;

    let mut total_points = 0;
    let mut chunk_count = 0;
    let meta_points = reader.meta().num_points;

    reader.for_each_chunk(|chunk| {
        assert!(
            chunk.points.len() <= 100,
            "Chunk size exceeds configured limit"
        );
        assert_eq!(
            chunk.index, chunk_count,
            "Chunk index should match iteration count"
        );
        assert_eq!(
            chunk.start_index, total_points,
            "Start index should match total processed"
        );

        total_points += chunk.points.len();
        chunk_count += 1;

        // Check if is_last flag is correct
        if chunk.is_last {
            assert_eq!(
                total_points, meta_points as usize,
                "Last chunk should complete all points"
            );
        }

        Ok(())
    })?;

    assert_eq!(
        total_points, meta_points as usize,
        "Should process all points"
    );
    assert!(chunk_count > 0, "Should have at least one chunk");

    Ok(())
}

#[test]
fn test_different_chunk_sizes() -> pcd_rs::Result<()> {
    let chunk_sizes = [1, 5, 50, 100, 500, 10000];

    for &chunk_size in &chunk_sizes {
        let mut reader = ChunkedReaderBuilder::new()
            .chunk_size(chunk_size)
            .open("test_files/ascii.pcd")?;

        let meta_points = reader.meta().num_points as usize;
        let mut total_points = 0;
        let mut chunks = Vec::new();

        reader.for_each_chunk(|chunk| {
            chunks.push(chunk.points.len());
            total_points += chunk.points.len();

            // All chunks except potentially the last should be exactly chunk_size
            if !chunk.is_last {
                assert_eq!(
                    chunk.points.len(),
                    chunk_size,
                    "Non-last chunk should be exactly chunk_size for size {}",
                    chunk_size
                );
            }

            Ok(())
        })?;

        assert_eq!(
            total_points, meta_points,
            "Should process all points for chunk_size {}",
            chunk_size
        );

        // Verify chunk count calculation
        let expected_chunks = (meta_points + chunk_size - 1) / chunk_size;
        assert_eq!(
            chunks.len(),
            expected_chunks,
            "Should have correct number of chunks for size {}",
            chunk_size
        );
    }

    Ok(())
}

#[test]
fn test_progress_callbacks() -> pcd_rs::Result<()> {
    let progress_updates = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = Arc::clone(&progress_updates);

    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(50)
        .open("test_files/ascii.pcd")?;

    let total_points = reader.meta().num_points as usize;

    reader.set_progress_callback(move |progress| {
        let mut updates = progress_clone.lock().unwrap();
        updates.push((
            progress.current_chunk,
            progress.points_processed,
            progress.percentage,
        ));

        // Validate progress values
        assert!(progress.percentage >= 0.0 && progress.percentage <= 100.0);
        assert!(progress.points_processed <= progress.total_points);
        assert!(progress.current_chunk <= progress.total_chunks); // <= because current_chunk is 1-based after increment
        assert_eq!(progress.total_points, total_points);
    });

    reader.for_each_chunk(|_chunk| Ok(()))?;

    let updates = progress_updates.lock().unwrap();
    assert!(!updates.is_empty(), "Should have received progress updates");

    // Check that progress is monotonically increasing
    for window in updates.windows(2) {
        let (chunk1, points1, pct1) = window[0];
        let (chunk2, points2, pct2) = window[1];

        assert!(chunk2 > chunk1, "Chunk index should increase");
        assert!(points2 >= points1, "Points processed should increase");
        assert!(pct2 >= pct1, "Percentage should increase");
    }

    // Last update should be 100% or close to it
    let last_update = updates.last().unwrap();
    assert!(last_update.2 >= 99.0, "Final progress should be near 100%");

    Ok(())
}

#[test]
fn test_chunk_mapping() -> pcd_rs::Result<()> {
    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(100)
        .open("test_files/ascii.pcd")?;

    let chunk_summaries = reader.map_chunks(|chunk| {
        let point_count = chunk.points.len();
        let mut z_sum = 0.0;
        let mut valid_points = 0;

        for point in &chunk.points {
            if let Some(Field::F32(z)) = point.0.get(2) {
                z_sum += z[0];
                valid_points += 1;
            }
        }

        let avg_z = if valid_points > 0 {
            z_sum / valid_points as f32
        } else {
            0.0
        };

        Ok((chunk.index, point_count, avg_z))
    })?;

    assert!(!chunk_summaries.is_empty(), "Should have chunk summaries");

    // Verify chunk indices are sequential
    for (i, (chunk_index, _count, _avg)) in chunk_summaries.iter().enumerate() {
        assert_eq!(*chunk_index, i, "Chunk indices should be sequential");
    }

    // Verify total point count
    let total_mapped: usize = chunk_summaries.iter().map(|(_, count, _)| count).sum();
    let expected_total = reader.meta().num_points as usize;
    assert_eq!(
        total_mapped, expected_total,
        "Mapped points should equal total points"
    );

    Ok(())
}

#[test]
fn test_cancellation() -> pcd_rs::Result<()> {
    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(10)
        .open("test_files/ascii.pcd")?;

    let chunks_processed = Arc::new(Mutex::new(0));
    let chunks_clone = Arc::clone(&chunks_processed);

    // Cancel the reader after 3 chunks are processed
    reader
        .for_each_chunk(|_chunk| {
            let mut count = chunks_clone.lock().unwrap();
            *count += 1;

            // Cancel after processing 3 chunks by returning early
            if *count >= 3 {
                // We can't cancel from within the closure due to borrowing rules,
                // so we'll just return an error to stop processing
                return Err(pcd_rs::Error::new_parse_error(0, "Cancelled"));
            }

            Ok(())
        })
        .ok(); // Ignore the error since we're testing cancellation

    let final_count = *chunks_processed.lock().unwrap();
    assert_eq!(
        final_count, 3,
        "Should process exactly 3 chunks before stopping"
    );

    Ok(())
}

#[test]
fn test_empty_chunk_handling() -> pcd_rs::Result<()> {
    // Test with chunk size larger than file
    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(100000) // Much larger than any test file
        .open("test_files/ascii.pcd")?;

    let mut chunk_count = 0;
    let meta_points = reader.meta().num_points as usize;

    reader.for_each_chunk(|chunk| {
        chunk_count += 1;
        assert_eq!(
            chunk.points.len(),
            meta_points,
            "Single chunk should contain all points"
        );
        assert_eq!(chunk.index, 0, "Should be chunk 0");
        assert!(chunk.is_last, "Should be marked as last chunk");
        Ok(())
    })?;

    assert_eq!(chunk_count, 1, "Should have exactly one chunk");

    Ok(())
}

#[test]
fn test_error_handling_in_chunks() -> pcd_rs::Result<()> {
    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(50)
        .open("test_files/ascii.pcd")?;

    let mut chunks_processed = 0;

    // Test that errors in chunk processing are propagated
    let result = reader.for_each_chunk(|_chunk| {
        chunks_processed += 1;

        if chunks_processed == 2 {
            return Err(pcd_rs::Error::new_parse_error(0, "Test error"));
        }

        Ok(())
    });

    assert!(
        result.is_err(),
        "Error in chunk processing should be propagated"
    );
    assert_eq!(
        chunks_processed, 2,
        "Should have processed 2 chunks before error"
    );

    Ok(())
}

#[test]
fn test_different_file_formats() -> pcd_rs::Result<()> {
    let test_files = ["test_files/ascii.pcd", "test_files/binary.pcd"];

    for file_path in &test_files {
        if std::path::Path::new(file_path).exists() {
            let mut reader = ChunkedReaderBuilder::new()
                .chunk_size(100)
                .open(file_path)?;

            let meta_points = reader.meta().num_points as usize;
            let mut total_points = 0;

            reader.for_each_chunk(|chunk| {
                total_points += chunk.points.len();

                // Verify that each point has fields (we can't access reader.meta() from closure)
                for point in &chunk.points {
                    assert!(!point.0.is_empty(), "Point should have at least one field");
                }

                Ok(())
            })?;

            assert_eq!(
                total_points, meta_points,
                "Should process all points for file {}",
                file_path
            );
        }
    }

    Ok(())
}

#[test]
fn test_chunk_size_edge_cases() -> pcd_rs::Result<()> {
    // Test chunk size of 1
    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(1)
        .open("test_files/ascii.pcd")?;

    let meta_points = reader.meta().num_points as usize;
    let mut chunk_count = 0;

    reader.for_each_chunk(|chunk| {
        chunk_count += 1;
        assert_eq!(
            chunk.points.len(),
            1,
            "Each chunk should have exactly 1 point"
        );
        Ok(())
    })?;

    assert_eq!(chunk_count, meta_points, "Should have one chunk per point");

    Ok(())
}

#[test]
fn test_builder_configuration() -> pcd_rs::Result<()> {
    // Test builder pattern with all configuration options
    let reader = ChunkedReaderBuilder::new()
        .chunk_size(200)
        .parallel(false)
        .buffer_chunks(5)
        .open("test_files/ascii.pcd")?;

    // Just verify that the builder created a valid reader
    assert_eq!(reader.meta().num_points > 0, true);

    // Test with different configurations
    let reader2 = ChunkedReaderBuilder::default()
        .chunk_size(500)
        .open("test_files/ascii.pcd")?;

    assert_eq!(reader2.meta().num_points > 0, true);

    Ok(())
}

#[cfg(feature = "parallel")]
#[test]
fn test_parallel_chunk_processing() -> pcd_rs::Result<()> {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Instant,
    };

    let processed_chunks = Arc::new(AtomicUsize::new(0));
    let processed_clone = Arc::clone(&processed_chunks);

    let start = Instant::now();

    let mut reader = ChunkedReaderBuilder::new()
        .chunk_size(100)
        .parallel(true)
        .max_threads(4)
        .open("test_files/ascii.pcd")?;

    reader.for_each_chunk(move |_chunk| {
        processed_clone.fetch_add(1, Ordering::Relaxed);

        // Simulate some processing time
        std::thread::sleep(std::time::Duration::from_millis(1));

        Ok(())
    })?;

    let elapsed = start.elapsed();
    let total_chunks = processed_chunks.load(Ordering::Relaxed);

    println!("Processed {} chunks in {:?}", total_chunks, elapsed);
    assert!(total_chunks > 0, "Should process some chunks");

    Ok(())
}
