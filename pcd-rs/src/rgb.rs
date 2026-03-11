//! Utilities for PCL-style packed RGB/RGBA values.
//!
//! PCL stores RGB color as a packed 32-bit integer reinterpreted as `f32`.
//! The layout is `0x00RRGGBB` for RGB and `0xAARRGGBB` for RGBA.
//!
//! The [`Rgb`] and [`Rgba`] wrapper types can be used as fields in structs
//! with `#[derive(PcdSerialize, PcdDeserialize)]`. They automatically
//! pack/unpack to the PCL float convention on the wire.

/// PCL-style RGB color, stored on the wire as a packed `f32`.
///
/// Use this as a field type in `#[derive(PcdSerialize, PcdDeserialize)]`
/// structs. The PCD field will be declared as `F32` with count 1,
/// and the packed float is automatically converted to/from RGB.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Pack into a PCL-style float.
    pub fn to_packed(self) -> f32 {
        rgb_to_float(self.r, self.g, self.b)
    }

    /// Unpack from a PCL-style float.
    pub fn from_packed(f: f32) -> Self {
        let (r, g, b) = float_to_rgb(f);
        Self { r, g, b }
    }
}

/// PCL-style RGBA color, stored on the wire as a packed `f32`.
///
/// Use this as a field type in `#[derive(PcdSerialize, PcdDeserialize)]`
/// structs. The PCD field will be declared as `F32` with count 1,
/// and the packed float is automatically converted to/from RGBA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Pack into a PCL-style float.
    pub fn to_packed(self) -> f32 {
        rgba_to_float(self.r, self.g, self.b, self.a)
    }

    /// Unpack from a PCL-style float.
    pub fn from_packed(f: f32) -> Self {
        let (r, g, b, a) = float_to_rgba(f);
        Self { r, g, b, a }
    }
}

/// Pack RGB values into a float using PCL's convention.
///
/// The RGB bytes are packed into a `u32` as `0x00RRGGBB`, then
/// reinterpreted (not converted) as `f32` via `f32::from_bits`.
pub fn rgb_to_float(r: u8, g: u8, b: u8) -> f32 {
    let packed: u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    f32::from_bits(packed)
}

/// Unpack a PCL-style packed RGB float into `(r, g, b)`.
///
/// The float is reinterpreted as `u32` via `f32::to_bits`, then
/// the RGB bytes are extracted from the `0x00RRGGBB` layout.
pub fn float_to_rgb(f: f32) -> (u8, u8, u8) {
    let packed = f.to_bits();
    let r = ((packed >> 16) & 0xFF) as u8;
    let g = ((packed >> 8) & 0xFF) as u8;
    let b = (packed & 0xFF) as u8;
    (r, g, b)
}

/// Pack RGBA values into a float using PCL's convention.
///
/// The RGBA bytes are packed into a `u32` as `0xAARRGGBB`, then
/// reinterpreted as `f32` via `f32::from_bits`.
pub fn rgba_to_float(r: u8, g: u8, b: u8, a: u8) -> f32 {
    let packed: u32 = (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    f32::from_bits(packed)
}

/// Unpack a PCL-style packed RGBA float into `(r, g, b, a)`.
///
/// The float is reinterpreted as `u32` via `f32::to_bits`, then
/// the RGBA bytes are extracted from the `0xAARRGGBB` layout.
pub fn float_to_rgba(f: f32) -> (u8, u8, u8, u8) {
    let packed = f.to_bits();
    let a = ((packed >> 24) & 0xFF) as u8;
    let r = ((packed >> 16) & 0xFF) as u8;
    let g = ((packed >> 8) & 0xFF) as u8;
    let b = (packed & 0xFF) as u8;
    (r, g, b, a)
}
