//! Tests for PCL-style RGB/RGBA pack/unpack utilities.

use pcd_rs::{float_to_rgb, float_to_rgba, rgb_to_float, rgba_to_float};

#[test]
fn test_rgb_round_trip() {
    let cases = [
        (0, 0, 0),
        (255, 255, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (128, 64, 32),
        (1, 2, 3),
    ];

    for (r, g, b) in cases {
        let f = rgb_to_float(r, g, b);
        let (r2, g2, b2) = float_to_rgb(f);
        assert_eq!(
            (r, g, b),
            (r2, g2, b2),
            "RGB round-trip failed for ({r}, {g}, {b})"
        );
    }
}

#[test]
fn test_rgba_round_trip() {
    let cases = [
        (0, 0, 0, 0),
        (255, 255, 255, 255),
        (255, 0, 0, 128),
        (0, 255, 0, 64),
        (0, 0, 255, 1),
        (128, 64, 32, 200),
    ];

    for (r, g, b, a) in cases {
        let f = rgba_to_float(r, g, b, a);
        let (r2, g2, b2, a2) = float_to_rgba(f);
        assert_eq!(
            (r, g, b, a),
            (r2, g2, b2, a2),
            "RGBA round-trip failed for ({r}, {g}, {b}, {a})"
        );
    }
}

#[test]
fn test_rgb_known_values() {
    // Pure red: 0x00FF0000
    let f = rgb_to_float(255, 0, 0);
    assert_eq!(f.to_bits(), 0x00FF0000);

    // Pure green: 0x0000FF00
    let f = rgb_to_float(0, 255, 0);
    assert_eq!(f.to_bits(), 0x0000FF00);

    // Pure blue: 0x000000FF
    let f = rgb_to_float(0, 0, 255);
    assert_eq!(f.to_bits(), 0x000000FF);
}

#[test]
fn test_rgba_known_values() {
    // Opaque red: 0xFFFF0000
    let f = rgba_to_float(255, 0, 0, 255);
    assert_eq!(f.to_bits(), 0xFFFF0000);

    // Half-transparent green: 0x8000FF00
    let f = rgba_to_float(0, 255, 0, 128);
    assert_eq!(f.to_bits(), 0x8000FF00);
}

#[test]
fn test_rgb_extracts_from_rgba() {
    // float_to_rgb should ignore the alpha channel
    let f = rgba_to_float(100, 150, 200, 255);
    let (r, g, b) = float_to_rgb(f);
    assert_eq!((r, g, b), (100, 150, 200));
}
