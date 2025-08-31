# API Reference

This chapter provides a quick reference to the main types and functions in pcd-rs.

## Core Types

### Readers

#### DynReader
```rust
pub struct DynReader<R: Read + Seek>

impl DynReader<R> {
    // Open from file path
    pub fn open(path: impl AsRef<Path>) -> Result<DynReader<File>>
    
    // From any reader
    pub fn from_reader(reader: R) -> Result<Self>
    
    // Get metadata
    pub fn meta(&self) -> &PcdMeta
}

impl Iterator for DynReader<R> {
    type Item = Result<DynRecord>;
}
```

#### Reader<T>
```rust
pub struct Reader<T: PcdDeserialize, R: Read + Seek>

impl Reader<T, R> {
    // Open typed reader
    pub fn open(path: impl AsRef<Path>) -> Result<Reader<T, File>>
    
    // From reader
    pub fn from_reader(reader: R) -> Result<Self>
}

impl Iterator for Reader<T, R> {
    type Item = Result<T>;
}
```

### Writers

#### DynWriter
```rust
pub struct DynWriter<W: Write + Seek>

impl DynWriter<W> {
    // Write a point
    pub fn push(&mut self, record: &DynRecord) -> Result<()>
    
    // Finalize the file
    pub fn finish(self) -> Result<()>
}
```

#### Writer<T>
```rust
pub struct Writer<T: PcdSerialize, W: Write + Seek>

impl Writer<T, W> {
    // Write typed point
    pub fn push(&mut self, value: &T) -> Result<()>
    
    // Finalize
    pub fn finish(self) -> Result<()>
}
```

#### WriterInit
```rust
pub struct WriterInit {
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub data_kind: DataKind,
    pub schema: Option<Schema>,
}

impl WriterInit {
    // Create dynamic writer
    pub fn create<W>(self, path: impl AsRef<Path>) -> Result<DynWriter<W>>
    
    // Create typed writer
    pub fn create_typed<T, W>(self, path: impl AsRef<Path>) -> Result<Writer<T, W>>
}
```

## Data Structures

### DynRecord
```rust
pub struct DynRecord(pub Vec<Field>);

impl DynRecord {
    // Extract XYZ coordinates
    pub fn to_xyz(&self) -> Result<(f32, f32, f32)>
}
```

### Field
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

impl Field {
    // Type-safe extraction
    pub fn as_f32(&self) -> Result<&Vec<f32>>
    pub fn as_u8(&self) -> Result<&Vec<u8>>
    // ... other types
    
    // Generic conversion
    pub fn to_value<T: Value>(&self) -> Result<Vec<T>>
}
```

## Metadata Types

### PcdMeta
```rust
pub struct PcdMeta {
    pub version: Version,
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub data_kind: DataKind,
    pub schema: Schema,
}

impl PcdMeta {
    // Total point count
    pub fn num_points(&self) -> u64
    
    // Check if organized
    pub fn is_organized(&self) -> bool
}
```

### Schema
```rust
pub struct Schema {
    pub fields: Vec<FieldDef>,
}

impl Schema {
    // Create from iterator
    pub fn from_iter<I>(iter: I) -> Self
    where I: IntoIterator<Item = (&str, ValueKind, usize)>
    
    // Check field existence
    pub fn has_field(&self, name: &str) -> bool
    
    // Get field index
    pub fn field_index(&self, name: &str) -> Option<usize>
}
```

### FieldDef
```rust
pub struct FieldDef {
    pub name: String,
    pub value_kind: ValueKind,
    pub count: usize,
}
```

### ValueKind
```rust
pub enum ValueKind {
    I8, I16, I32,
    U8, U16, U32,
    F32, F64,
}

impl ValueKind {
    // Size in bytes
    pub fn size(&self) -> usize
    
    // PCD type character
    pub fn type_char(&self) -> char
}
```

### DataKind
```rust
pub enum DataKind {
    Ascii,
    Binary,
    BinaryCompressed,
}
```

### ViewPoint
```rust
pub struct ViewPoint {
    pub tx: f64,  // Translation X
    pub ty: f64,  // Translation Y
    pub tz: f64,  // Translation Z
    pub qw: f64,  // Quaternion W
    pub qx: f64,  // Quaternion X
    pub qy: f64,  // Quaternion Y
    pub qz: f64,  // Quaternion Z
}

impl Default for ViewPoint {
    // Returns identity transform
    fn default() -> Self
}
```

## Traits

### PcdSerialize
```rust
pub trait PcdSerialize {
    // Get field specifications
    fn write_spec() -> Vec<FieldDef>;
    
    // Convert to fields
    fn write_fields(&self) -> Vec<Field>;
}

// Derive macro available
#[derive(PcdSerialize)]
struct MyPoint { /* ... */ }
```

### PcdDeserialize
```rust
pub trait PcdDeserialize: Sized {
    // Get expected schema
    fn read_spec() -> Vec<FieldDef>;
    
    // Create from fields
    fn read_fields(fields: Vec<Field>) -> Result<Self>;
}

// Derive macro available
#[derive(PcdDeserialize)]
struct MyPoint { /* ... */ }
```

### Value
```rust
pub trait Value: Sized {
    // Associated type kind
    const KIND: ValueKind;
    
    // Binary conversion
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}

// Implemented for: i8, i16, i32, u8, u16, u32, f32, f64
```

## Error Handling

### Error Type
```rust
pub enum Error {
    IoError(std::io::Error),
    InvalidHeader(String),
    SchemaError(String),
    DataError(String),
    TypeMismatch { expected: ValueKind, found: ValueKind },
    FieldCountMismatch,
    PointCountMismatch { expected: u64, actual: u64 },
    // ... more variants
}
```

### Result Type
```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Common Patterns

### Reading Any PCD File
```rust
use pcd_rs::{DynReader, Result};

fn read_pcd(path: &str) -> Result<Vec<DynRecord>> {
    DynReader::open(path)?
        .collect::<Result<Vec<_>>>()
}
```

### Writing Simple Points
```rust
use pcd_rs::{WriterInit, DynWriter, DynRecord, Field, DataKind, Schema, ValueKind};

fn write_points(points: Vec<(f32, f32, f32)>) -> Result<()> {
    let mut writer = WriterInit {
        width: points.len() as u64,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: Some(Schema::from_iter([
            ("x", ValueKind::F32, 1),
            ("y", ValueKind::F32, 1),
            ("z", ValueKind::F32, 1),
        ])),
    }
    .create("output.pcd")?;
    
    for (x, y, z) in points {
        let record = DynRecord(vec![
            Field::F32(vec![x]),
            Field::F32(vec![y]),
            Field::F32(vec![z]),
        ]);
        writer.push(&record)?;
    }
    
    writer.finish()
}
```

### Using Derive Macros
```rust
use pcd_rs::{PcdSerialize, PcdDeserialize, Reader, Writer};

#[derive(Debug, Clone, PcdSerialize, PcdDeserialize)]
struct Point {
    x: f32,
    y: f32,
    z: f32,
    #[pcd(rename = "intensity")]
    i: f32,
}

fn process_typed() -> Result<()> {
    // Read
    let reader: Reader<Point> = Reader::open("input.pcd")?;
    let points: Vec<Point> = reader.collect::<Result<_>>()?;
    
    // Write
    let mut writer: Writer<Point, _> = WriterInit {
        width: points.len() as u64,
        height: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Binary,
        schema: None,
    }
    .create("output.pcd")?;
    
    for point in points {
        writer.push(&point)?;
    }
    
    writer.finish()
}
```

## Feature Flags

```toml
[dependencies]
# Core functionality only
pcd-rs = "0.12"

# With derive macros
pcd-rs = { version = "0.12", features = ["derive"] }
```

## Platform Support

- **Tier 1**: Linux, macOS, Windows (x86_64)
- **Tier 2**: Linux, macOS (aarch64)
- **Tier 3**: WebAssembly (experimental)