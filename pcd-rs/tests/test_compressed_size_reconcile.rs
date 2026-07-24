//! Regression tests for binary_compressed uncompressed_size reconciliation.
//!
//! The `binary_compressed` data section carries two attacker-controlled `u32`
//! fields (`compressed_size`, `uncompressed_size`) read from the binary body.
//! The reader must treat `uncompressed_size` as a verifiable value, not a
//! primary allocation driver: the expected uncompressed length is fully
//! determined by the ASCII header (`sum(field.byte_size * field.count) *
//! num_points`). A divergence is corruption (or an attack) and must be rejected
//! *before* preallocating — otherwise an attacker can drive a multi-GiB
//! allocation from a handful of header bytes regardless of the declared point
//! count (untrusted-count preallocation DoS).
//!
//! These tests use small, modest `uncompressed_size` values that mismatch the
//! header-derived expected size (e.g. 1 KiB vs an expected 4 bytes), proving
//! the reconciliation fires on the Err path *before* any large allocation.
//! They do NOT allocate the declared size.

use pcd_rs::{DataKind, DynReader, DynRecord, DynWriter, Field, Schema, ValueKind, WriterInit};
use std::io::Cursor;

/// Minimal PCD v0.7 `DATA binary_compressed` header for a single-field
/// (x: f32), single-point cloud. The header-derived expected uncompressed size
/// is `record_size(4) * num_points(1) = 4 bytes`.
fn build_header() -> Vec<u8> {
    let h = b"VERSION 0.7\n\
FIELDS x\n\
SIZE 4\n\
TYPE F\n\
COUNT 1\n\
WIDTH 1\n\
HEIGHT 1\n\
POINTS 1\n\
DATA binary_compressed\n";
    h.to_vec()
}

/// Append the two body size u32s (little-endian) followed by `body` bytes.
fn build_blob(compressed_size: u32, uncompressed_size: u32, body: &[u8]) -> Vec<u8> {
    let mut buf = build_header();
    buf.extend_from_slice(&compressed_size.to_le_bytes());
    buf.extend_from_slice(&uncompressed_size.to_le_bytes());
    buf.extend_from_slice(body);
    buf
}

/// Build an LZF-compressed stream that decompresses to *exactly* `n` bytes,
/// every byte written (self-referencing back-references with offset=1 starting
/// from a single literal). Used to construct a body whose `uncompressed_size`
/// matches the actual decompressed length so that, on the unfixed reader, the
/// LZF decoder succeeds and `from_bytes` returns `Ok` (the RED: an attacker
/// `u32` is trusted as the allocation magnitude and accepted despite the header
/// saying the record is 4 bytes).
fn build_lzf_exact(n: usize) -> Vec<u8> {
    assert!(n >= 1, "build_lzf_exact expects n >= 1");
    let mut out: Vec<u8> = Vec::with_capacity(n / 64 + 16);
    // 1 literal byte seeds the back-reference source.
    out.push(0u8); // literal run of length 1
    out.push(b'A');
    let mut remaining = n - 1;
    // Full 264-byte back-references (long form: len = 7 + 255 + 2 = 264).
    while remaining > 264 {
        out.push(0xe0);
        out.push(255);
        out.push(0); // offset = 0 + 0 + 1 = 1 (self-ref)
        remaining -= 264;
    }
    // Final remainder in [0, 264].
    match remaining {
        0 => {}
        1 => {
            // A back-reference has minimum length 2; emit a 1-byte literal instead.
            out.push(0u8); // literal run of length 1
            out.push(b'A');
        }
        r if r >= 9 => {
            // long form: len = 7 + extra + 2 = 9 + extra
            out.push(0xe0);
            out.push((r - 9) as u8);
            out.push(0);
        }
        r => {
            // short form: len = (ctrl >> 5) + 2; ctrl = (r - 2) << 5
            out.push(((r - 2) << 5) as u8);
            out.push(0); // offset_low (offset = 0 + 0 + 1 = 1)
        }
    }
    out
}

#[test]
fn test_uncompressed_size_mismatch_small_rejected() {
    // Header says POINTS 1 / FIELDS x / SIZE 4 -> expected uncompressed = 4
    // bytes, but the body declares uncompressed_size = 1 KiB. This must Err
    // (not Ok, and crucially not allocate 1 KiB from the untrusted u32).
    let body = vec![0u8; 8];
    let blob = build_blob(body.len() as u32, 1024, &body);
    let res = DynReader::from_bytes(&blob);
    assert!(
        res.is_err(),
        "uncompressed_size mismatch (1024 != header-derived 4) must be rejected before prealloc"
    );
}

#[test]
fn test_uncompressed_size_huge_virtual_rejected() {
    // Probe A shape: compressed_size=0, uncompressed_size=256 MiB. Before the
    // fix this executed `vec![0u8; 256 MiB]` inside `lzf::decompress` and only
    // errored afterwards (the attacker u32 appeared verbatim in the error
    // string). After the fix the reconciliation rejects it before any
    // allocation. We use 256 MiB as the declared size but never materialize it.
    let blob = build_blob(0, 256 * 1024 * 1024, &[]);
    let res = DynReader::from_bytes(&blob);
    assert!(
        res.is_err(),
        "uncompressed_size=256 MiB on a 4-byte record must be rejected before prealloc"
    );
    // The error must be the reconciliation error, not the post-alloc LZF error
    // that embeds the attacker u32 verbatim.
    let err = match res {
        Ok(_) => panic!("expected Err, got Ok"),
        Err(e) => format!("{}", e),
    };
    assert!(
        !err.contains("268435456"),
        "must not reach lzf::decompress (which would embed the attacker u32); got: {err}"
    );
}

#[test]
fn test_uncompressed_size_physical_amplification_rejected() {
    // Probe B shape (scaled down to a modest 1 KiB so the test never makes a
    // large allocation): a tiny LZF stream of self-referencing back-references
    // that decompresses to *exactly* the declared `uncompressed_size` (1024).
    // On the unfixed reader the LZF decoder succeeds, `col_major`/`row_major`
    // are allocated from the attacker `u32`, and `from_bytes` returns **Ok**
    // despite the header saying the record is 4 bytes (the RED:
    // Ok-when-it-should-Err). After the fix the reconciliation rejects before
    // any allocation. (The scanner harness proves the same path at 64 MiB ->
    // ~68 MiB RSS -> Ok on pristine HEAD; this is the modest in-tree guard.)
    let declared = 1024usize;
    let lzf = build_lzf_exact(declared);
    let blob = build_blob(lzf.len() as u32, declared as u32, &lzf);
    let res = DynReader::from_bytes(&blob);
    assert!(
        res.is_err(),
        "uncompressed_size=1024 on a 4-byte record must be rejected before prealloc \
         (on unfixed HEAD this returned Ok)"
    );
}

#[test]
fn test_compressed_size_only_zero_with_nonzero_uncompressed_rejected() {
    // compressed_size=0 but uncompressed_size != expected. The empty shortcut
    // only triggers when *both* are zero; otherwise the reconciliation must
    // reject. (compressed_size=0 means read_exact reads nothing, but the
    // uncompressed_size check still fires first.)
    let blob = build_blob(0, 8, &[]);
    let res = DynReader::from_bytes(&blob);
    assert!(
        res.is_err(),
        "uncompressed_size=8 (!= expected 4) with compressed_size=0 must be rejected"
    );
}

#[test]
fn test_empty_compressed_cloud_still_loads() {
    // An empty compressed cloud (POINTS 0) is written with both size fields 0;
    // the empty shortcut must still return Ok with zero points.
    let h = b"VERSION 0.7\n\
FIELDS x\n\
SIZE 4\n\
TYPE F\n\
COUNT 1\n\
WIDTH 0\n\
HEIGHT 1\n\
POINTS 0\n\
DATA binary_compressed\n";
    let mut blob = h.to_vec();
    blob.extend_from_slice(&0u32.to_le_bytes()); // compressed_size
    blob.extend_from_slice(&0u32.to_le_bytes()); // uncompressed_size
    let reader = DynReader::from_bytes(&blob).expect("empty compressed cloud must load");
    let meta = reader.meta();
    assert_eq!(meta.num_points, 0);
}

#[test]
fn test_valid_small_compressed_cloud_still_loads() {
    // A well-formed binary_compressed cloud (header-derived expected == body
    // uncompressed_size) must still parse Ok and yield the correct point. Build
    // it with the public Writer API into an in-memory buffer, then read back.
    let schema = Schema::from_iter([("x", ValueKind::F32, 1)]);

    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);

    {
        let mut writer: DynWriter<_> = WriterInit {
            width: 1,
            height: 1,
            viewpoint: Default::default(),
            data_kind: DataKind::BinaryCompressed,
            schema: Some(schema),
            version: None,
        }
        .build_from_writer(cursor)
        .expect("build writer");

        writer
            .push(&DynRecord(vec![Field::F32(vec![1.0])]))
            .expect("push");
        writer.finish().expect("finish");
    }

    let reader = DynReader::from_bytes(&buf).expect("valid compressed cloud must load");
    let points: Vec<DynRecord> = reader.collect::<Result<_, _>>().expect("collect");
    assert_eq!(points.len(), 1);
    match &points[0].0[0] {
        Field::F32(v) => assert_eq!(v[0], 1.0),
        _ => panic!("Expected F32"),
    }
}
