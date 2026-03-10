# Comparison with Other PCD Implementations

Analysis of pcd-rs against other PCD implementations (March 2026).

## Implementations Studied

- **PCL** (C++) — the reference implementation, github.com/PointCloudLibrary/pcl
- **pypcd4** (Python) — modern Python PCD library, github.com/MapIV/pypcd4
- **pypcd** (Python) — original Python PCD library, github.com/dimatura/pypcd
- **pasture** (Rust) — point cloud library with SoA/AoS layouts, github.com/igd-geo/pasture
- **bye_pcd_rs** (Rust) — fork of pcd-rs, crates.io/crates/bye_pcd_rs

## Critical Issue: binary_compressed Layout

pcd-rs reads and writes binary_compressed data in **row-major (AoS)** order,
but the PCD specification (and every other implementation) uses **column-major
(SoA)** order.

- **Writer** buffers each point's fields contiguously (`XYZRGB XYZRGB ...`),
  then LZF-compresses. PCL expects per-field planes (`XXX...YYY...ZZZ...`).
- **Reader** decompresses into a flat buffer and reads fields sequentially as
  if row-major. PCL-generated compressed files produce garbage output.

pcd-rs can only round-trip with itself. Any binary_compressed file from PCL,
Open3D, pypcd, or any other tool will be misread.

## Feature Gaps

### High Priority

| Feature               | PCL        | pypcd/pypcd4 | pasture | pcd-rs                    |
|-----------------------|------------|--------------|---------|---------------------------|
| SoA compressed layout | Yes        | Yes          | N/A     | **No (broken)**           |
| I64/U64 types         | Yes        | Yes          | Yes     | **Missing**               |
| Flexible header order | Yes        | Yes          | N/A     | **Strict order required** |
| Padding `_` fields    | Skip bytes | Rename       | N/A     | Rename (wrong semantics)  |
| `COLUMNS` keyword     | Yes        | No           | N/A     | **Missing**               |
| Header-only read      | Yes        | No           | Yes     | **Missing**               |

### Medium Priority

| Feature                       | PCL | pypcd/pypcd4 | pasture         | pcd-rs             |
|-------------------------------|-----|--------------|-----------------|--------------------|
| mmap for binary I/O           | Yes | No           | Yes (zero-copy) | No                 |
| LZF fallback (incompressible) | N/A | Yes          | N/A             | No                 |
| RGB pack/unpack helpers       | Yes | Yes          | N/A             | No                 |
| `from_bytes(&[u8])` reader    | N/A | N/A          | N/A             | No                 |
| Batch/bulk read API           | Yes | Yes (numpy)  | Yes             | No                 |
| WIDTH*HEIGHT == POINTS check  | Yes | No           | N/A             | No                 |
| NaN output as `nan`           | Yes | Yes          | N/A             | No (outputs `NaN`) |
| Offset for embedded PCD       | Yes | No           | N/A             | No                 |

### Not Worth Implementing

- File locking / NFS msync — PCL-specific, overkill
- WASM support — bye_pcd_rs has it stubbed but unimplemented
- ROS PointCloud2 interop — application-level (pypcd4 has it)
- Subset writing by index — users can filter before writing

## Optimization Opportunities

### mmap for binary reads

PCL and pasture both memory-map binary PCD files. This avoids
kernel-to-userspace copies for large point clouds. Could use the `memmap2`
crate. Most impactful for large binary files (millions of points).

### Columnar buffer for compressed data

pasture's `HashMapBuffer` stores each attribute in its own `Vec<u8>`. This is
the natural internal representation for PCD binary_compressed, which stores
data per-field. Decompress directly into per-field column buffers instead of
the current scatter/gather approach.

### Batch read API

pasture's `read_into(buffer, count)` reads N points at once into a
pre-allocated buffer. This reduces per-point overhead from the iterator
pattern. Useful as a complement to the existing Iterator API.

### Pre-allocated compression buffer

PCL allocates `data_size * 1.5 + 8` bytes upfront for compression output.
pcd-rs uses a smaller estimate (`input.len() + input.len() / 16 + 64`) that
may trigger reallocation.

### Header-only read

PCL has a standalone `readHeader()` API that reads only metadata. Useful for
file inspection without loading data. Especially important for
binary_compressed, where the current reader decompresses everything upfront.

## Per-Implementation Notes

### PCL (reference)

The definitive implementation. Key behaviors to match:

- Column-major SoA layout for binary_compressed with per-field "planes"
- `_` fields represent struct alignment padding; SIZE bytes are skipped in binary reads
- `COLUMNS` accepted as alias for `FIELDS`
- Header entries accepted in any order
- `rgb` field gets special treatment: written as TYPE U in header, uint32 bit
  representation in ASCII mode (some RGB values map to NaN as float)
- LZF hash table uses HLOG=13 (8192 entries); pcd-rs uses HLOG=14 (16384)
- Uses `mmap` + `fallocate` for binary reads/writes
- Supports file offset parameter for reading PCD from TAR archives
- Scans for NaN/Inf to set `is_dense` flag after reading

### pypcd4

Modern Python implementation with good API design:

- Correct SoA handling for binary_compressed
- Falls back to uncompressed when LZF doesn't reduce size
- Multi-count fields (COUNT > 1) expanded into sub-fields with `__NNNN` suffixes
- Convenience constructors for common sensor schemas (Ouster, XYZIRT, etc.)
- Point cloud concatenation, slicing, boolean mask filtering
- Pydantic-based header validation
- RGB encode/decode helpers
- ROS PointCloud2 bidirectional conversion

### pypcd (original)

Similar to pypcd4 but older. Notable details:

- Documents that `_` padding fields break compressed point clouds in PCL;
  provides `rename_padding` option
- Uses `width` (not `points`) for column reconstruction in compressed format
- Compression fallback: writes uncompressed data with
  `compressed_size == uncompressed_size` when LZF doesn't help

### pasture

Not a PCD library (supports LAS/LAZ/3D Tiles), but its architecture is
instructive:

- Separates memory layout (interleaved vs columnar) from ownership
  (borrowed vs owned)
- `VectorBuffer` (AoS), `HashMapBuffer` (SoA), `ExternalMemoryBuffer`
  (zero-copy mmap)
- `BufferLayoutConverter` handles all four AoS/SoA conversion paths
- Batch reads via `read_into(buffer, count)`
- Runtime-typed buffers with opt-in static typing via derive macro

### bye_pcd_rs

Fork of pcd-rs. Despite claims in its README:

- **No mmap support** — the mmap text is copied from PCL docs, no code exists
- **No binary_compressed support** — the enum variant exists but reader/writer
  treat it identically to binary (no LZF, no SoA transposition)
- Adds `from_bytes(&[u8])` constructor and `to_xyz()` helper on DynRecord
- Not a competitive concern; pcd-rs is strictly more capable

## Priority Roadmap

1. **Fix binary_compressed SoA layout** — correctness bug, interop-breaking
2. **Add I64/U64 types** — easy, blocks reading some PCL files
3. **Flexible header parsing** — strict ordering rejects valid files
4. **Header-only read API** — quick win, useful for inspection
5. **mmap for binary** — performance for large files
6. **`from_bytes` constructor** — small convenience
7. **RGB helpers** — small convenience
