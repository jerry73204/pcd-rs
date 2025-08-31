# Data Flow

This chapter explains how data flows through the pcd-rs library during reading and writing operations.

## Reading Data Flow

### Overview

```
PCD File → Header Parser → Schema Builder → Data Iterator → Type Converter → User Code
```

### Detailed Flow

1. **File Opening**
   ```
   File Path/Handle
        │
        ▼
   BufReader (buffered I/O)
        │
        ▼
   Header Parser
   ```

2. **Header Parsing**
   ```
   Raw Header Lines
        │
        ▼
   Field Extraction (VERSION, FIELDS, SIZE, etc.)
        │
        ▼
   Validation & Ordering Check
        │
        ▼
   PcdMeta Construction
   ```

3. **Schema Construction**
   ```
   FIELDS + SIZE + TYPE + COUNT
        │
        ▼
   FieldDef Creation
        │
        ▼
   Schema Assembly
        │
        ▼
   Offset Calculation (for binary)
   ```

4. **Data Reading**
   ```
   Data Section
        │
        ├─── ASCII ──→ Line Parser ──→ Token Split ──→ Value Parse
        │
        ├─── Binary ──→ Buffer Read ──→ Byte Extraction ──→ Type Cast
        │
        └─── Compressed ──→ Decompress ──→ Binary Flow
   ```

5. **Type Conversion**
   ```
   DynRecord (dynamic)
        │
        ▼
   PcdDeserialize::read_fields (if static)
        │
        ▼
   User Type T
   ```

## Writing Data Flow

### Overview

```
User Data → Type Converter → Schema Validator → Header Writer → Data Encoder → PCD File
```

### Detailed Flow

1. **Initialization**
   ```
   WriterInit Configuration
        │
        ├──→ Schema (provided or inferred)
        ├──→ Dimensions (width × height)
        ├──→ Data Format (ASCII/Binary)
        └──→ Viewpoint
              │
              ▼
        Writer Creation
   ```

2. **Header Generation**
   ```
   PcdMeta
        │
        ▼
   Header Field Formatting
        │
        ├──→ "VERSION 0.7"
        ├──→ "FIELDS x y z ..."
        ├──→ "SIZE 4 4 4 ..."
        ├──→ "TYPE F F F ..."
        └──→ etc.
              │
              ▼
        Write to File
   ```

3. **Point Processing**
   ```
   User Type T (if static)
        │
        ▼
   PcdSerialize::write_fields
        │
        ▼
   DynRecord
        │
        ▼
   Field Validation
        │
        ▼
   Data Encoding
   ```

4. **Data Encoding**
   ```
   DynRecord
        │
        ├─── ASCII ──→ Format Values ──→ Write Line
        │
        ├─── Binary ──→ Serialize Bytes ──→ Write Buffer
        │
        └─── Compressed ──→ Accumulate ──→ Compress ──→ Write
   ```

5. **Finalization**
   ```
   Check Point Count
        │
        ▼
   Flush Buffers
        │
        ▼
   Close File
   ```

## Memory Flow

### Reading Memory Pattern

```
Disk ──→ OS Buffer ──→ BufReader ──→ Parse Buffer ──→ DynRecord ──→ User Type
      8KB chunks    8KB buffer    Line/Point     Fields        Optional
```

Memory characteristics:
- Streaming: One point in memory at a time
- Buffered I/O: 8KB default buffer
- Reusable buffers for binary reading

### Writing Memory Pattern

```
User Type ──→ DynRecord ──→ Format Buffer ──→ BufWriter ──→ OS Buffer ──→ Disk
Optional      Fields        Point data       8KB buffer    System       Flush
```

Memory characteristics:
- Accumulation: All points before writing
- Buffered output: Reduces system calls
- Format-specific buffers

## Type System Flow

### Static Type Flow

```rust
// Reading
PCD File ──→ DynReader ──→ DynRecord ──→ T::read_fields() ──→ User Type T

// Writing
User Type T ──→ T::write_fields() ──→ DynRecord ──→ DynWriter ──→ PCD File
```

### Dynamic Type Flow

```rust
// Reading
PCD File ──→ DynReader ──→ DynRecord ──→ User Processing

// Writing
User Creation ──→ DynRecord ──→ DynWriter ──→ PCD File
```

## Error Propagation

### Reading Errors

```
File I/O Error ──┐
                 │
Header Error ────┼──→ Result<T, Error> ──→ User Handling
                 │
Data Error ──────┘
```

Error points:
1. File not found/permissions
2. Invalid header format
3. Schema inconsistencies
4. Data parsing failures
5. Type conversion errors

### Writing Errors

```
Config Error ────┐
                 │
Schema Error ────┼──→ Result<(), Error> ──→ User Handling
                 │
Write Error ─────┘
```

Error points:
1. Invalid configuration
2. Schema validation failure
3. I/O write errors
4. Point count mismatch
5. Finalization errors

## Optimization Paths

### Fast Path (Binary)

```
Binary Data ──→ Direct Memory Map ──→ Type Cast ──→ User Type
              (future)              Zero-copy
```

Optimizations:
- No parsing overhead
- Direct memory access
- Platform-native endianness

### Slow Path (ASCII)

```
Text Data ──→ Line Parse ──→ Token Split ──→ Parse Numbers ──→ User Type
            String alloc    Iterator       Parse overhead
```

Bottlenecks:
- String allocation
- Number parsing
- Format flexibility cost

## Parallel Data Flow (Future)

### Parallel Reading

```
PCD File
    │
    ├──→ Thread 1: Read chunk 1 ──→ Parse ──→ Queue ──┐
    │                                                   │
    ├──→ Thread 2: Read chunk 2 ──→ Parse ──→ Queue ──┼──→ Merge ──→ User
    │                                                   │
    └──→ Thread 3: Read chunk 3 ──→ Parse ──→ Queue ──┘
```

### Parallel Writing

```
Points Collection
    │
    ├──→ Thread 1: Encode batch 1 ──→ Buffer ──┐
    │                                           │
    ├──→ Thread 2: Encode batch 2 ──→ Buffer ──┼──→ Sequential Write
    │                                           │
    └──→ Thread 3: Encode batch 3 ──→ Buffer ──┘
```

## Stream Processing

### Continuous Reading

```rust
// Infinite stream processing
loop {
    let reader = DynReader::open_stream(stream)?;
    for point in reader {
        process_point(point?);
        if should_stop() { break; }
    }
}
```

### Incremental Writing

```rust
// Write as data arrives
let mut writer = create_streaming_writer()?;
for point in point_stream {
    writer.push(&point)?;
    if writer.buffer_full() {
        writer.flush()?;
    }
}
```

## Data Validation Flow

### Schema Validation

```
User Schema ──→ Consistency Check ──→ Type Validation ──→ Size Validation
                Field count          I/U/F types        1/2/4/8 bytes
```

### Runtime Validation

```
Point Data ──→ Field Count Check ──→ Type Check ──→ Range Check ──→ Accept/Reject
              Match schema          Match types    NaN/Inf handling
```