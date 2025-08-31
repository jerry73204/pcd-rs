//! Tests for binary_compressed PCD format support

use pcd_rs::{DataKind, DynReader, DynRecord, DynWriter, Field, Schema, ValueKind, WriterInit};
use std::io::Cursor;

#[test]
fn test_write_read_compressed() {
    // Create test data
    let points = vec![
        DynRecord(vec![
            Field::F32(vec![1.0]),
            Field::F32(vec![2.0]),
            Field::F32(vec![3.0]),
            Field::U8(vec![255]),
        ]),
        DynRecord(vec![
            Field::F32(vec![4.0]),
            Field::F32(vec![5.0]),
            Field::F32(vec![6.0]),
            Field::U8(vec![128]),
        ]),
        DynRecord(vec![
            Field::F32(vec![7.0]),
            Field::F32(vec![8.0]),
            Field::F32(vec![9.0]),
            Field::U8(vec![0]),
        ]),
    ];

    // Define schema
    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
        ("intensity", ValueKind::U8, 1),
    ]);

    // Write to buffer with compression
    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema.clone()),
        }
        .build_from_writer(cursor)
        .unwrap();

        for point in &points {
            writer.push(point).unwrap();
        }

        writer.finish().unwrap();
    }

    // Read back from compressed buffer
    let cursor = Cursor::new(&buffer);
    let reader = DynReader::from_reader(cursor).unwrap();

    // Verify metadata
    assert_eq!(reader.meta().data, DataKind::BinaryCompressed);
    assert_eq!(reader.meta().num_points, points.len() as u64);

    // Verify points
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>().unwrap();
    assert_eq!(read_points.len(), points.len());

    for (original, read) in points.iter().zip(read_points.iter()) {
        assert_eq!(original.0.len(), read.0.len());
        for (orig_field, read_field) in original.0.iter().zip(read.0.iter()) {
            match (orig_field, read_field) {
                (Field::F32(o), Field::F32(r)) => assert_eq!(o, r),
                (Field::U8(o), Field::U8(r)) => assert_eq!(o, r),
                _ => panic!("Field type mismatch"),
            }
        }
    }
}

#[test]
fn test_compressed_large_data() {
    // Create larger dataset to test compression efficiency
    let num_points = 1000;
    let mut points = Vec::new();

    for i in 0..num_points {
        let x = i as f32 * 0.1;
        let y = (i as f32 * 0.2).sin();
        let z = (i as f32 * 0.3).cos();

        points.push(DynRecord(vec![
            Field::F32(vec![x]),
            Field::F32(vec![y]),
            Field::F32(vec![z]),
        ]));
    }

    let schema = Schema::from_iter([
        ("x", ValueKind::F32, 1),
        ("y", ValueKind::F32, 1),
        ("z", ValueKind::F32, 1),
    ]);

    // Write compressed
    let mut compressed_buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut compressed_buffer);
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema.clone()),
        }
        .build_from_writer(cursor)
        .unwrap();

        for point in &points {
            writer.push(point).unwrap();
        }
        writer.finish().unwrap();
    }

    // Write uncompressed for comparison
    let mut uncompressed_buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut uncompressed_buffer);
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::Binary,
            schema: Some(schema),
        }
        .build_from_writer(cursor)
        .unwrap();

        for point in &points {
            writer.push(point).unwrap();
        }
        writer.finish().unwrap();
    }

    // Compressed should be smaller (in many cases)
    println!("Uncompressed size: {} bytes", uncompressed_buffer.len());
    println!("Compressed size: {} bytes", compressed_buffer.len());
    println!(
        "Compression ratio: {:.2}%",
        (compressed_buffer.len() as f64 / uncompressed_buffer.len() as f64) * 100.0
    );

    // Read and verify compressed data
    let cursor = Cursor::new(&compressed_buffer);
    let reader = DynReader::from_reader(cursor).unwrap();
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>().unwrap();

    assert_eq!(read_points.len(), points.len());

    // Verify first and last points
    assert_eq!(points[0].0.len(), read_points[0].0.len());
    assert_eq!(
        points[num_points - 1].0.len(),
        read_points[num_points - 1].0.len()
    );
}

#[test]
fn test_compressed_with_arrays() {
    // Test with array fields (COUNT > 1)
    let points = vec![
        DynRecord(vec![
            Field::F32(vec![1.0, 2.0, 3.0]), // position array
            Field::F32(vec![0.0, 1.0, 0.0]), // normal array
        ]),
        DynRecord(vec![
            Field::F32(vec![4.0, 5.0, 6.0]),
            Field::F32(vec![1.0, 0.0, 0.0]),
        ]),
    ];

    let schema = Schema::from_iter([
        ("position", ValueKind::F32, 3),
        ("normal", ValueKind::F32, 3),
    ]);

    let mut buffer = Vec::new();
    {
        let cursor = Cursor::new(&mut buffer);
        let mut writer = WriterInit {
            width: points.len() as u64,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
        }
        .build_from_writer(cursor)
        .unwrap();

        for point in &points {
            writer.push(point).unwrap();
        }
        writer.finish().unwrap();
    }

    // Read back
    let cursor = Cursor::new(&buffer);
    let reader = DynReader::from_reader(cursor).unwrap();
    let read_points: Vec<DynRecord> = reader.collect::<Result<_, _>>().unwrap();

    assert_eq!(read_points.len(), points.len());

    // Verify array data
    for (original, read) in points.iter().zip(read_points.iter()) {
        for (orig_field, read_field) in original.0.iter().zip(read.0.iter()) {
            match (orig_field, read_field) {
                (Field::F32(o), Field::F32(r)) => {
                    assert_eq!(o.len(), r.len());
                    for (ov, rv) in o.iter().zip(r.iter()) {
                        assert_eq!(ov, rv);
                    }
                }
                _ => panic!("Unexpected field type"),
            }
        }
    }
}
