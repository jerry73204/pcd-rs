//! PCD point cloud file parser for Rust.
//!
//! `pcd-rs` allows you to parse PCD point cloud data from a file,
//! a path, or a binary buffer. The reader implements `Iterator` to
//! let you iterate over points with ease.
//!
//! ```rust
//! use failure::Fallible;
//! use pcd_rs::SeqReaderOptions;
//! use std::path::Path;
//!
//! fn main() -> Fallible<()> {
//!     let path = Path::new("test_files/ascii.pcd");
//!     let reader = SeqReaderOptions::from_path(path)?;
//!
//!     // Get meta data
//!     let meta = reader.meta();
//!
//!     // Scan all points
//!     let points = reader.collect::<Fallible<Vec<_>>>()?;
//!
//!     Ok(())
//! }
//! ```

extern crate byteorder;
#[macro_use]
extern crate failure;

pub mod error;
mod seq_reader;
mod utils;

pub use seq_reader::{SeqReader, SeqReaderOptions};

/// The struct keep meta data of PCD file.
#[derive(Debug)]
pub struct PCDMeta {
    pub version: String,
    pub width: u64,
    pub height: u64,
    pub viewpoint: Vec<u64>,
    pub num_points: u64,
    pub data: DataKind,
    pub field_defs: Vec<FieldDef>,
}

/// The enum specifies one of signed, unsigned integers, and floating point number type to the field.
#[derive(Debug)]
enum TypeKind {
    I,
    U,
    F,
}

/// The enum indicates whether the point cloud data is encoded in ASCII or binary.
#[derive(Debug)]
pub enum DataKind {
    ASCII,
    Binary,
}

/// The enum specifies the exact type for each PCD field.
#[derive(Debug)]
pub enum ValueKind {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
}

/// The enum holds the exact value of each PCD field.
#[derive(Debug)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    F64(f64),
    U8V(Vec<u8>),
    U16V(Vec<u16>),
    U32V(Vec<u32>),
    I8V(Vec<i8>),
    I16V(Vec<i16>),
    I32V(Vec<i32>),
    F32V(Vec<f32>),
    F64V(Vec<f64>),
}

/// Define the properties of a PCD field.
#[derive(Debug)]
pub struct FieldDef {
    name: String,
    kind: ValueKind,
    count: u64,
}
