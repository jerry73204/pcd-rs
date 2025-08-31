# Project Status

## Current Version

**pcd-rs v0.12.0** - Released 2024

The library is actively maintained and production-ready for most use cases.

## Stability

| Component | Status | Notes |
|-----------|--------|-------|
| Core API | Stable | Breaking changes follow semver |
| Dynamic Reader/Writer | Stable | Well-tested with various PCD files |
| Static API (derive) | Stable | Mature implementation |
| Binary Format | Stable | Full support |
| ASCII Format | Stable | Full support |
| Compressed Format | Experimental | Basic LZF support |

## Platform Support

| Platform | Status | CI Coverage |
|----------|--------|-------------|
| Linux x86_64 | ✅ Full Support | Yes |
| macOS x86_64 | ✅ Full Support | Yes |
| macOS ARM64 | ✅ Full Support | Yes |
| Windows x86_64 | ✅ Full Support | Yes |
| WebAssembly | ⚠️ Untested | No |

## Feature Completeness

### Supported PCD Features

- ✅ Version 0.7 format
- ✅ ASCII data encoding
- ✅ Binary data encoding
- ✅ All primitive types (I8-I32, U8-U32, F32-F64)
- ✅ Field arrays (COUNT > 1)
- ✅ Organized point clouds
- ✅ Unorganized point clouds
- ✅ Custom field names
- ✅ Viewpoint metadata
- ✅ Unknown field handling ("_" fields)

### Partially Supported

- ⚠️ Binary compressed format (basic LZF only)
- ⚠️ Large file streaming (memory constraints)

### Not Yet Supported

- ❌ PCD versions before 0.7
- ❌ Custom compression algorithms
- ❌ Memory-mapped file access
- ❌ Parallel processing

## Performance Characteristics

| Operation | Performance | Notes |
|-----------|------------|-------|
| ASCII Read | ~50-100 MB/s | I/O bound |
| Binary Read | ~500-1000 MB/s | Near disk speed |
| ASCII Write | ~30-50 MB/s | Formatting overhead |
| Binary Write | ~300-500 MB/s | Direct memory copy |
| Derive Overhead | ~0-5% | Compile-time optimization |

## Known Limitations

1. **Memory Usage**: Entire point cloud loaded for writing
2. **Compression**: Only basic LZF algorithm supported
3. **Streaming**: Limited streaming for very large files
4. **Validation**: Schema validation could be stricter
5. **Error Messages**: Some error messages could be more descriptive

## Testing Coverage

- Unit tests for all public APIs
- Integration tests with real PCD files
- Property-based testing for serialization
- Example programs demonstrating usage
- CI/CD on multiple platforms

## Documentation Status

| Documentation | Status |
|---------------|--------|
| API Docs (rustdoc) | Complete |
| README | Complete |
| Examples | Comprehensive |
| Changelog | Maintained |
| Developer Guide | In Progress |

## Community

- **GitHub Stars**: Growing steadily
- **Downloads**: Increasing monthly
- **Issues**: Actively triaged
- **Pull Requests**: Welcome and reviewed
- **Response Time**: Usually within a week

## Dependencies

Minimal dependency footprint:
- `byteorder`: Endianness handling
- `thiserror`: Error derivation
- `regex`: Header parsing
- `num-traits`: Numeric abstractions
- `itertools`: Iterator utilities

Optional:
- `pcd-rs-derive`: Procedural macros (with "derive" feature)