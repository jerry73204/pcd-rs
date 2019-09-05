// The module defines [PCDRecord](crate::PCDRecord) trait, which is
// analogous to a _point_ in PCD data.
// Any object scanned by readers and written by writers should implement
// this trait.
//
// To implement PCDRecord on a struct, it's as simple as using derive macro:
// ```rust
// use pcd_rs::PCDRecord;
//
// #[derive(PCDRecord)]
// pub struct TimestampedPoint {
//     x: f32,
//     y: f32,
//     z: f32,
//     timestamp: u32,
// }
// ```
//
// The implementation can be derived if:
// - The struct is either a normal struct or tuple struct. Unit struct is not allowed.
// - Each field type is primitive, array of primitive type, or Vec of primitive type.
// - Supported primitive types are u8, u16, u32, i8, i16, i32, f32, f64.

use crate::{FieldDef, ValueKind};
use byteorder::{LittleEndian, ReadBytesExt};
use failure::Fallible;
use std::io::prelude::*;

// PCDRecord is analogous to a _point_ in PCD data.
//
// In ASCII mode, a record is represented a line of data, while
// in binary mode, it is a sequence of binary integers or floating numbers.

pub trait PCDRecordRead: Sized {
    fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)>;
    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
    fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
}

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

// TODO: Implement PCDrecordRead for array types using cons generics
// macro_rules! impl_array {
//     ($ty:ty, $kind:ident, $read:ident) => {
//         impl<const N: usize> PCDRecordRead for [$ty; N] {
//             fn read_spec() -> Vec<(Option<String>, ValueKind, Option<usize>)> {
//                 vec![(None, ValueKind::$kind, Some(N))]
//             }

//             fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//                 let mut array = [Default::default(); N];
//                 for index in 0..N {
//                     array[index] = reader.$read::<LittleEndian>()?;
//                 }
//                 Ok(array)
//             }

//             fn read_line<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//                 let mut line = String::new();
//                 reader.read_line(&mut line)?;
//                 let mut tokens = line.split_ascii_whitespace().into_iter();

//                 let expect_len = N;
//                 let (found_len, _) = tokens.size_hint();
//                 if expect_len != found_len {
//                     let error = PCDError::new_text_token_mismatch_error(expect_len, found_len);
//                     return Err(error.into());
//                 }

//                 let mut array = [Default::default(); N];
//                 for index in 0..N {
//                     array[index] = tokens.next().unwrap().parse::<$ty>()?
//                 }
//                 Ok(array)
//             }
//         }
//     };
// }

// impl_array!(u16, U16, read_u16);
