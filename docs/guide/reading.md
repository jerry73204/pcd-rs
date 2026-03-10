# Reading PCD Files

This chapter covers various techniques for reading PCD files with pcd-rs.

## Basic Reading

### Opening Files

```rust
use pcd_rs::DynReader;

// From file path
let reader = DynReader::open("pointcloud.pcd")?;

// From any Read + Seek source
let file = File::open("pointcloud.pcd")?;
let reader = DynReader::from_reader(file)?;

// From memory buffer
let buffer = Cursor::new(pcd_bytes);
let reader = DynReader::from_reader(buffer)?;
```

## Inspecting Metadata

Before reading points, inspect the file metadata:

```rust
let reader = DynReader::open("pointcloud.pcd")?;
let meta = reader.meta();

println!("PCD Version: {}", meta.version);
println!("Dimensions: {}x{}", meta.width, meta.height);
println!("Total points: {}", meta.num_points());
println!("Data format: {:?}", meta.data_kind);

// Inspect schema
for field in &meta.schema.fields {
    println!("Field: {} (type: {:?}, count: {})", 
        field.name, field.value_kind, field.count);
}
```

## Iteration Patterns

### Basic Iteration

```rust
let reader = DynReader::open("pointcloud.pcd")?;

for point in reader {
    let point = point?;
    // Process point
}
```

### Collecting All Points

```rust
let reader = DynReader::open("pointcloud.pcd")?;
let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;
```

### Limited Reading

```rust
// Read only first 100 points
let reader = DynReader::open("pointcloud.pcd")?;
let points: Vec<DynRecord> = reader
    .take(100)
    .collect::<Result<_, _>>()?;
```

### Filtering Points

```rust
let reader = DynReader::open("pointcloud.pcd")?;

// Filter points based on criteria
let filtered: Vec<DynRecord> = reader
    .filter_map(|result| {
        result.ok().filter(|point| {
            // Example: keep points with positive x
            if let Some(x) = point.to_xyz().ok().map(|xyz| xyz.0) {
                x > 0.0
            } else {
                false
            }
        })
    })
    .collect();
```

## Working with Fields

### Accessing Field Data

```rust
let reader = DynReader::open("pointcloud.pcd")?;

for point in reader {
    let point = point?;
    
    // Access by index
    let first_field = &point.0[0];
    
    // Extract specific types
    if let Field::F32(values) = first_field {
        println!("X coordinate: {}", values[0]);
    }
}
```

### Common Field Extraction

```rust
// Extract XYZ coordinates
let (x, y, z) = point.to_xyz()?;

// Manual field extraction
fn extract_rgb(record: &DynRecord) -> Option<(u8, u8, u8)> {
    // Assuming fields 3, 4, 5 are RGB
    match (&record.0[3], &record.0[4], &record.0[5]) {
        (Field::U8(r), Field::U8(g), Field::U8(b)) => {
            Some((r[0], g[0], b[0]))
        }
        _ => None,
    }
}
```

## Static Type Reading

### Defining Point Types

```rust
use pcd_rs::{PcdDeserialize, Reader};

#[derive(Debug, PcdDeserialize)]
struct PointXYZ {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, PcdDeserialize)]
struct PointXYZRGB {
    x: f32,
    y: f32,
    z: f32,
    r: u8,
    g: u8,
    b: u8,
}
```

### Type-Safe Reading

```rust
// Read with specific type
let reader: Reader<PointXYZ> = Reader::open("pointcloud.pcd")?;

for point in reader {
    let point: PointXYZ = point?;
    println!("Point: ({}, {}, {})", point.x, point.y, point.z);
}
```

### Handling Arrays

```rust
#[derive(PcdDeserialize)]
struct PointWithNormal {
    position: [f32; 3],  // XYZ as array
    normal: [f32; 3],    // Normal vector
    curvature: f32,
}
```

## Error Handling

### Graceful Error Recovery

```rust
let reader = DynReader::open("pointcloud.pcd")?;

let valid_points: Vec<DynRecord> = reader
    .filter_map(|result| {
        match result {
            Ok(point) => Some(point),
            Err(e) => {
                eprintln!("Failed to read point: {}", e);
                None
            }
        }
    })
    .collect();
```

### Common Errors

```rust
use pcd_rs::Error;

match DynReader::open("pointcloud.pcd") {
    Ok(reader) => { /* process */ },
    Err(Error::IoError(e)) => {
        eprintln!("I/O error: {}", e);
    },
    Err(Error::InvalidHeader(msg)) => {
        eprintln!("Invalid header: {}", msg);
    },
    Err(Error::SchemaError(msg)) => {
        eprintln!("Schema error: {}", msg);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    },
}
```

## Performance Tips

### Buffering

```rust
use std::io::BufReader;

// Explicit buffering for non-file sources
let file = File::open("large.pcd")?;
let buffered = BufReader::with_capacity(8192, file);
let reader = DynReader::from_reader(buffered)?;
```

### Parallel Processing

```rust
use rayon::prelude::*;

// Collect points first
let reader = DynReader::open("pointcloud.pcd")?;
let points: Vec<DynRecord> = reader.collect::<Result<_, _>>()?;

// Process in parallel
let results: Vec<_> = points
    .par_iter()
    .map(|point| {
        // Heavy computation per point
        process_point(point)
    })
    .collect();
```

### Memory Management

```rust
// Process in chunks to limit memory usage
let reader = DynReader::open("huge.pcd")?;
let chunk_size = 10000;

for chunk in &reader.chunks(chunk_size) {
    let points: Vec<DynRecord> = chunk.collect::<Result<_, _>>()?;
    process_chunk(&points);
    // Chunk is dropped here, freeing memory
}
```

## Advanced Techniques

### Custom Deserializers

```rust
impl PcdDeserialize for CustomPoint {
    fn read_spec() -> Vec<FieldDef> {
        vec![
            FieldDef::new("x", ValueKind::F32, 1),
            FieldDef::new("y", ValueKind::F32, 1),
            FieldDef::new("z", ValueKind::F32, 1),
            FieldDef::new("intensity", ValueKind::F32, 1),
        ]
    }
    
    fn read_fields(fields: Vec<Field>) -> Result<Self> {
        // Custom validation/transformation
        let x = fields[0].as_f32()?[0];
        let y = fields[1].as_f32()?[0];
        let z = fields[2].as_f32()?[0];
        let intensity = fields[3].as_f32()?[0];
        
        // Apply custom logic
        let normalized_intensity = intensity / 255.0;
        
        Ok(CustomPoint {
            x, y, z,
            intensity: normalized_intensity,
        })
    }
}
```

### Schema Validation

```rust
fn validate_schema(reader: &DynReader) -> Result<()> {
    let schema = &reader.meta().schema;
    
    // Check required fields exist
    let required = ["x", "y", "z"];
    for field_name in &required {
        if !schema.fields.iter().any(|f| f.name == *field_name) {
            return Err(Error::MissingRequiredField(field_name.to_string()));
        }
    }
    
    Ok(())
}
```

## Common Patterns

### Point Cloud Statistics

```rust
fn compute_bounds(reader: DynReader) -> Result<([f32; 3], [f32; 3])> {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    
    for point in reader {
        let (x, y, z) = point?.to_xyz()?;
        min[0] = min[0].min(x);
        min[1] = min[1].min(y);
        min[2] = min[2].min(z);
        max[0] = max[0].max(x);
        max[1] = max[1].max(y);
        max[2] = max[2].max(z);
    }
    
    Ok((min, max))
}
```

### Format Detection

```rust
fn detect_format(path: &Path) -> Result<DataKind> {
    let reader = DynReader::open(path)?;
    Ok(reader.meta().data_kind.clone())
}
```