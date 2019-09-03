//! The module defines [PCDRecord](crate::PCDRecord) trait, which is
//! analogous to a _point_ in PCD data.
//! Any object scanned by readers and written by writers should implement
//! this trait.
//!
//! To implement PCDRecord on a struct, it's as simple as using derive macro:
//! ```rust
//! use pcd_rs::PCDRecord;
//!
//! #[derive(PCDRecord)]
//! pub struct TimestampedPoint {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//!     timestamp: u32,
//! }
//! ```
//!
//! The implementation can be derived if:
//! - The struct is either a normal struct or tuple struct. Unit struct is not allowed.
//! - Each field type is primitive, array of primitive type, or Vec of primitive type.
//! - Supported primitive types are u8, u16, u32, i8, i16, i32, f32, f64.

use crate::{FieldDef, ValueKind};
use failure::Fallible;
use std::io::prelude::*;

// PCDRecord is analogous to a _point_ in PCD data.
//
// In ASCII mode, a record is represented a line of data, while
// in binary mode, it is a sequence of binary integers or floating numbers.

pub trait PCDRecordRead: Sized {
    fn read_spec() -> Vec<(ValueKind, Option<usize>)>;
    fn read_chunk<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
    fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self>;
}

pub trait PCDRecordWrite: Sized {
    fn write_spec() -> Vec<(ValueKind, usize)>;
    fn write_chunk<R: Write>(&self, writer: &mut R, field_names: &[String]) -> Fallible<()>;
    fn write_line<R: Write>(&self, writer: &mut R, field_names: &[String]) -> Fallible<()>;
}

// impl PCDRecord for u8 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::U8, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_u8()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_u8(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for u16 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::U16, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_u16::<LittleEndian>()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_u16::<LittleEndian>(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for u32 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::U32, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_u32::<LittleEndian>()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_u32::<LittleEndian>(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for i8 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::I8, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_i8()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_i8(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for i16 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::I16, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_i16::<LittleEndian>()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_i16::<LittleEndian>(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for i32 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::I32, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_i32::<LittleEndian>()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_i32::<LittleEndian>(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for f32 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::F32, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_f32::<LittleEndian>()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_f32::<LittleEndian>(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl PCDRecord for f64 {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::F64, Some(1))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let value = reader.read_f64::<LittleEndian>()?;
//         Ok(value)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         writer.write_f64::<LittleEndian>(*self)?;
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         Ok(line.parse()?)
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//         writeln!(writer, "{}", self)?;
//         Ok(())
//     }
// }

// impl<const N: usize> PCDRecord for [u8; N] {
//     fn record_spec() -> Vec<(ValueKind, Option<usize>)> {
//         vec![(ValueKind::U8, Some(N))]
//     }

//     fn read_chunk<R: BufRead>(reader: &mut R, _field_defs: &[FieldDef]) -> Fallible<Self> {
//         let mut array = [0; N];
//         for index in 0..N {
//             array[index] = reader.read_u8()?;
//         }
//         Ok(array)
//     }

//     fn write_chunk<R: Write>(&self, writer: &mut R, _field_defs: &[FieldDef]) -> Fallible<()> {
//         for value in self.iter() {
//             writer.write_u8(*value)?;
//         }
//         Ok(())
//     }

//     fn read_line<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Self> {
//         let mut line = String::new();
//         reader.read_line(&mut line)?;
//         let tokens = line.split_ascii_whitespace().collect::<Vec<_>>();
//         if tokens.len() != N {
//             let error = PCDError::new_text_token_mismatch_error(N, );
//         }
//     }

//     fn write_line<R: Write>(&self, writer: &mut R) -> Fallible<()> {
//     }
// }
