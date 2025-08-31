# Future Work

This page outlines planned features and improvements for pcd-rs, organized into development phases.

## Phase 1: Near-term (v0.13.0 - v0.14.0)

### Performance Optimizations
- [ ] **Memory-mapped file support** for large PCD files
- [ ] **Parallel processing** for multi-core systems
- [ ] **SIMD optimizations** for data conversion
- [ ] **Streaming writer** to avoid buffering entire point cloud

### Compression Support
- [ ] **Full LZF compression** implementation
- [ ] **Zlib compression** support
- [ ] **LZ4 compression** for better performance
- [ ] **Compression level** configuration

### API Improvements
- [ ] **Async I/O support** with tokio/async-std
- [ ] **Iterator adapters** for point transformation
- [ ] **Batch operations** for efficiency
- [ ] **Schema builder** with fluent API

## Phase 2: Mid-term (v0.15.0 - v0.16.0)

### Format Support
- [ ] **PCD version 0.6** compatibility
- [ ] **PCD version 0.5** compatibility
- [ ] **PLY format** reader/writer
- [ ] **LAS/LAZ format** integration

### Advanced Features
- [ ] **Spatial indexing** (KD-tree, Octree)
- [ ] **Point cloud operations** (filtering, sampling)
- [ ] **Coordinate transformations**
- [ ] **Normal estimation** utilities

### Developer Experience
- [ ] **Derive macro improvements** (better error messages)
- [ ] **Schema validation** at compile time
- [ ] **Format conversion** utilities
- [ ] **Debugging tools** for PCD files

## Phase 3: Long-term (v1.0.0)

### Core Stability
- [ ] **Stable API guarantee** (1.0 release)
- [ ] **Performance benchmarks** suite
- [ ] **Fuzzing** for robustness
- [ ] **Security audit** for file parsing

### Ecosystem Integration
- [ ] **PCL interop** layer
- [ ] **ROS integration** examples
- [ ] **Python bindings** via PyO3
- [ ] **C API** for FFI

### Advanced Processing
- [ ] **GPU acceleration** support
- [ ] **Distributed processing** for huge datasets
- [ ] **Incremental writing** for streaming applications
- [ ] **Custom metadata** extensions

## Experimental Ideas

### Research Topics
- [ ] **Machine learning** integration (point cloud networks)
- [ ] **WebAssembly** support for browser usage
- [ ] **Real-time streaming** protocols
- [ ] **Cloud storage** integration (S3, Azure, GCS)

### Community Features
- [ ] **Plugin system** for custom formats
- [ ] **Visualization** integration
- [ ] **Command-line tools** for PCD manipulation
- [ ] **GUI viewer** application

## Contributing Opportunities

### Good First Issues
- Documentation improvements
- Additional examples
- Test coverage expansion
- Error message enhancements

### Medium Complexity
- New compression algorithms
- Performance optimizations
- Format conversions
- API ergonomics

### Advanced Contributions
- Async I/O implementation
- Memory-mapped files
- SIMD optimizations
- GPU acceleration

## Breaking Changes Consideration

### Planned for v1.0.0
- Stabilize all public APIs
- Finalize trait definitions
- Standardize error types
- Lock core abstractions

### Migration Strategy
- Deprecation warnings in 0.x versions
- Migration guide documentation
- Automated migration tools
- Compatibility layer for transition

## Performance Goals

### Target Metrics
- ASCII read: >100 MB/s
- Binary read: >1 GB/s
- Memory usage: <2x file size
- Latency: <10ms for small files

### Optimization Areas
- Zero-allocation parsing
- Lock-free data structures
- Cache-friendly layouts
- Vectorized operations

## Community Roadmap

### Documentation
- [ ] Video tutorials
- [ ] Architecture deep-dives
- [ ] Performance guides
- [ ] Best practices

### Outreach
- [ ] Conference talks
- [ ] Blog posts
- [ ] Comparison guides
- [ ] Integration examples

## Compatibility Matrix

### Future Version Support
| Feature | v0.13 | v0.14 | v0.15 | v1.0 |
|---------|-------|-------|-------|------|
| Rust Version | 1.70+ | 1.70+ | 1.75+ | 1.75+ |
| Breaking Changes | Minor | Minor | Minor | None |
| API Stability | 90% | 95% | 98% | 100% |
| Performance Target | +20% | +40% | +60% | +100% |

## Feedback Wanted

We welcome community input on prioritization:
- Which features are most important?
- What use cases need better support?
- What performance targets matter most?
- What integrations would be valuable?

Please open an issue on [GitHub](https://github.com/jerry73204/pcd-rs/issues) to discuss!