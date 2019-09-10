//! The module defines [PCDRecordRead](crate::record::PCDRecordRead) and
//! [PCDRecordWrite](crate::record::PCDRecordWrite) traits. Both are analogous to
//! points in PCD data.
//!
//! Any object scanned by readers or written by writers must implement
//! [PCDRecordRead](crate::record::PCDRecordRead) or [PCDRecordWrite](crate::record::PCDRecordWrite)
//! respectively.
//!
//! These traits are not intended to implemented manually.
//! Please use derive macro instead. For example,
//!
//! ```rust
//! use pcd_rs::{PCDRecordRead, PCDRecordWrite};
//!
//! #[derive(PCDRecordRead, PCDRecordWrite)]
//! pub struct TimestampedPoint {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//!     timestamp: u32,
//! }
//! ```
//!
//! The derive macro accepts normal structs and tuple structs, but does not accept unit structs.
//!
//! [PCDRecordRead](crate::record::PCDRecordRead) allows fields with either primitive type,
//! array of primitive type or [Vec](<std::vec::Vec>) of primitive type.
//!
//! [PCDRecordWrite](crate::record::PCDRecordWrite) allows fields with either primitive type or
//! array of primitive type. The [Vec](<std::vec::Vec>) is ruled out since the length
//! is not determined in compile-time.
//!
//! Make sure struct field names match the `FIELDS` header in PCD data.
//! Otherwise it panics at runtime. You can specify the exact name or bypass name check
//! with attributes. For example,
//!
//! ```rust
//! use pcd_rs::{PCDRecordRead};
//!
//! #[derive(PCDRecordRead)]
//! pub struct TimestampedPoint {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//!     #[pcd_rename("true_name")]
//!     rust_name: u32,
//!     #[pcd_ignore_name]
//!     whatever_name: u32,
//! }
//! ```
//!
//! The name check are automatically ignored for tuple structs.

use crate::{FieldDef, ValueKind};
use byteorder::{LittleEndian, ReadBytesExt};
use failure::Fallible;
use std::io::prelude::*;

/// [PCDRecordRead](crate::record::PCDRecordRead) is analogous to a _point_ returned from a reader.
///
/// The trait is not intended to be implemented from scratch. You must
/// derive the implementation with `#[derive(PCDRecordRead)]`.
///
/// When the PCD data is in ASCII mode, the record is represented by a line of literals.
/// Otherwise if the data is in binary mode, the record is represented by a fixed size chunk.
pub trait PCDRecordRead: Sized {
    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)>;
    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
    fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
}

/// [PCDRecordWrite](crate::record::PCDRecordWrite) is analogous to a _point_ written by a writer.
///
/// The trait is not intended to be implemented from scratch. You must
/// derive the implementation with `#[derive(PCDRecordWrite)]`.
///
/// When the PCD data is in ASCII mode, the record is represented by a line of literals.
/// Otherwise if the data is in binary mode, the record is represented by a fixed size chunk.
pub trait PCDRecordWrite: Sized {
    fn write_spec() -> Vec<(String, ValueKind, usize)>;
    fn write_chunk<R: Write>(&self, writer: &mut R) -> Fallible<()>;
    fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()>;
}

impl PCDRecordRead for u8 {
    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)> {
        vec![(None, ValueKind::U8, Some(1))]
    }

    fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
        let value = reader.read_u8()?;
        Ok(value)
    }

    fn read_line<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line.parse()?)
    }
}

impl PCDRecordRead for i8 {
    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)> {
        vec![(None, ValueKind::I8, Some(1))]
    }

    fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
        let value = reader.read_i8()?;
        Ok(value)
    }

    fn read_line<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        Ok(line.parse()?)
    }
}

macro_rules! impl_primitive {
    ($ty:ty, $kind:ident, $read:ident) => {
        impl PCDRecordRead for $ty {
            fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)> {
                vec![(None, ValueKind::$kind, Some(1))]
            }

            fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
                let value = reader.$read::<LittleEndian>()?;
                Ok(value)
            }

            fn read_line<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
                let mut line = String::new();
                reader.read_line(&mut line)?;
                Ok(line.parse()?)
            }
        }
    };
}

impl_primitive!(u16, U16, read_u16);
impl_primitive!(u32, U32, read_u32);
impl_primitive!(i16, I16, read_i16);
impl_primitive!(i32, I32, read_i32);
impl_primitive!(f32, F32, read_f32);
impl_primitive!(f64, F64, read_f64);
