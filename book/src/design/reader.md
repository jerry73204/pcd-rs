# Reader Design

This chapter details the design and implementation of PCD file readers in pcd-rs.

## Design Goals

1. **Streaming**: Process large files without loading everything into memory
2. **Type Safety**: Compile-time validation when schema is known
3. **Flexibility**: Runtime schema discovery for unknown formats
4. **Performance**: Minimal overhead and allocations
5. **Error Recovery**: Graceful handling of malformed data

## Architecture

### Reader Hierarchy

```
Reader<T> (Static API)
    │
    └──> DynReader (Dynamic API)
            │
            ├──> HeaderParser
            ├──> DataDecoder
            └──> BufferManager
```

## Core Components

### DynReader Implementation

```rust
pub struct DynReader<R: Read + Seek> {
    // Metadata from header
    meta: PcdMeta,
    
    // Buffered reader for efficiency
    reader: BufReader<R>,
    
    // Current position in point sequence
    current_point: u64,
    
    // Reusable buffer for binary data
    buffer: Vec<u8>,
    
    // Precomputed offsets for binary format
    field_offsets: Vec<usize>,
}
```

### Header Parsing

The header parser follows a state machine approach:

```rust
enum HeaderState {
    ExpectingVersion,
    ExpectingFields,
    ExpectingSize,
    ExpectingType,
    ExpectingCount,
    ExpectingWidth,
    ExpectingHeight,
    ExpectingViewpoint,
    ExpectingPoints,
    ExpectingData,
    Complete,
}
```

Key features:
- Strict ordering validation
- Line-by-line parsing
- Early error detection
- Schema construction

### Data Decoding

Different strategies based on `DataKind`:

#### ASCII Decoding
```rust
fn decode_ascii_point(&mut self) -> Result<DynRecord> {
    let line = self.read_line()?;
    let values = line.split_whitespace();
    
    let mut fields = Vec::new();
    for (value, field_def) in values.zip(&self.meta.schema) {
        let field = parse_field(value, field_def)?;
        fields.push(field);
    }
    
    Ok(DynRecord(fields))
}
```

#### Binary Decoding
```rust
fn decode_binary_point(&mut self) -> Result<DynRecord> {
    // Read exact bytes for one point
    let point_size = self.meta.point_size();
    self.buffer.resize(point_size, 0);
    self.reader.read_exact(&mut self.buffer)?;
    
    // Extract fields using precomputed offsets
    let mut fields = Vec::new();
    for (field_def, &offset) in self.meta.schema.iter().zip(&self.field_offsets) {
        let field = extract_field(&self.buffer[offset..], field_def)?;
        fields.push(field);
    }
    
    Ok(DynRecord(fields))
}
```

## Iterator Implementation

The reader implements Rust's `Iterator` trait:

```rust
impl<R: Read + Seek> Iterator for DynReader<R> {
    type Item = Result<DynRecord>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_point >= self.meta.num_points() {
            return None;
        }
        
        let result = match self.meta.data_kind {
            DataKind::Ascii => self.decode_ascii_point(),
            DataKind::Binary => self.decode_binary_point(),
            DataKind::BinaryCompressed => self.decode_compressed_point(),
        };
        
        self.current_point += 1;
        Some(result)
    }
}
```

Benefits:
- Lazy evaluation
- Composability with iterator adaptors
- Memory efficiency
- Natural error propagation

## Type Conversion Layer

The static `Reader<T>` wraps `DynReader`:

```rust
impl<T: PcdDeserialize, R: Read + Seek> Iterator for Reader<T, R> {
    type Item = Result<T>;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| {
            result.and_then(|record| {
                // Validate schema compatibility
                self.validate_schema(&record)?;
                
                // Convert dynamic record to static type
                T::read_fields(record.0)
            })
        })
    }
}
```

## Performance Optimizations

### Buffer Management
- Reusable buffers to avoid allocations
- Sized exactly for binary point data
- Line buffer for ASCII parsing

### Precomputed Offsets
- Calculate field offsets once during initialization
- Direct indexing for binary data extraction
- No repeated size calculations

### Minimal Validation
- Schema validated once at open
- Type checking only when necessary
- Trust binary data layout

## Error Handling

### Error Types
```rust
pub enum ReaderError {
    // I/O errors
    Io(std::io::Error),
    
    // Header parsing errors
    InvalidHeader(String),
    MissingField(String),
    InvalidFieldOrder,
    
    // Data errors
    InvalidDataFormat,
    UnexpectedEof,
    FieldCountMismatch,
    
    // Type conversion errors
    TypeMismatch { expected: ValueKind, found: ValueKind },
    InvalidValue(String),
}
```

### Recovery Strategies
1. **Skip malformed points**: Continue iteration
2. **Partial reads**: Return successfully read points
3. **Schema mismatch**: Clear error messages
4. **I/O errors**: Propagate with context

## Advanced Features

### Seeking Support
```rust
impl<R: Read + Seek> DynReader<R> {
    pub fn seek_to_point(&mut self, index: u64) -> Result<()> {
        match self.meta.data_kind {
            DataKind::Binary => {
                let offset = self.data_offset + index * self.point_size;
                self.reader.seek(SeekFrom::Start(offset))?;
                self.current_point = index;
                Ok(())
            }
            _ => Err(Error::SeekNotSupported),
        }
    }
}
```

### Parallel Reading (Future)
```rust
pub struct ParallelReader<R> {
    readers: Vec<DynReader<R>>,
    chunk_size: usize,
}

impl<R: Read + Seek + Clone> ParallelReader<R> {
    pub fn read_parallel(&mut self) -> Vec<Result<DynRecord>> {
        self.readers
            .par_iter_mut()
            .flat_map(|reader| reader.by_ref().take(self.chunk_size))
            .collect()
    }
}
```

## Testing Strategy

### Unit Tests
- Header parsing edge cases
- Field type conversions
- Buffer management
- Iterator behavior

### Integration Tests
- Real PCD files
- Various schemas
- All data formats
- Large files

### Property Tests
- Round-trip read/write
- Schema compatibility
- Random data generation

## Future Improvements

1. **Async I/O**: Non-blocking reads
2. **Memory mapping**: Direct file access
3. **Compressed format**: Full LZF support
4. **Streaming**: Infinite point streams
5. **Validation**: Stricter schema checking