# Phase 3: Performance

Optimization opportunities identified from PCL and pasture implementations.

## 3.1 mmap for binary reads

PCL uses `mmap` for binary file I/O, avoiding kernel-to-userspace copies.
pasture supports zero-copy reads via `ExternalMemoryBuffer`. For large point
clouds (millions of points), this can significantly reduce read time.

- [ ] Add optional `memmap2` dependency (feature-gated)
- [ ] Implement mmap-based binary reader path
- [ ] Fall back to buffered I/O when mmap is unavailable or file is too small
- [ ] Benchmark mmap vs buffered I/O at various file sizes
- [ ] Add tests for mmap path
- [ ] Document the feature flag

### Acceptance criteria

- [ ] mmap reader is at least as fast as buffered I/O for files > 1MB
- [ ] Feature-gated so it does not add dependencies for users who don't need it
- [ ] Falls back gracefully on platforms without mmap

## 3.2 Batch read API

pasture's `read_into(buffer, count)` reads N points at once. This reduces
per-point iterator overhead for bulk processing.

- [ ] Add `Reader::read_batch(count) -> Result<Vec<Record>>` method
- [ ] Pre-allocate the output vector
- [ ] For binary data, read `count * point_size` bytes in one I/O call
- [ ] Benchmark batch vs iterator for large files
- [ ] Add tests

### Acceptance criteria

- [ ] Batch reads are measurably faster than iterator-based reads for large N
- [ ] Iterator API remains available and unchanged

## 3.3 Compression buffer tuning

PCL pre-allocates `data_size * 1.5 + 8` bytes for compression output.
pcd-rs uses a smaller estimate that may trigger reallocation.

- [ ] Increase initial compression buffer to `data_size * 3/2 + 8`
- [ ] Add overflow check for u32 size limits
- [ ] Handle incompressible data: if compressed >= uncompressed, write
      uncompressed with `compressed_size == uncompressed_size` (pypcd behavior)
- [ ] Add test for incompressible data (random bytes)

### Acceptance criteria

- [ ] No reallocation during compression for typical data
- [ ] Incompressible data does not produce larger output than input

## 3.4 NaN and ASCII output compatibility

PCL outputs `nan` (lowercase) for NaN float values and uses precision 8 for
ASCII output. pcd-rs outputs `NaN` (Rust default).

- [ ] Use lowercase `nan` in ASCII float output
- [ ] Match PCL's default precision (8 significant digits) for float output
- [ ] Add round-trip test for NaN values in ASCII mode

### Acceptance criteria

- [ ] ASCII output for NaN matches PCL format
- [ ] Float precision matches PCL defaults
