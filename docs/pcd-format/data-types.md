# Data Types

This chapter details the data types supported in PCD files and their representation in pcd-rs.

## Type System Overview

PCD supports three categories of data types:
- **Signed Integers** (I): i8, i16, i32
- **Unsigned Integers** (U): u8, u16, u32  
- **Floating Point** (F): f32, f64

## Type Specifications

### Integer Types

#### Signed Integers (TYPE: I)

| SIZE | Rust Type | Range | Common Use |
|------|-----------|-------|------------|
| 1 | i8 | -128 to 127 | Labels, small offsets |
| 2 | i16 | -32,768 to 32,767 | Larger labels, indices |
| 4 | i32 | -2³¹ to 2³¹-1 | Timestamps, large indices |

```rust
// Example: Label field
#[derive(PcdSerialize, PcdDeserialize)]
struct LabeledPoint {
    x: f32,
    y: f32,
    z: f32,
    label: i32,  // TYPE: I, SIZE: 4
}
```

#### Unsigned Integers (TYPE: U)

| SIZE | Rust Type | Range | Common Use |
|------|-----------|-------|------------|
| 1 | u8 | 0 to 255 | RGB colors, intensity |
| 2 | u16 | 0 to 65,535 | Ring number, larger colors |
| 4 | u32 | 0 to 2³²-1 | Packed RGB, indices |

```rust
// Example: Color fields
#[derive(PcdSerialize, PcdDeserialize)]
struct ColorPoint {
    x: f32,
    y: f32, 
    z: f32,
    r: u8,   // TYPE: U, SIZE: 1
    g: u8,   // TYPE: U, SIZE: 1
    b: u8,   // TYPE: U, SIZE: 1
}
```

### Floating Point Types (TYPE: F)

| SIZE | Rust Type | Precision | Common Use |
|------|-----------|-----------|------------|
| 4 | f32 | ~7 digits | Coordinates, normals |
| 8 | f64 | ~15 digits | High-precision timestamps |

```rust
// Example: High-precision point
#[derive(PcdSerialize, PcdDeserialize)]
struct PrecisePoint {
    x: f64,  // TYPE: F, SIZE: 8
    y: f64,  // TYPE: F, SIZE: 8
    z: f64,  // TYPE: F, SIZE: 8
    timestamp: f64,  // Microsecond precision
}
```

## Special Data Representations

### RGB/RGBA Colors

RGB data can be stored in multiple ways:

#### Separate Channels
```
FIELDS r g b
SIZE 1 1 1
TYPE U U U
COUNT 1 1 1
```

```rust
struct RGBPoint {
    r: u8,
    g: u8,
    b: u8,
}
```

#### Packed Format
```
FIELDS rgb
SIZE 4
TYPE U
COUNT 1
```

```rust
struct PackedRGBPoint {
    rgb: u32,  // Packed as 0x00RRGGBB
}

// Unpacking
fn unpack_rgb(packed: u32) -> (u8, u8, u8) {
    let r = ((packed >> 16) & 0xFF) as u8;
    let g = ((packed >> 8) & 0xFF) as u8;
    let b = (packed & 0xFF) as u8;
    (r, g, b)
}
```

#### Float RGB (PCL Compatible)
```
FIELDS rgb
SIZE 4
TYPE F
COUNT 1
```

```rust
struct FloatRGBPoint {
    rgb: f32,  // Reinterpreted from u32
}

// Conversion
fn float_to_rgb(rgb_float: f32) -> (u8, u8, u8) {
    let rgb_int = rgb_float.to_bits();
    unpack_rgb(rgb_int)
}
```

### Array Fields

Fields with COUNT > 1 represent arrays:

```
FIELDS position normal descriptor
SIZE 4 4 4
TYPE F F F
COUNT 3 3 128
```

```rust
#[derive(PcdSerialize, PcdDeserialize)]
struct FeaturePoint {
    position: [f32; 3],     // COUNT: 3
    normal: [f32; 3],       // COUNT: 3
    descriptor: Vec<f32>,   // COUNT: 128
}
```

### Invalid Data (NaN)

For organized point clouds, invalid points use NaN:

```rust
use std::f32::NAN;

// Invalid point in organized cloud
let invalid_point = DynRecord(vec![
    Field::F32(vec![NAN]),  // x
    Field::F32(vec![NAN]),  // y
    Field::F32(vec![NAN]),  // z
]);
```

## Type Conversions in pcd-rs

### Field Enum

The `Field` enum represents dynamic field data:

```rust
pub enum Field {
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}
```

### Type Extraction

```rust
impl Field {
    pub fn as_f32(&self) -> Result<&Vec<f32>> {
        match self {
            Field::F32(v) => Ok(v),
            _ => Err(Error::TypeMismatch),
        }
    }
    
    pub fn to_value<T: Value>(&self) -> Result<Vec<T>> {
        // Convert to requested type
    }
}
```

### ValueKind Enum

Represents PCD type specifications:

```rust
pub enum ValueKind {
    I8, I16, I32,
    U8, U16, U32,
    F32, F64,
}

impl ValueKind {
    pub fn size(&self) -> usize {
        match self {
            ValueKind::I8 | ValueKind::U8 => 1,
            ValueKind::I16 | ValueKind::U16 => 2,
            ValueKind::I32 | ValueKind::U32 | ValueKind::F32 => 4,
            ValueKind::F64 => 8,
        }
    }
    
    pub fn type_char(&self) -> char {
        match self {
            ValueKind::I8 | ValueKind::I16 | ValueKind::I32 => 'I',
            ValueKind::U8 | ValueKind::U16 | ValueKind::U32 => 'U',
            ValueKind::F32 | ValueKind::F64 => 'F',
        }
    }
}
```

## Binary Representation

### Endianness

pcd-rs uses system endianness (typically little-endian):

```rust
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

// Reading
let value = reader.read_f32::<NativeEndian>()?;

// Writing  
writer.write_f32::<NativeEndian>(value)?;
```

### Memory Layout

Binary data is packed without padding:

```
Point with x:f32, y:f32, z:f32, label:i32
Memory: [x_bytes(4)][y_bytes(4)][z_bytes(4)][label_bytes(4)]
Total: 16 bytes per point
```

## Common Patterns

### Multi-channel Data

```rust
// LiDAR point with multiple returns
#[derive(PcdSerialize, PcdDeserialize)]
struct MultiReturnPoint {
    x: f32,
    y: f32,
    z: f32,
    intensity: [f32; 3],  // Multiple return intensities
    return_number: u8,
    number_of_returns: u8,
}
```

### Time-stamped Data

```rust
// Point with high-precision timestamp
#[derive(PcdSerialize, PcdDeserialize)]
struct TimedPoint {
    x: f32,
    y: f32,
    z: f32,
    timestamp: f64,  // Microsecond precision
    frame_id: u32,
}
```

### Semantic Data

```rust
// Point with semantic information
#[derive(PcdSerialize, PcdDeserialize)]
struct SemanticPoint {
    x: f32,
    y: f32,
    z: f32,
    class_id: u16,      // Object class
    instance_id: u32,   // Instance segmentation
    confidence: f32,    // Classification confidence
}
```

## Type Safety Best Practices

1. **Use appropriate types**: Don't use f64 when f32 suffices
2. **Pack related data**: Use arrays for vectors/colors
3. **Validate ranges**: Check integer overflow potential
4. **Handle NaN**: Especially in organized clouds
5. **Document units**: Specify units in field names or comments

## Performance Considerations

### Type Sizes and Performance

| Type | Size | Read Speed | Use When |
|------|------|------------|----------|
| u8/i8 | 1 byte | Fastest | Small ranges sufficient |
| u16/i16 | 2 bytes | Fast | Medium ranges needed |
| u32/i32/f32 | 4 bytes | Fast | Standard precision |
| f64 | 8 bytes | Slower | High precision required |

### Optimization Tips

1. **Minimize field count**: Fewer fields = faster I/O
2. **Use smallest sufficient type**: u8 instead of u32 for colors
3. **Align to cache lines**: Group related fields
4. **Avoid mixed types**: Uniform types optimize better