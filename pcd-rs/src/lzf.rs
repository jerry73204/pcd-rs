//! LZF compression and decompression implementation for PCD binary_compressed format.
//!
//! Based on the LZF algorithm by Marc Alexander Lehmann.
//! This is a simple, fast compression algorithm suitable for real-time compression.

use crate::error::{Error, Result};

const HLOG: usize = 14;
const HSIZE: usize = 1 << HLOG;
const MAX_LIT: usize = 32;
const MAX_OFF: usize = 8192;
const MAX_REF: usize = 264; // 255 + 8 + 1

/// Decompress LZF compressed data.
///
/// # Arguments
/// * `input` - The compressed data
/// * `output_len` - Expected size of decompressed data
///
/// # Returns
/// The decompressed data as a Vec<u8>
pub fn decompress(input: &[u8], output_len: usize) -> Result<Vec<u8>> {
    let mut output = vec![0u8; output_len];
    let mut in_pos = 0;
    let mut out_pos = 0;

    while in_pos < input.len() {
        let ctrl = input[in_pos];
        in_pos += 1;

        if ctrl < 32 {
            // Literal run
            let len = ctrl as usize + 1;

            if in_pos + len > input.len() {
                return Err(Error::ParseError {
                    line: 0,
                    desc: "LZF decompression error: literal run exceeds input size".into(),
                });
            }

            if out_pos + len > output_len {
                return Err(Error::ParseError {
                    line: 0,
                    desc: "LZF decompression error: output buffer overflow".into(),
                });
            }

            output[out_pos..out_pos + len].copy_from_slice(&input[in_pos..in_pos + len]);
            in_pos += len;
            out_pos += len;
        } else {
            // Back reference
            let mut len = (ctrl >> 5) as usize;
            if len == 7 {
                // Long match
                if in_pos >= input.len() {
                    return Err(Error::ParseError {
                        line: 0,
                        desc: "LZF decompression error: long match length exceeds input".into(),
                    });
                }
                len += input[in_pos] as usize;
                in_pos += 1;
            }
            len += 2;

            if in_pos >= input.len() {
                return Err(Error::ParseError {
                    line: 0,
                    desc: "LZF decompression error: reference offset exceeds input size".into(),
                });
            }

            let high_offset = ((ctrl & 0x1f) as usize) << 8;
            let offset = high_offset + input[in_pos] as usize + 1;
            in_pos += 1;

            if offset > out_pos {
                return Err(Error::ParseError {
                    line: 0,
                    desc: format!(
                        "LZF decompression error: invalid back reference (offset {} > position {})",
                        offset, out_pos
                    ),
                });
            }

            if out_pos + len > output_len {
                return Err(Error::ParseError {
                    line: 0,
                    desc: "LZF decompression error: output buffer overflow".into(),
                });
            }

            // Copy from back reference (handle overlapping copies)
            let src_pos = out_pos - offset;
            if offset >= len {
                // Non-overlapping copy
                output.copy_within(src_pos..src_pos + len, out_pos);
            } else {
                // Overlapping copy - copy byte by byte
                for i in 0..len {
                    output[out_pos + i] = output[src_pos + i];
                }
            }
            out_pos += len;
        }
    }

    if out_pos != output_len {
        return Err(Error::ParseError {
            line: 0,
            desc: format!(
                "LZF decompression error: expected {} bytes, got {}",
                output_len, out_pos
            ),
        });
    }

    Ok(output)
}

/// Compress data using LZF algorithm.
///
/// # Arguments
/// * `input` - The data to compress
///
/// # Returns
/// The compressed data as a Vec<u8>
pub fn compress(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut output = Vec::with_capacity(input.len() + input.len() / 16 + 64);
    let mut htab = vec![0usize; HSIZE];

    let mut in_pos = 0;
    let mut lit = 0;
    let mut lit_pos = 0;

    while in_pos < input.len() {
        // Only try to find matches if we have enough lookahead
        if in_pos + 4 <= input.len() {
            let hval = hash(&input[in_pos..in_pos + 3]);
            let ref_pos = htab[hval];
            htab[hval] = in_pos;

            // Check if we have a match
            if ref_pos != 0
                && in_pos > ref_pos
                && in_pos - ref_pos <= MAX_OFF
                && input[ref_pos] == input[in_pos]
                && input[ref_pos + 1] == input[in_pos + 1]
                && input[ref_pos + 2] == input[in_pos + 2]
            {
                // Calculate match length
                let mut match_len = 3;
                let max_len = std::cmp::min(MAX_REF, input.len() - in_pos);

                while match_len < max_len
                    && ref_pos + match_len < input.len()
                    && input[ref_pos + match_len] == input[in_pos + match_len]
                {
                    match_len += 1;
                }

                // Output pending literals
                if lit > 0 {
                    output[lit_pos] = (lit - 1) as u8;
                    lit = 0;
                }

                // Output back reference
                let offset = in_pos - ref_pos - 1;
                let len = match_len - 2;

                if len < 7 {
                    output.push(((offset >> 8) as u8) | ((len as u8) << 5));
                } else {
                    output.push(((offset >> 8) as u8) | 0xe0);
                    output.push((len - 7) as u8);
                }
                output.push((offset & 0xff) as u8);

                // Update position and continue
                in_pos += match_len;
                continue;
            }
        }

        // No match found, add to literal run
        if lit == 0 {
            lit_pos = output.len();
            output.push(0); // Reserve space for literal count
        }

        output.push(input[in_pos]);
        lit += 1;
        in_pos += 1;

        if lit == MAX_LIT {
            output[lit_pos] = (MAX_LIT - 1) as u8;
            lit = 0;
        }
    }

    // Write final literal length
    if lit > 0 {
        output[lit_pos] = (lit - 1) as u8;
    }

    Ok(output)
}

/// Simple hash function for LZF
fn hash(data: &[u8]) -> usize {
    if data.len() < 3 {
        return 0;
    }
    let v = ((data[0] as usize) << 16) | ((data[1] as usize) << 8) | (data[2] as usize);
    ((v >> (24 - HLOG)) ^ v) & (HSIZE - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = b"Hello, world! This is a test of LZF compression. Hello, world!";

        let compressed = compress(original).unwrap();
        assert!(compressed.len() < original.len());

        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_empty_data() {
        let original = b"";
        let compressed = compress(original).unwrap();
        assert_eq!(compressed.len(), 0);
    }

    #[test]
    fn test_incompressible_data() {
        // Random-like data that won't compress well
        let original: Vec<u8> = (0..100).map(|i| (i * 7 + 13) as u8).collect();

        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_repetitive_data() {
        let original = vec![42u8; 1000];

        let compressed = compress(&original).unwrap();
        assert!(compressed.len() < original.len() / 10); // Should compress very well

        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_short_patterns() {
        let original = b"aaabbbcccaaabbbccc";

        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_single_byte() {
        let original = b"x";
        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_max_literal_run() {
        // Test maximum literal run (32 bytes)
        let mut original = Vec::new();
        for i in 0..32 {
            original.push((i * 7) as u8);
        }

        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_long_match() {
        // Create data that will trigger long matches (>= 7 byte matches)
        let mut original = Vec::new();
        let pattern = b"abcdefghijklmnop";
        original.extend_from_slice(pattern);
        original.extend_from_slice(b"xyz");
        original.extend_from_slice(pattern); // This should create a long match

        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_overlapping_copy() {
        // Test case that requires overlapping copy during decompression
        let original = b"abcabcabcabcabc";

        let compressed = compress(original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_binary_data() {
        // Test with actual binary float data (like PCD files)
        let original = vec![
            0x00, 0x00, 0x80, 0x3f, // 1.0f
            0x00, 0x00, 0x00, 0x40, // 2.0f
            0x00, 0x00, 0x40, 0x40, // 3.0f
            0x00, 0x00, 0x80, 0x3f, // 1.0f again
            0x00, 0x00, 0x00, 0x40, // 2.0f again
        ];

        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_invalid_literal_run() {
        // Invalid compressed data - literal run exceeds input
        let invalid = vec![31, 1, 2]; // Says 32 bytes but only has 2
        let result = decompress(&invalid, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_invalid_back_ref() {
        // Invalid compressed data - back reference beyond current position
        let invalid = vec![0x80, 0x10]; // Back reference with offset > position
        let result = decompress(&invalid, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_data() {
        // Test with larger data (1KB)
        let mut original = Vec::with_capacity(1024);
        for i in 0..256 {
            original.extend_from_slice(&[i as u8; 4]);
        }

        let compressed = compress(&original).unwrap();
        // Note: May not compress if data is too random

        let decompressed = decompress(&compressed, original.len()).unwrap();
        assert_eq!(decompressed, original);
    }
}
