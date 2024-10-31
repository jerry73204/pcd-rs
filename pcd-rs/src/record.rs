#![doc = r##"
Defines serializing and deserializing traits and common record types.

Any object scanned by readers or written by writers must implement
[PcdDeserialize](crate::record::PcdDeserialize) or [PcdSerialize](crate::record::PcdSerialize)
respectively.

These traits are not intended to implemented manually.
Please use derive macro instead. For example,

"##]
#![cfg_attr(
    feature = "derive",
    doc = r##"
```rust
use pcd_rs::{PcdDeserialize, PcdSerialize};

#[derive(PcdDeserialize, PcdSerialize)]
pub struct TimestampedPoint {
    x: f32,
    y: f32,
    z: f32,
    timestamp: u32,
}
```
"##
)]
#![doc = r##"
The derive macro accepts normal structs and tuple structs, but does not accept unit structs.

[PcdDeserialize](crate::record::PcdDeserialize) allows fields with either primitive type,
array of primitive type or [Vec](<std::vec::Vec>) of primitive type.

[PcdSerialize](crate::record::PcdSerialize) allows fields with either primitive type or
array of primitive type. The [Vec](<std::vec::Vec>) is ruled out since the length
is not determined in compile-time.

Make sure struct field names match the `FIELDS` header in PCD data.
Otherwise it panics at runtime. You can specify the exact name in header or bypass name check
with attributes. The name check are automatically disabled for tuple structs.
"##]
#![cfg_attr(
    feature = "derive",
    doc = r##"
```rust
use pcd_rs::PcdDeserialize;

#[derive(PcdDeserialize)]
pub struct TimestampedPoint {
    x: f32,
    y: f32,
    z: f32,
    #[pcd(rename = "true_name")]
    rust_name: u32,
    #[pcd(ignore)]
    whatever_name: u32,
}
```
"##
)]
use crate::{
    error::Error,
    metas::{FieldDef, Schema, ValueKind},
    traits::Value,
    Result,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;
use num_traits::NumCast;
use std::io::prelude::*;

/// [PcdDeserialize](crate::record::PcdDeserialize) is analogous to a _point_ returned from a reader.
///
/// The trait is not intended to be implemented from scratch. You must
/// derive the implementation with `#[derive(PcdDeserialize)]`.
///
/// When the PCD data is in Ascii mode, the record is represented by a line of literals.
/// Otherwise if the data is in binary mode, the record is represented by a fixed size chunk.
pub trait PcdDeserialize: Sized {
    fn is_dynamic() -> bool;
    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)>;
    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &Schema) -> Result<Self>;
    fn read_line<R: BufRead>(reader: &mut R, field_defs: &Schema) -> Result<Self>;
}

/// [PcdSerialize](crate::record::PcdSerialize) is analogous to a _point_ written by a writer.
///
/// The trait is not intended to be implemented from scratch. You must
/// derive the implementation with `#[derive(PcdSerialize)]`.
///
/// When the PCD data is in Ascii mode, the record is represented by a line of literals.
/// Otherwise if the data is in binary mode, the record is represented by a fixed size chunk.
pub trait PcdSerialize: Sized {
    fn is_dynamic() -> bool;
    fn write_spec() -> Schema;
    fn write_chunk<R: Write + Seek>(&self, writer: &mut R, spec: &Schema) -> Result<()>;
    fn write_line<R: Write + Seek>(&self, writer: &mut R, spec: &Schema) -> Result<()>;
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

    pub fn to_value<T>(&self) -> Option<T>
    where
        T: Value + NumCast,
    {
        use Field as F;

        if T::KIND != self.kind() {
            return None;
        }

        Some(match self {
            F::I8(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::I16(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::I32(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::U8(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::U16(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::U32(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::F32(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
            F::F64(v) => match &**v {
                &[t] => T::from(t)?,
                _ => return None,
            },
        })
    }
}

/// Represents an untyped _point_ in PCD data.
#[derive(Debug, Clone, PartialEq)]
pub struct DynRecord(pub Vec<Field>);

impl DynRecord {
    pub fn is_schema_consistent(&self, schema: &Schema) -> bool {
        if self.0.len() != schema.len() {
            return false;
        }

        self.0
            .iter()
            .zip(schema.iter())
            .all(|(field, schema_field)| {
                use Field as F;
                use ValueKind as K;

                if field.count() != schema_field.count as usize {
                    return false;
                }

                matches!(
                    (field, schema_field.kind),
                    (F::I8(_), K::I8)
                        | (F::I16(_), K::I16)
                        | (F::I32(_), K::I32)
                        | (F::U8(_), K::U8)
                        | (F::U16(_), K::U16)
                        | (F::U32(_), K::U32)
                        | (F::F32(_), K::F32)
                        | (F::F64(_), K::F64)
                )
            })
    }

    pub fn to_xyz<T>(&self) -> Option<[T; 3]>
    where
        T: Value + NumCast,
    {
        use Field as F;

        if self.0.first()?.kind() != T::KIND {
            return None;
        }

        Some(match &*self.0 {
            [F::I8(xv), F::I8(yv), F::I8(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::I16(xv), F::I16(yv), F::I16(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::I32(xv), F::I32(yv), F::I32(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::U8(xv), F::U8(yv), F::U8(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::U16(xv), F::U16(yv), F::U16(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::U32(xv), F::U32(yv), F::U32(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::F32(xv), F::F32(yv), F::F32(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            [F::F64(xv), F::F64(yv), F::F64(zv), ..] => match (&**xv, &**yv, &**zv) {
                (&[x], &[y], &[z]) => [T::from(x)?, T::from(y)?, T::from(z)?],
                _ => return None,
            },
            _ => return None,
        })
    }
}

impl PcdSerialize for DynRecord {
    fn is_dynamic() -> bool {
        true
    }

    fn write_spec() -> Schema {
        unreachable!();
    }

    fn write_chunk<Writer>(&self, writer: &mut Writer, spec: &Schema) -> Result<()>
    where
        Writer: Write + Seek,
    {
        if !self.is_schema_consistent(spec) {
            return Err(Error::new_writer_schema_mismatch_error(
                Self::write_spec().fields,
                spec.fields.to_vec(),
            ));
        }

        for field in self.0.iter() {
            use Field as F;

            match field {
                F::I8(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_i8(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::I16(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_i16::<LittleEndian>(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::I32(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_i32::<LittleEndian>(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::U8(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_u8(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::U16(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_u16::<LittleEndian>(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::U32(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_u32::<LittleEndian>(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::F32(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_f32::<LittleEndian>(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
                F::F64(values) => {
                    values
                        .iter()
                        .map(|val| Ok(writer.write_f64::<LittleEndian>(*val)?))
                        .collect::<Result<Vec<_>>>()?;
                }
            }
        }

        Ok(())
    }

    fn write_line<Writer>(&self, writer: &mut Writer, spec: &Schema) -> Result<()>
    where
        Writer: Write + Seek,
    {
        if !self.is_schema_consistent(spec) {
            return Err(Error::new_writer_schema_mismatch_error(
                Self::write_spec().fields,
                spec.fields.to_vec(),
            ));
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

    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &Schema) -> Result<Self> {
        use Field as F;
        use ValueKind as K;

        let fields = field_defs
            .iter()
            .map(|def| {
                let FieldDef { kind, count, .. } = *def;

                let counter = 0..count;

                let field = match kind {
                    K::I8 => {
                        let values = counter
                            .map(|_| Ok(reader.read_i8()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::I8(values)
                    }
                    K::I16 => {
                        let values = counter
                            .map(|_| Ok(reader.read_i16::<LittleEndian>()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::I16(values)
                    }
                    K::I32 => {
                        let values = counter
                            .map(|_| Ok(reader.read_i32::<LittleEndian>()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::I32(values)
                    }
                    K::U8 => {
                        let values = counter
                            .map(|_| Ok(reader.read_u8()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::U8(values)
                    }
                    K::U16 => {
                        let values = counter
                            .map(|_| Ok(reader.read_u16::<LittleEndian>()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::U16(values)
                    }
                    K::U32 => {
                        let values = counter
                            .map(|_| Ok(reader.read_u32::<LittleEndian>()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::U32(values)
                    }
                    K::F32 => {
                        let values = counter
                            .map(|_| Ok(reader.read_f32::<LittleEndian>()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::F32(values)
                    }
                    K::F64 => {
                        let values = counter
                            .map(|_| Ok(reader.read_f64::<LittleEndian>()?))
                            .collect::<Result<Vec<_>>>()?;
                        F::F64(values)
                    }
                };

                Ok(field)
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(fields))
    }

    fn read_line<R: BufRead>(reader: &mut R, field_defs: &Schema) -> Result<Self> {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();

        {
            let expect = field_defs.iter().map(|def| def.count as usize).sum();
            if tokens.len() != expect {
                return Err(Error::new_text_token_mismatch_error(expect, tokens.len()));
            }
        }

        let mut tokens_iter = tokens.into_iter();

        let fields: Vec<Field> = field_defs
            .iter()
            .map(|def| -> Result<_, Error> {
                let FieldDef { kind, count, .. } = *def;

                let count = count as usize;

                let field = match kind {
                    ValueKind::I8 => {
                        let values: Vec<i8> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::I8(values)
                    }
                    ValueKind::I16 => {
                        let values: Vec<i16> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::I16(values)
                    }
                    ValueKind::I32 => {
                        let values: Vec<i32> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::I32(values)
                    }
                    ValueKind::U8 => {
                        let values: Vec<u8> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::U8(values)
                    }
                    ValueKind::U16 => {
                        let values: Vec<u16> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::U16(values)
                    }
                    ValueKind::U32 => {
                        let values: Vec<u32> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::U32(values)
                    }
                    ValueKind::F32 => {
                        let values: Vec<f32> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::F32(values)
                    }
                    ValueKind::F64 => {
                        let values: Vec<f64> = (&mut tokens_iter)
                            .map(|token| token.parse())
                            .take(count)
                            .try_collect()?;
                        Field::F64(values)
                    }
                };

                Ok(field)
            })
            .try_collect()?;

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

    fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &Schema) -> Result<Self> {
        let value = reader.read_u8()?;
        Ok(value)
    }

    fn read_line<R: BufRead>(reader: &mut R, _field_defs: &Schema) -> Result<Self> {
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

    fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &Schema) -> Result<Self> {
        let value = reader.read_i8()?;
        Ok(value)
    }

    fn read_line<R: BufRead>(reader: &mut R, _field_defs: &Schema) -> Result<Self> {
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

            fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &Schema) -> Result<Self> {
                let value = reader.$read::<LittleEndian>()?;
                Ok(value)
            }

            fn read_line<R: BufRead>(reader: &mut R, _field_defs: &Schema) -> Result<Self> {
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
