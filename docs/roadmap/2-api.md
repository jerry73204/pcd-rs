# Phase 2: API Improvements

Add missing convenience APIs and fill usability gaps identified from other
implementations.

## 2.1 Header-only read

PCL and pasture support reading only the metadata without loading point data.
Useful for file inspection, format detection, and pre-allocation.

- [ ] Add `PcdMeta::from_path(path)` that reads and parses only the header
- [ ] Add `PcdMeta::from_reader(reader)` for non-file sources
- [ ] For binary_compressed, do not decompress data
- [ ] Add example showing header inspection usage
- [ ] Add tests

### Acceptance criteria

- [ ] Metadata can be read without loading or decompressing point data
- [ ] Works for all data kinds and versions

## 2.2 `from_bytes` constructor

bye_pcd_rs and common usage patterns show demand for reading PCD from
in-memory byte slices without going through files.

- [ ] Add `Reader::from_bytes(&[u8])` convenience constructor
- [ ] Add `DynReader::from_bytes(&[u8])` convenience constructor
- [ ] Add tests for reading from byte slices

### Acceptance criteria

- [ ] PCD data in a `&[u8]` can be read without creating a temporary file

## 2.3 RGB helpers

PCL and pypcd both provide RGB pack/unpack utilities. Packed RGB-as-float32 is
a very common convention in PCD files.

- [ ] Add `rgb_to_float(r: u8, g: u8, b: u8) -> f32` utility
- [ ] Add `float_to_rgb(f: f32) -> (u8, u8, u8)` utility
- [ ] Add `rgba_to_float` and `float_to_rgba` variants
- [ ] Document the PCL RGB packing convention
- [ ] Add tests for round-trip and edge cases (NaN-producing bit patterns)

### Acceptance criteria

- [ ] RGB pack/unpack matches PCL behavior
- [ ] Documented in the public API

## 2.4 Padding field handling

PCL uses `_` fields to represent struct alignment padding in binary data.
pcd-rs renames them to `unknown_field_N` and treats them as real data fields,
which is incorrect for binary reads.

- [ ] Recognize `_` fields as padding in binary reads (skip SIZE bytes)
- [ ] Preserve `_` fields in metadata for header round-trip
- [ ] Skip padding fields when constructing `DynRecord` points
- [ ] Add test with PCL-style padding fields

### Acceptance criteria

- [ ] Binary PCD files with `_` padding fields from PCL are read correctly
- [ ] Padding bytes are skipped, not returned as field data
