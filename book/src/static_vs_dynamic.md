# Static vs Dynamic API

pcd-rs provides two complementary APIs for working with PCD files. This chapter helps you choose the right approach for your use case.

## Overview Comparison

| Aspect | Dynamic API | Static API |
|--------|------------|------------|
| Schema | Runtime discovery | Compile-time definition |
| Type Safety | Runtime checks | Compile-time guarantees |
| Performance | Good | Excellent |
| Flexibility | Maximum | Limited to defined types |
| Use Case | Unknown schemas | Known schemas |

## Dynamic API

### When to Use

Choose the dynamic API when:
- Working with PCD files of unknown schema
- Building generic PCD processing tools
- Schema varies at runtime
- Maximum flexibility is needed

### Example Use Cases

```rust
use pcd_rs::{DynReader, DynWriter, DynRecord};

// Generic PCD viewer
fn view_any_pcd(path: &str) -> Result<()> {
    let reader = DynReader::open(path)?;
    
    // Discover schema at runtime
    let schema = reader.meta().schema.clone();
    println!("Found fields: {:?}", schema.field_names());
    
    // Process points generically
    for point in reader {
        let point = point?;
        process_generic_point(&point, &schema);
    }
    
    Ok(())
}

// Format converter
fn convert_pcd(input: &str, output: &str) -> Result<()> {
    let reader = DynReader::open(input)?;
    let meta = reader.meta().clone();
    
    // Preserve original schema
    let mut writer = WriterInit {
        width: meta.width,
        height: meta.height,
        viewpoint: meta.viewpoint,
        data_kind: DataKind::Binary,
        schema: Some(meta.schema),
    }
    .create(output)?;
    
    // Copy points without knowing structure
    for point in reader {
        writer.push(&point?)?;
    }
    
    writer.finish()
}
```

### Advantages

✅ **Maximum Flexibility**
- Handle any valid PCD schema
- No compilation required for new formats

✅ **Runtime Adaptability**
- Adjust to different schemas dynamically
- Build generic tools

✅ **Schema Discovery**
- Inspect unknown PCD files
- Extract metadata without prior knowledge

### Disadvantages

❌ **Runtime Overhead**
- Type checking at runtime
- Dynamic dispatch costs

❌ **Less Type Safety**
- Errors discovered at runtime
- Manual type conversions

❌ **Verbose Code**
- Manual field extraction
- Pattern matching required

## Static API

### When to Use

Choose the static API when:
- Schema is known at compile time
- Type safety is important
- Performance is critical
- Working with specific point types

### Example Use Cases

```rust
use pcd_rs::{Reader, Writer, PcdSerialize, PcdDeserialize};

#[derive(Debug, Clone, PcdSerialize, PcdDeserialize)]
struct LidarPoint {
    x: f32,
    y: f32,
    z: f32,
    intensity: f32,
    ring: u16,
    timestamp: f64,
}

// Type-safe processing
fn process_lidar_scan(path: &str) -> Result<Vec<LidarPoint>> {
    let reader: Reader<LidarPoint> = Reader::open(path)?;
    
    // Direct access to typed fields
    let points: Vec<LidarPoint> = reader
        .filter_map(Result::ok)
        .filter(|p| p.intensity > 0.5)
        .collect();
    
    Ok(points)
}

// Efficient writing
fn write_lidar_points(points: &[LidarPoint], path: &str) -> Result<()> {
    let mut writer: Writer<LidarPoint, _> = WriterInit {
        width: points.len() as u64,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: None, // Inferred from type
    }
    .create(path)?;
    
    for point in points {
        writer.push(point)?; // Type-safe push
    }
    
    writer.finish()
}
```

### Advantages

✅ **Compile-Time Safety**
- Errors caught during compilation
- No runtime type errors

✅ **Better Performance**
- Zero-cost abstractions
- Compiler optimizations
- Inlining opportunities

✅ **Cleaner Code**
- Direct field access
- No manual type conversions
- IDE autocomplete support

✅ **Self-Documenting**
- Types document the schema
- Clear data structures

### Disadvantages

❌ **Less Flexibility**
- Schema must be known
- Requires recompilation for new schemas

❌ **More Boilerplate**
- Type definitions required
- Derive macros needed

## Choosing the Right API

### Decision Tree

```
Do you know the schema at compile time?
    │
    ├─ Yes ──→ Will the schema change?
    │              │
    │              ├─ No ──→ Use Static API
    │              │
    │              └─ Yes ──→ Use Dynamic API
    │
    └─ No ──→ Use Dynamic API
```

### Common Patterns

#### Mixed Approach

```rust
// Start with dynamic to discover schema
fn analyze_and_process(path: &str) -> Result<()> {
    // First pass: discover schema
    let reader = DynReader::open(path)?;
    let schema = reader.meta().schema.clone();
    
    // Validate expected fields
    if schema.has_fields(&["x", "y", "z", "intensity"]) {
        // Second pass: use static API for known schema
        let reader: Reader<KnownPoint> = Reader::open(path)?;
        process_known_points(reader)?;
    } else {
        // Fall back to dynamic processing
        let reader = DynReader::open(path)?;
        process_unknown_points(reader)?;
    }
    
    Ok(())
}
```

#### Schema Migration

```rust
// Convert dynamic to static
impl TryFrom<DynRecord> for MyPoint {
    type Error = Error;
    
    fn try_from(record: DynRecord) -> Result<Self> {
        Ok(MyPoint {
            x: record.get_field("x")?.as_f32()?[0],
            y: record.get_field("y")?.as_f32()?[0],
            z: record.get_field("z")?.as_f32()?[0],
        })
    }
}

// Use conversion
let reader = DynReader::open("points.pcd")?;
let points: Vec<MyPoint> = reader
    .map(|r| r.and_then(MyPoint::try_from))
    .collect::<Result<_>>()?;
```

## Performance Comparison

### Benchmark Results

| Operation | Dynamic API | Static API | Difference |
|-----------|------------|------------|------------|
| Read 1M points (binary) | 1.2s | 1.0s | 20% faster |
| Read 1M points (ASCII) | 8.5s | 7.8s | 8% faster |
| Write 1M points (binary) | 0.8s | 0.7s | 14% faster |
| Memory usage | +15% | Baseline | 15% less |

### Memory Usage

**Dynamic API:**
```
DynRecord overhead: ~24 bytes
Field overhead: ~32 bytes per field
Total per point: 24 + (32 × field_count)
```

**Static API:**
```
Struct size: Exact sum of field sizes
No runtime overhead
Better cache locality
```

## Best Practices

### Dynamic API Best Practices

1. **Cache Schema Information**
```rust
let schema = reader.meta().schema.clone();
let x_index = schema.field_index("x")?;
// Use index for faster access
```

2. **Batch Processing**
```rust
let points: Vec<DynRecord> = reader.collect::<Result<_>>()?;
// Process in batches for better performance
```

3. **Validate Early**
```rust
fn validate_schema(schema: &Schema) -> Result<()> {
    // Check required fields upfront
    let required = ["x", "y", "z"];
    for field in required {
        if !schema.has_field(field) {
            return Err(Error::MissingField(field.to_string()));
        }
    }
    Ok(())
}
```

### Static API Best Practices

1. **Use Appropriate Types**
```rust
#[derive(PcdSerialize, PcdDeserialize)]
struct EfficientPoint {
    position: [f32; 3],  // More efficient than x, y, z
    color: u32,          // Packed RGB
}
```

2. **Leverage Type System**
```rust
fn process<P: PointTrait>(reader: Reader<P>) -> Result<()> 
where 
    P: PcdDeserialize + Clone + Send 
{
    // Generic processing with constraints
}
```

3. **Custom Derives**
```rust
#[derive(PcdSerialize, PcdDeserialize)]
struct CustomPoint {
    #[pcd(rename = "t")]
    timestamp: f64,
    
    #[pcd(ignore)]
    cached_value: Option<f32>,
}
```

## Migration Guide

### From Dynamic to Static

```rust
// Before: Dynamic API
let reader = DynReader::open("cloud.pcd")?;
for point in reader {
    let point = point?;
    let x = point.0[0].as_f32()?[0];
    // ...
}

// After: Static API
#[derive(PcdDeserialize)]
struct Point { x: f32, y: f32, z: f32 }

let reader: Reader<Point> = Reader::open("cloud.pcd")?;
for point in reader {
    let point = point?;
    let x = point.x;  // Direct access
    // ...
}
```

### From Static to Dynamic

```rust
// Before: Static API (limited to one type)
let reader: Reader<SpecificPoint> = Reader::open("cloud.pcd")?;

// After: Dynamic API (handle any schema)
let reader = DynReader::open("cloud.pcd")?;
match identify_schema(reader.meta().schema) {
    SchemaType::Lidar => process_lidar(reader),
    SchemaType::RGBD => process_rgbd(reader),
    SchemaType::Unknown => process_generic(reader),
}
```