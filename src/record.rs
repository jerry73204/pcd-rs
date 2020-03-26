//! The module defines [PcdDeserialize](crate::record::PcdDeserialize) and
//! [PcdSerialize](crate::record::PcdSerialize) traits. Both are analogous to
//! points in PCD data.
//!
//! Any object scanned by readers or written by writers must implement
//! [PcdDeserialize](crate::record::PcdDeserialize) or [PcdSerialize](crate::record::PcdSerialize)
//! respectively.
//!
//! These traits are not intended to implemented manually.
//! Please use derive macro instead. For example,
//!
//! ```rust
//! use pcd_rs::{PcdDeserialize, PcdSerialize};
//!
//! #[derive(PcdDeserialize, PcdSerialize)]
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
//! [PcdDeserialize](crate::record::PcdDeserialize) allows fields with either primitive type,
//! array of primitive type or [Vec](<std::vec::Vec>) of primitive type.
//!
//! [PcdSerialize](crate::record::PcdSerialize) allows fields with either primitive type or
//! array of primitive type. The [Vec](<std::vec::Vec>) is ruled out since the length
//! is not determined in compile-time.
//!
//! Make sure struct field names match the `FIELDS` header in PCD data.
//! Otherwise it panics at runtime. You can specify the exact name in header or bypass name check
//! with attributes. The name check are automatically disabled for tuple structs.
//!
//! ```rust
//! use pcd_rs::PcdDeserialize;
//!
//! #[derive(PcdDeserialize)]
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

use crate::{
    error::PcdError,
    metas::{FieldDef, ValueKind},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use failure::Fallible;
use std::io::prelude::*;

/// [PcdDeserialize](crate::record::PcdDeserialize) is analogous to a _point_ returned from a reader.
///
/// The trait is not intended to be implemented from scratch. You must
/// derive the implementation with `#[derive(PcdDeserialize)]`.
///
/// When the PCD data is in ASCII mode, the record is represented by a line of literals.
/// Otherwise if the data is in binary mode, the record is represented by a fixed size chunk.
pub trait PcdDeserialize: Sized {
    fn is_dynamic() -> bool;
    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)>;
    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
    fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
}

/// [PcdSerialize](crate::record::PcdSerialize) is analogous to a _point_ written by a writer.
///
/// The trait is not intended to be implemented from scratch. You must
/// derive the implementation with `#[derive(PcdSerialize)]`.
///
/// When the PCD data is in ASCII mode, the record is represented by a line of literals.
/// Otherwise if the data is in binary mode, the record is represented by a fixed size chunk.
pub trait PcdSerialize: Sized {
    fn is_dynamic() -> bool;
    fn write_spec() -> Vec<(String, ValueKind, usize)>;
    fn write_chunk<R: Write + Seek>(
        &self,
        writer: &mut R,
        spec: &[(String, ValueKind, usize)],
    ) -> Fallible<()>;
    fn write_line<R: Write + Seek>(
        &self,
        writer: &mut R,
        spec: &[(String, ValueKind, usize)],
    ) -> Fallible<()>;
}

// Runtime record types

/// An enum representation of untyped data fields.
#[derive(Debug, Clone, PartialEq)]
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

impl Field {
    pub fn kind(&self) -> ValueKind {
        use Field as F;
        use ValueKind as K;

        match self {
            F::I8(_) => K::I8,
            F::I16(_) => K::I16,
            F::I32(_) => K::I32,
            F::U8(_) => K::U8,
            F::U16(_) => K::U16,
            F::U32(_) => K::U32,
            F::F32(_) => K::F32,
            F::F64(_) => K::F64,
        }
    }

    pub fn count(&self) -> usize {
        use Field as F;

        match self {
            F::I8(values) => values.len(),
            F::I16(values) => values.len(),
            F::I32(values) => values.len(),
            F::U8(values) => values.len(),
            F::U16(values) => values.len(),
            F::U32(values) => values.len(),
            F::F32(values) => values.len(),
            F::F64(values) => values.len(),
        }
    }
}

/// Represents an untyped _point_ in PCD data.
#[derive(Debug, Clone, PartialEq)]
pub struct DynRecord(pub Vec<Field>);

impl DynRecord {
    pub fn is_schema_consistent(&self, schema: &[(String, ValueKind, usize)]) -> bool {
        if self.0.len() != schema.len() {
            return false;
        }

        for (field, (_name, kind, count)) in self.0.iter().zip(schema.iter()) {
            use Field as F;
            use ValueKind as K;

            let matched = match field {
                F::I8(values) => values.len() == *count && *kind == K::I8,
                F::I16(values) => values.len() == *count && *kind == K::I16,
                F::I32(values) => values.len() == *count && *kind == K::I32,
                F::U8(values) => values.len() == *count && *kind == K::U8,
                F::U16(values) => values.len() == *count && *kind == K::U16,
                F::U32(values) => values.len() == *count && *kind == K::U32,
                F::F32(values) => values.len() == *count && *kind == K::F32,
                F::F64(values) => values.len() == *count && *kind == K::F64,
            };

            if !matched {
                return false;
            }
        }

        true
    }
}

impl PcdSerialize for DynRecord {
    fn is_dynamic() -> bool {
        true
    }

    fn write_spec() -> Vec<(String, ValueKind, usize)> {
        unreachable!();
    }

    fn write_chunk<Writer>(
        &self,
        writer: &mut Writer,
        spec: &[(String, ValueKind, usize)],
    ) -> Fallible<()>
    where
        Writer: Write + Seek,
    {
        if !self.is_schema_consistent(spec) {
            bail!("The content of record does not match the writer schema.");
        }

        for field in self.0.iter() {
            use Field as F;

            match field {
                F::I8(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_i8(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::I16(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_i16::<LittleEndian>(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::I32(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_i32::<LittleEndian>(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::U8(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_u8(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::U16(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_u16::<LittleEndian>(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::U32(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_u32::<LittleEndian>(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::F32(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_f32::<LittleEndian>(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
                F::F64(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_f64::<LittleEndian>(*val)?))
                        .collect::<Fallible<Vec<_>>>()?;
                }
            }
        }

        Ok(())
    }

    fn write_line<Writer>(
        &self,
        writer: &mut Writer,
        spec: &[(String, ValueKind, usize)],
    ) -> Fallible<()>
    where
        Writer: Write + Seek,
    {
        if !self.is_schema_consistent(spec) {
            bail!("The content of record does not match the writer schema.");
        }

        let mut tokens = vec![];

        for field in self.0.iter() {
            use Field as F;

            match field {
                F::I8(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::I16(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::I32(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::U8(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::U16(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::U32(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::F32(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
                F::F64(values) => {
                    let iter = values.iter().map(|val| val.to_string());
                    tokens.extend(iter);
                }
            }
        }

        writeln!(writer, "{}", tokens.join(" "))?;

        Ok(())
    }
}

impl PcdDeserialize for DynRecord {
    fn is_dynamic() -> bool {
        true
    }

    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)> {
        unreachable!();
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

        Ok(Self(fields))
    }

    fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

        {
            let expect = field_defs.iter().map(|def| def.count as usize).sum();
            let error = PcdError::new_text_token_mismatch_error(expect, tokens.len());
            if tokens.len() != expect {
                return Err(error.into());
            }
        }

        let mut tokens_iter = tokens.into_iter();
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
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I8(values)
                    }
                    ValueKind::I16 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I16(values)
                    }
                    ValueKind::I32 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::I32(values)
                    }
                    ValueKind::U8 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U8(values)
                    }
                    ValueKind::U16 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U16(values)
                    }
                    ValueKind::U32 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::U32(values)
                    }
                    ValueKind::F32 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::F32(values)
                    }
                    ValueKind::F64 => {
                        let values = counter
                            .map(|_| Ok(tokens_iter.next().unwrap().parse()?))
                            .collect::<Fallible<Vec<_>>>()?;
                        Field::F64(values)
                    }
                };

                Ok(field)
            })
            .collect::<Fallible<Vec<_>>>()?;

        Ok(Self(fields))
    }
}

// impl for primitive types

impl PcdDeserialize for u8 {
    fn is_dynamic() -> bool {
        false
    }

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

impl PcdDeserialize for i8 {
    fn is_dynamic() -> bool {
        false
    }

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
        impl PcdDeserialize for $ty {
            fn is_dynamic() -> bool {
                false
            }

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
