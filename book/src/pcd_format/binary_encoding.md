# Binary Encoding

This chapter explains how PCD data is encoded in binary format for efficient storage and processing.

## Binary Format Overview

Binary encoding in PCD files provides:
- **Efficiency**: Direct memory representation
- **Speed**: No parsing overhead
- **Precision**: Exact floating-point values
- **Compactness**: Smaller file sizes than ASCII

## Data Layout

### Point Structure

Points are stored sequentially with fields packed tightly:

```
Point 1: [field1][field2][field3]...
Point 2: [field1][field2][field3]...
Point 3: [field1][field2][field3]...
...
```

### Field Packing

No padding between fields:

```
x:f32 y:f32 z:f32 rgb:u32
[4 bytes][4 bytes][4 bytes][4 bytes] = 16 bytes/point
```

## Encoding Examples

### Simple XYZ Point

Header:
```
FIELDS x y z
SIZE 4 4 4
TYPE F F F
```

Binary layout (12 bytes per point):
```
Offset  Size  Field  Type
0       4     x      f32
4       4     y      f32
8       4     z      f32
```

Rust representation:
```rust
#[repr(C, packed)]
struct XYZPoint {
    x: f32,
    y: f32,
    z: f32,
}
```

### Complex Point

Header:
```
FIELDS x y z rgb normal_x normal_y normal_z curvature
SIZE 4 4 4 4 4 4 4 4
TYPE F F F U F F F F
```

Binary layout (32 bytes per point):
```
Offset  Size  Field      Type
0       4     x          f32
4       4     y          f32
8       4     z          f32
12      4     rgb        u32
16      4     normal_x   f32
20      4     normal_y   f32
24      4     normal_z   f32
28      4     curvature  f32
```

## Reading Binary Data

### Direct Reading

```rust
use byteorder::{NativeEndian, ReadBytesExt};
use std::io::Read;

fn read_binary_point<R: Read>(reader: &mut R) -> Result<Point> {
    let x = reader.read_f32::<NativeEndian>()?;
    let y = reader.read_f32::<NativeEndian>()?;
    let z = reader.read_f32::<NativeEndian>()?;
    let rgb = reader.read_u32::<NativeEndian>()?;
    
    Ok(Point { x, y, z, rgb })
}
```

### Buffer-based Reading

```rust
fn read_point_from_buffer(buffer: &[u8]) -> Result<Point> {
    if buffer.len() < 16 {
        return Err(Error::InsufficientData);
    }
    
    let x = f32::from_ne_bytes([
        buffer[0], buffer[1], buffer[2], buffer[3]
    ]);
    let y = f32::from_ne_bytes([
        buffer[4], buffer[5], buffer[6], buffer[7]
    ]);
    let z = f32::from_ne_bytes([
        buffer[8], buffer[9], buffer[10], buffer[11]
    ]);
    let rgb = u32::from_ne_bytes([
        buffer[12], buffer[13], buffer[14], buffer[15]
    ]);
    
    Ok(Point { x, y, z, rgb })
}
```

## Writing Binary Data

### Direct Writing

```rust
use byteorder::{NativeEndian, WriteBytesExt};
use std::io::Write;

fn write_binary_point<W: Write>(writer: &mut W, point: &Point) -> Result<()> {
    writer.write_f32::<NativeEndian>(point.x)?;
    writer.write_f32::<NativeEndian>(point.y)?;
    writer.write_f32::<NativeEndian>(point.z)?;
    writer.write_u32::<NativeEndian>(point.rgb)?;
    
    Ok(())
}
```

### Buffer-based Writing

```rust
fn write_point_to_buffer(point: &Point, buffer: &mut Vec<u8>) {
    buffer.extend_from_slice(&point.x.to_ne_bytes());
    buffer.extend_from_slice(&point.y.to_ne_bytes());
    buffer.extend_from_slice(&point.z.to_ne_bytes());
    buffer.extend_from_slice(&point.rgb.to_ne_bytes());
}
```

## Array Fields Encoding

### Fixed-size Arrays

Header:
```
FIELDS position normal
SIZE 4 4
TYPE F F
COUNT 3 3
```

Binary layout:
```
position: [x:f32][y:f32][z:f32]
normal:   [nx:f32][ny:f32][nz:f32]
Total: 24 bytes
```

### Variable-size Arrays

```rust
// Reading array field
fn read_array_field<R: Read>(reader: &mut R, count: usize) -> Result<Vec<f32>> {
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(reader.read_f32::<NativeEndian>()?);
    }
    Ok(values)
}
```

## Endianness

### Native Endian (Default)

PCD files use the system's native endianness:

```rust
// Most systems are little-endian
#[cfg(target_endian = "little")]
fn read_f32(bytes: [u8; 4]) -> f32 {
    f32::from_le_bytes(bytes)
}

#[cfg(target_endian = "big")]
fn read_f32(bytes: [u8; 4]) -> f32 {
    f32::from_be_bytes(bytes)
}
```

### Cross-platform Considerations

```rust
use byteorder::NativeEndian;

// Always use NativeEndian for PCD files
type PcdEndian = NativeEndian;

// This ensures correct behavior on any platform
fn read_pcd_float<R: Read>(reader: &mut R) -> Result<f32> {
    reader.read_f32::<PcdEndian>()
}
```

## Compressed Binary Format

### LZF Compression

The compressed format uses LZF algorithm:

```
Header: [compressed_size:u32][uncompressed_size:u32]
Data: [compressed_bytes...]
```

### Decompression Process

```rust
fn decompress_lzf(compressed: &[u8]) -> Result<Vec<u8>> {
    if compressed.len() < 8 {
        return Err(Error::InvalidCompressedData);
    }
    
    // Read sizes
    let compressed_size = u32::from_le_bytes([
        compressed[0], compressed[1], compressed[2], compressed[3]
    ]);
    let uncompressed_size = u32::from_le_bytes([
        compressed[4], compressed[5], compressed[6], compressed[7]
    ]);
    
    // Decompress data
    let compressed_data = &compressed[8..8 + compressed_size as usize];
    let mut output = vec![0u8; uncompressed_size as usize];
    
    lzf::decompress(compressed_data, &mut output)?;
    
    Ok(output)
}
```

## Performance Optimization

### Memory Mapping (Future)

```rust
use memmap2::MmapOptions;
use std::fs::File;

fn memory_map_binary_pcd(path: &Path) -> Result<PointCloud> {
    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    
    // Parse header to find data offset
    let data_offset = parse_header_offset(&mmap)?;
    
    // Direct access to binary data
    let data = &mmap[data_offset..];
    
    // Zero-copy point access
    interpret_as_points(data)
}
```

### Vectorized Reading

```rust
fn read_points_vectorized(buffer: &[u8], count: usize) -> Vec<Point> {
    const POINT_SIZE: usize = 16; // Size of Point struct
    
    let mut points = Vec::with_capacity(count);
    
    for chunk in buffer.chunks_exact(POINT_SIZE) {
        // Process multiple points in parallel
        let point = unsafe {
            std::ptr::read_unaligned(chunk.as_ptr() as *const Point)
        };
        points.push(point);
    }
    
    points
}
```

## Binary Format Validation

### Checksum Verification

```rust
fn validate_binary_data(data: &[u8], expected_points: usize, point_size: usize) -> Result<()> {
    let expected_size = expected_points * point_size;
    
    if data.len() != expected_size {
        return Err(Error::InvalidDataSize {
            expected: expected_size,
            actual: data.len(),
        });
    }
    
    // Optional: Check for NaN/Inf in floating point data
    for chunk in data.chunks_exact(4) {
        let value = f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        if !value.is_finite() && !value.is_nan() {
            // Handle special values
        }
    }
    
    Ok(())
}
```

## Common Issues and Solutions

### Alignment Issues

```rust
// Wrong: May cause alignment issues
#[repr(C)]
struct BadPoint {
    x: f32,
    label: u8,  // Causes padding
    y: f32,
}

// Correct: Properly packed
#[repr(C, packed)]
struct GoodPoint {
    x: f32,
    y: f32,
    z: f32,
    label: u8,
}
```

### Buffer Overflow Prevention

```rust
fn safe_read_point(buffer: &[u8], offset: usize) -> Option<Point> {
    const POINT_SIZE: usize = 16;
    
    if offset + POINT_SIZE > buffer.len() {
        return None;
    }
    
    Some(read_point_from_buffer(&buffer[offset..offset + POINT_SIZE]))
}
```

## Best Practices

1. **Use buffered I/O**: Reduce system calls
2. **Pre-allocate buffers**: Known sizes improve performance
3. **Validate data sizes**: Prevent buffer overflows
4. **Handle endianness**: Use `NativeEndian` consistently
5. **Check special values**: Handle NaN/Inf appropriately
6. **Optimize for cache**: Read sequential data
7. **Consider memory mapping**: For very large files