# Project Roadmap

## Current Version: 0.12.0

pcd-rs is actively maintained and production-ready for most use cases. This roadmap organizes development work into clear phases with concrete deliverables.

## Phase 1: Critical Format Support (Highest Priority)

Essential features for full PCD format compatibility.

### Binary Compressed Format
- [ ] Complete LZF decompression implementation
- [ ] LZF compression for writing compressed files
- [ ] Streaming decompression for large files
- [ ] Compression ratio configuration
- [ ] Fallback to uncompressed on compression failure
- [ ] Unit tests for various compressed data patterns
- [ ] Integration tests with real compressed PCD files
- [ ] Benchmarks comparing compressed vs uncompressed performance

### Legacy PCD Version Support
- [ ] PCD v0.6 reader implementation
- [ ] PCD v0.6 writer implementation
- [ ] PCD v0.5 reader implementation
- [ ] PCD v0.5 writer implementation
- [ ] Automatic version detection from header
- [ ] Version upgrade utility (v0.5/0.6 → v0.7)
- [ ] Compatibility tests with PCL-generated files
- [ ] Documentation of version differences

### Memory-Mapped File Access
- [ ] Design memory-mapped reader API
- [ ] Implement mmap for binary format reading
- [ ] Zero-copy point access interface
- [ ] Lazy loading of point data on access
- [ ] Random access to points by index
- [ ] Support for files larger than available RAM
- [ ] Platform-specific optimizations (Linux, Windows, macOS)
- [ ] Fallback to standard I/O when mmap unavailable

## Phase 2: Performance & Scalability

Optimizations for large-scale point cloud processing.

### Streaming Operations
- [ ] Streaming writer API design
- [ ] Write points without knowing total count upfront
- [ ] Chunked reading for memory-bounded processing
- [ ] Chunked writing with automatic header update
- [ ] Pipeline processing support with backpressure
- [ ] Streaming compression/decompression
- [ ] Progress callback for long operations
- [ ] Cancelable operations

### Parallel Processing
- [ ] Parallel ASCII parsing using rayon
- [ ] Multi-threaded binary data decoding
- [ ] Parallel compression with thread pool
- [ ] Concurrent point transformation pipelines
- [ ] Work-stealing for load balancing
- [ ] Configurable thread pool size
- [ ] Benchmarks showing parallel speedup
- [ ] Thread-safe reader/writer wrappers

### SIMD Optimizations
- [ ] SIMD type conversion routines
- [ ] Vectorized floating-point parsing
- [ ] Batch endianness conversion
- [ ] SIMD-accelerated compression
- [ ] Platform detection and fallback
- [ ] AVX2/AVX512 implementations where beneficial
- [ ] ARM NEON support
- [ ] Benchmark suite for SIMD performance

## Phase 3: Async & Modern I/O

Integration with async Rust ecosystem.

### Async I/O Support
- [ ] Async reader trait definition
- [ ] Async writer trait definition
- [ ] Tokio-based implementation
- [ ] Async-std-based implementation
- [ ] Async iterator for point streaming
- [ ] Non-blocking file operations
- [ ] Async compression/decompression
- [ ] Integration tests with async runtime
- [ ] Examples using async API

### Advanced I/O Features
- [ ] S3/cloud storage support
- [ ] HTTP/HTTPS streaming
- [ ] Pipe/socket support
- [ ] Memory buffer optimization
- [ ] I/O scheduling for multiple files
- [ ] Prefetching and caching strategies

## Phase 4: Format Interoperability

Support for additional point cloud formats.

### PLY Format
- [ ] PLY header parser
- [ ] PLY ASCII reader
- [ ] PLY binary reader
- [ ] PLY writer implementation
- [ ] Property list support
- [ ] Custom property handling
- [ ] PLY ↔ PCD conversion utility
- [ ] Format comparison benchmarks

### LAS/LAZ Format
- [ ] LAS header parsing
- [ ] LAS point data reading
- [ ] LAZ decompression support
- [ ] LAS writer implementation
- [ ] LAS ↔ PCD conversion
- [ ] Coordinate system handling
- [ ] Classification code mapping

### Other Formats
- [ ] XYZ ASCII format support
- [ ] OBJ point cloud subset
- [ ] CSV point cloud import/export
- [ ] Binary STL point extraction
- [ ] E57 format basics
- [ ] Custom format plugin system

## Phase 5: Developer Experience

Tools and improvements for library users.

### Command-Line Tools
- [ ] `pcd-info` - Display PCD file metadata
- [ ] `pcd-convert` - Format conversion utility
- [ ] `pcd-validate` - File validation tool
- [ ] `pcd-merge` - Combine multiple PCD files
- [ ] `pcd-split` - Split large PCD files
- [ ] `pcd-filter` - Point cloud filtering
- [ ] `pcd-sample` - Downsample point clouds
- [ ] `pcd-view` - Basic ASCII viewer

### Error Handling Improvements
- [ ] Detailed error messages with line/column info
- [ ] Schema mismatch diagnostics
- [ ] Suggested fixes for common errors
- [ ] Error recovery strategies
- [ ] Partial read/write support
- [ ] Validation mode with warnings
- [ ] Error serialization for debugging

### Documentation & Examples
- [ ] Complete API documentation
- [ ] Cookbook with common recipes
- [ ] Performance tuning guide
- [ ] Migration guide from PCL
- [ ] Video tutorials
- [ ] Interactive examples
- [ ] Benchmark results dashboard

## Phase 6: Ecosystem Integration

Integration with broader Rust and robotics ecosystem.

### ROS Integration
- [ ] ROS2 message conversion
- [ ] PointCloud2 compatibility
- [ ] ROS bag file support
- [ ] Real-time streaming from ROS topics
- [ ] Service definitions for PCD operations
- [ ] Launch file examples

### Visualization
- [ ] Basic point cloud viewer
- [ ] Integration with kiss3d
- [ ] Integration with bevy
- [ ] Web-based viewer with WASM
- [ ] Jupyter notebook support
- [ ] Color mapping utilities

### Machine Learning
- [ ] PyO3 Python bindings
- [ ] NumPy array conversion
- [ ] PyTorch tensor integration
- [ ] ONNX model support for processing
- [ ] Point cloud augmentation utilities

## Phase 7: 1.0 Stabilization

Final preparations for stable release.

### API Stabilization
- [ ] API review and cleanup
- [ ] Deprecate experimental features
- [ ] Finalize trait definitions
- [ ] Document all breaking changes
- [ ] Migration guide from 0.x
- [ ] API stability guarantees

### Quality Assurance
- [ ] 100% safe code in public API
- [ ] Comprehensive test coverage (>90%)
- [ ] Fuzzing for all parsers
- [ ] Security audit
- [ ] Performance regression tests
- [ ] Cross-platform CI/CD

### Performance Targets
- [ ] 1M points binary read < 500ms
- [ ] 1M points binary write < 300ms
- [ ] 10GB file support without OOM
- [ ] Compression ratio > 3:1
- [ ] Parallel speedup > 3x on 4 cores

## Completed Features ✅

### Core Functionality (v0.1.0 - v0.12.0)
- [x] PCD v0.7 format support
- [x] ASCII format reading/writing
- [x] Binary format reading/writing
- [x] Dynamic schema API (DynReader/DynWriter)
- [x] Static type API with generics
- [x] Derive macros (PcdSerialize/PcdDeserialize)
- [x] All primitive types (I8-I32, U8-U32, F32-F64)
- [x] Field arrays (COUNT > 1)
- [x] Organized point cloud support
- [x] Unorganized point cloud support
- [x] Unknown field handling ("_" fields)
- [x] Viewpoint metadata support
- [x] Iterator-based streaming
- [x] Builder pattern for writers
- [x] Comprehensive error types
- [x] Result-based error handling

### Recent Improvements
- [x] Removed anyhow dependency (v0.12.0)
- [x] Updated all dependencies (v0.12.0)
- [x] Workspace reorganization (v0.11.0)
- [x] Unknown field support (v0.11.0)
- [x] Non-owning to_xyz() method (v0.11.0)
- [x] Documentation on docs.rs (v0.10.0)
- [x] Value trait for type conversion (v0.9.x)
- [x] Field::to_value() method (v0.9.x)

### Infrastructure
- [x] CI/CD pipeline
- [x] Unit test suite
- [x] Integration test suite
- [x] Example programs
- [x] API documentation
- [x] README with examples
- [x] License (MIT)
- [x] Cargo workspace setup
- [x] Benchmarking framework

## Timeline Estimate

| Phase | Target | Version | Priority |
|-------|--------|---------|----------|
| Phase 1 | Q1 2024 | 0.13.0 | CRITICAL |
| Phase 2 | Q2 2024 | 0.14.0 | HIGH |
| Phase 3 | Q3 2024 | 0.15.0 | MEDIUM |
| Phase 4 | Q4 2024 | 0.16.0 | MEDIUM |
| Phase 5 | Q1 2025 | 0.17.0 | LOW |
| Phase 6 | Q2 2025 | 0.18.0 | LOW |
| Phase 7 | Q3 2025 | 1.0.0 | MEDIUM |

## Contributing

### How to Help

**High Priority Contributions:**
1. Implement LZF compression (Phase 1)
2. Add PCD v0.6 support (Phase 1)
3. Create memory-mapped reader (Phase 1)
4. Add parallel processing (Phase 2)
5. Write command-line tools (Phase 5)

**Good First Issues:**
- Add more examples
- Improve error messages
- Extend test coverage
- Fix documentation typos
- Add benchmarks

**Testing Needed:**
- Large file handling (>1GB)
- Various compression ratios
- Cross-platform compatibility
- Performance on different CPUs
- Real-world PCD files

### Development Process

1. Check the roadmap phase you want to contribute to
2. Open an issue to discuss your approach
3. Fork and create a feature branch
4. Implement with tests and documentation
5. Submit a pull request referencing the roadmap item

## Success Metrics

**Key Performance Indicators:**
- Full PCD format compatibility (all versions)
- Support for 10GB+ files
- Sub-second processing for 1M points
- Zero unsafe code in public API
- 90%+ test coverage
- Active community contributions

**User Satisfaction Goals:**
- Easy migration from PCL
- Intuitive API design
- Comprehensive documentation
- Fast response to issues
- Regular release cycle

## Feedback

We need your input! Please let us know:
- Which phases are most important to you?
- What file sizes do you work with?
- Which formats do you need?
- What performance targets matter?
- How can we improve the API?

Open an issue on [GitHub](https://github.com/jerry73204/pcd-rs/issues) or start a discussion!