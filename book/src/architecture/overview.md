# Architecture Overview

The pcd-rs library is designed with modularity, type safety, and performance in mind. This chapter provides a high-level overview of the library's architecture.

## Design Principles

### 1. Dual API Philosophy

pcd-rs provides two complementary APIs:

- **Dynamic API**: Runtime flexibility for unknown schemas
- **Static API**: Compile-time safety and zero-cost abstractions

This dual approach allows users to choose the right tool for their use case.

### 2. Zero-Copy Where Possible

The library minimizes allocations and copies:

- Binary data is read directly into typed structures
- Iterator-based streaming avoids loading entire files into memory
- Efficient buffer management for writing

### 3. Type Safety

Leveraging Rust's type system:

- Strong typing for PCD data types (F32, U8, I32, etc.)
- Compile-time validation with derive macros
- Result-based error handling throughout

## High-Level Architecture

```
┌─────────────────────────────────────────────┐
│                User Application              │
└─────────────┬───────────────┬────────────────┘
              │               │
       Static API      Dynamic API
              │               │
┌─────────────▼───────────────▼────────────────┐
│            Core Library (pcd-rs)             │
├───────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌──────────┐    │
│  │ Reader  │  │ Writer  │  │  Traits  │    │
│  └─────────┘  └─────────┘  └──────────┘    │
│  ┌─────────┐  ┌─────────┐  ┌──────────┐    │
│  │ Record  │  │  Metas  │  │  Error   │    │
│  └─────────┘  └─────────┘  └──────────┘    │
└───────────────────────────────────────────────┘
              │
┌─────────────▼─────────────────────────────────┐
│        Derive Macros (pcd-rs-derive)         │
│  ┌──────────────┐  ┌───────────────┐        │
│  │ PcdSerialize │  │ PcdDeserialize│        │
│  └──────────────┘  └───────────────┘        │
└───────────────────────────────────────────────┘
```

## Crate Structure

The project consists of two crates:

### pcd-rs (Main Library)

The core library containing:

- **reader.rs**: `DynReader` and `Reader<T>` implementations
- **writer.rs**: `DynWriter` and `Writer<T>` implementations
- **record.rs**: `DynRecord` and `Field` types for dynamic data
- **metas.rs**: PCD metadata structures (`Schema`, `FieldDef`, etc.)
- **traits.rs**: Core traits (`PcdSerialize`, `PcdDeserialize`, `Value`)
- **error.rs**: Error types and Result aliases
- **utils.rs**: Internal utilities

### pcd-rs-derive (Procedural Macros)

Provides derive macros:

- **PcdSerialize**: Generates serialization code for structs
- **PcdDeserialize**: Generates deserialization code for structs

## Data Flow

### Reading Flow

1. **File Opening**: Reader opens file and parses header
2. **Schema Discovery**: Extracts field definitions from header
3. **Data Iteration**: Lazily reads points as iterator
4. **Type Conversion**: Converts to either `DynRecord` or user type

### Writing Flow

1. **Writer Initialization**: Configure with `WriterInit`
2. **Header Generation**: Build header from schema
3. **Point Serialization**: Convert points to PCD format
4. **File Finalization**: Write data and close file

## Memory Model

### Dynamic API Memory Layout

```
DynRecord
├── Field[0]: F32(Vec<f32>)
├── Field[1]: U8(Vec<u8>)
└── Field[2]: I32(Vec<i32>)
```

### Static API Memory Layout

```
PointXYZ (struct)
├── x: f32  (4 bytes)
├── y: f32  (4 bytes)
└── z: f32  (4 bytes)
Total: 12 bytes (packed)
```

## Concurrency Model

pcd-rs is designed with the following concurrency properties:

- **Readers**: Can be safely sent between threads (`Send`)
- **Writers**: Exclusive access required (not `Sync`)
- **Records**: Immutable once created, safe to share

## Extension Points

The library provides several extension points:

1. **Custom Types**: Implement `PcdSerialize`/`PcdDeserialize`
2. **Field Transformations**: Process fields during read/write
3. **Schema Validation**: Custom validation logic
4. **Error Handling**: Wrap errors with application context

## Performance Considerations

Key performance optimizations:

- **Lazy Evaluation**: Iterator-based API avoids loading entire files
- **Buffer Reuse**: Internal buffers are reused across iterations
- **Direct Memory Mapping**: Binary format uses direct memory access
- **Compile-Time Optimization**: Static API enables inlining and dead code elimination

## Next Steps

- Explore [Core Components](./core_components.md) for detailed component descriptions
- Learn about [Data Flow](./data_flow.md) for processing pipelines