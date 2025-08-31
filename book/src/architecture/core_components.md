# Core Components

This chapter details the core components that make up the pcd-rs library.

## Reader Components

### DynReader

The `DynReader` provides runtime-flexible PCD file reading:

```rust
pub struct DynReader<R> {
    meta: PcdMeta,
    reader: BufReader<R>,
    // Internal state...
}
```

Key responsibilities:
- Parse PCD headers dynamically
- Stream points as `DynRecord` instances
- Handle ASCII, binary, and compressed formats
- Validate data consistency

### Reader<T>

The generic `Reader<T>` provides type-safe reading:

```rust
pub struct Reader<T, R> 
where 
    T: PcdDeserialize,
    R: Read + Seek
{
    inner: DynReader<R>,
    _phantom: PhantomData<T>,
}
```

Features:
- Wraps `DynReader` with type information
- Compile-time schema validation
- Zero-cost abstraction over dynamic reader

## Writer Components

### DynWriter

Writes PCD files with runtime-defined schemas:

```rust
pub struct DynWriter<W> {
    writer: W,
    meta: PcdMeta,
    written_points: u64,
    // Buffer management...
}
```

Responsibilities:
- Generate PCD headers from schema
- Serialize points in chosen format
- Manage write buffers
- Validate point count

### Writer<T>

Type-safe writer for known schemas:

```rust
pub struct Writer<T, W>
where
    T: PcdSerialize,
    W: Write + Seek
{
    inner: DynWriter<W>,
    _phantom: PhantomData<T>,
}
```

### WriterInit

Builder pattern for writer configuration:

```rust
pub struct WriterInit {
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub data_kind: DataKind,
    pub schema: Option<Schema>,
}
```

## Data Structures

### DynRecord

Dynamic point representation:

```rust
pub struct DynRecord(pub Vec<Field>);
```

- Flexible field storage
- Runtime type checking
- Suitable for unknown schemas

### Field

Individual field data:

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

Each variant stores:
- Primitive type data
- Support for arrays (COUNT > 1)
- Efficient memory layout

## Metadata Components

### PcdMeta

Complete PCD file metadata:

```rust
pub struct PcdMeta {
    pub version: Version,
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub data_kind: DataKind,
    pub schema: Schema,
}
```

### Schema

Field definitions for the point cloud:

```rust
pub struct Schema {
    fields: Vec<FieldDef>,
}

pub struct FieldDef {
    pub name: String,
    pub value_kind: ValueKind,
    pub count: usize,
}
```

### ValueKind

PCD data types:

```rust
pub enum ValueKind {
    I8, I16, I32,
    U8, U16, U32,
    F32, F64,
}
```

Maps to PCD's TYPE field:
- `I`: Signed integers
- `U`: Unsigned integers
- `F`: Floating point

### DataKind

Storage format:

```rust
pub enum DataKind {
    Ascii,
    Binary,
    BinaryCompressed,
}
```

## Trait System

### PcdSerialize

Enables type serialization:

```rust
pub trait PcdSerialize {
    fn write_spec() -> Vec<FieldDef>;
    fn write_fields(&self) -> Vec<Field>;
}
```

### PcdDeserialize

Enables type deserialization:

```rust
pub trait PcdDeserialize: Sized {
    fn read_spec() -> Vec<FieldDef>;
    fn read_fields(fields: Vec<Field>) -> Result<Self>;
}
```

### Value Trait

Primitive type conversions:

```rust
pub trait Value: Sized {
    const KIND: ValueKind;
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}
```

Implemented for:
- All integer types (i8, i16, i32, u8, u16, u32)
- Floating point types (f32, f64)

## Error Handling

### Error Type

Comprehensive error enumeration:

```rust
pub enum Error {
    IoError(std::io::Error),
    ParseError(String),
    SchemaError(String),
    DataError(String),
    // More variants...
}
```

### Result Type Alias

Convenience type:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Internal Utilities

### Buffer Management

- Reusable buffers for reading/writing
- Endianness handling via `byteorder`
- Efficient string parsing with `regex`

### Validation

- Schema consistency checks
- Point count validation
- Type compatibility verification

## Component Interaction

```
User Code
    │
    ├─> Writer<T>/Reader<T> (Static API)
    │      │
    │      └─> DynWriter/DynReader (delegates)
    │              │
    │              ├─> Schema (validation)
    │              ├─> Field (data storage)
    │              └─> I/O (file operations)
    │
    └─> DynWriter/DynReader (Dynamic API)
           │
           └─> (same as above)
```

## Thread Safety

- **Readers**: `Send` but not `Sync` (iterator state)
- **Writers**: `Send` but not `Sync` (exclusive access)
- **Records**: `Send + Sync` when fields are immutable
- **Metadata**: `Send + Sync` (immutable after creation)