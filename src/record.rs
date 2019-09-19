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
//! Otherwise it panics at runtime. You can specify the exact name in header or bypass name check
//! with attributes. The name check are automatically disabled for tuple structs.
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
//! The module provides [Field](crate::record::Field), an enum of data fields, and
//! [Record](crate::record::Record), an alias of `Vec<Field>` for untyped data loading.
//! [Record](crate::record::Record) already implements [PCDRecordRead](crate::record::PCDRecordRead),
//! and can be directly passed to reader.

use crate::{error::PCDError, FieldDef, ValueKind};
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
    fn read_spec() -> Option<Vec<(Option<String>, ValueKind, Option<usize>)>>;
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

// Runtime record types

/// An enum representation of untyped data fields.
#[derive(Debug, Clone)]
pub enum Field {
    I8(Vec<i8>),
    I16(Vec<i16>),
    I32(Vec<i32>),
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

/// The alias represents an untyped _point_ in PCD data.
pub type Record = Vec<Field>;

impl PCDRecordRead for Record {
    fn read_spec() -> Option<Vec<(Option<String>, ValueKind, Option<usize>)>> {
        None
    }

    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self> {
        let fields = field_defs
            .iter()
            .map(|def| {
                let FieldDef {
                    name: _,
                    kind,
                    count,
                } = def;

                let counter = (0..*count).into_iter();

                let field = match kind {
                    ValueKind::I8 => {
                        let values = counter
                            .map(|_| Ok(reader.read_i8()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I8(values)
                    }
                    ValueKind::I16 => {
                        let values = counter
                            .map(|_| Ok(reader.read_i16::<LittleEndian>()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I16(values)
                    }
                    ValueKind::I32 => {
                        let values = counter
                            .map(|_| Ok(reader.read_i32::<LittleEndian>()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I32(values)
                    }
                    ValueKind::U8 => {
                        let values = counter
                            .map(|_| Ok(reader.read_u8()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U8(values)
                    }
                    ValueKind::U16 => {
                        let values = counter
                            .map(|_| Ok(reader.read_u16::<LittleEndian>()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U16(values)
                    }
                    ValueKind::U32 => {
                        let values = counter
                            .map(|_| Ok(reader.read_u32::<LittleEndian>()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U32(values)
                    }
                    ValueKind::F32 => {
                        let values = counter
                            .map(|_| Ok(reader.read_f32::<LittleEndian>()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::F32(values)
                    }
                    ValueKind::F64 => {
                        let values = counter
                            .map(|_| Ok(reader.read_f64::<LittleEndian>()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::F64(values)
                    }
                };

                Ok(field)
            })
            .collect::<Fallible<Vec<_>>>()?;

        Ok(fields)
    }

    fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

        {
            let expect = field_defs.iter().map(|def| def.count as usize).sum();

            let error = PCDError::new_text_token_mismatch_error(expect, tokens.len());
            if tokens.len() != expect {
                return Err(error.into());
            }
        }

        let mut tokens_iter = tokens.into_iter();
        let fields = field_defs
            .iter()
            .map(|def| {
                let token = tokens_iter.next().unwrap();
                let FieldDef {
                    name: _,
                    kind,
                    count,
                } = def;

                let counter = (0..*count).into_iter();

                let field = match kind {
                    ValueKind::I8 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I8(values)
                    }
                    ValueKind::I16 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I16(values)
                    }
                    ValueKind::I32 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I32(values)
                    }
                    ValueKind::U8 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U8(values)
                    }
                    ValueKind::U16 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U16(values)
                    }
                    ValueKind::U32 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U32(values)
                    }
                    ValueKind::F32 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::F32(values)
                    }
                    ValueKind::F64 => {
                        let values = counter
                            .map(|_| Ok(token.parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::F64(values)
                    }
                };

                Ok(field)
            })
            .collect::<Fallible<Vec<_>>>()?;

        Ok(fields)
    }
}

// impl for primitive types

impl PCDRecordRead for u8 {
    fn read_spec() -> Option<Vec<(Option<String>, ValueKind, Option<usize>)>> {
        Some(vec![(None, ValueKind::U8, Some(1))])
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
    fn read_spec() -> Option<Vec<(Option<String>, ValueKind, Option<usize>)>> {
        Some(vec![(None, ValueKind::I8, Some(1))])
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
            fn read_spec() -> Option<Vec<(Option<String>, ValueKind, Option<usize>)>> {
                Some(vec![(None, ValueKind::$kind, Some(1))])
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
