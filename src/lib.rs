//! PCD point cloud file parser for Rust.
//!
//! `pcd-rs` allows you to parse PCD point cloud data from a file,
//! a path, or a binary buffer. The reader implements `Iterator` to
//! let you iterate over points with ease.
//!
//! ```rust
//! use failure::Fallible;
//! use pcd_rs::{SeqReaderBuilder, PCDRecord};
//! use std::path::Path;
//!
//! #[derive(PCDRecord)]
//! pub struct Point {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//!     w: f32,
//! }
//!
//! fn main() -> Fallible<()> {
//!     let reader = SeqReaderBuilder::open_path("test_files/ascii.pcd")?;
//!     let points = reader.collect::<Fallible<Vec<Point>>>()?;
//!     assert_eq!(points.len(), 213);
//!     Ok(())
//! }
//! ```

// #![feature(const_generics)]

pub extern crate byteorder;
#[macro_use]
pub extern crate failure;
extern crate pcd_rs_derive;

pub mod error;
mod record;
mod seq_reader;
mod utils;

#[doc(hidden)]
pub use pcd_rs_derive::*;
pub use record::PCDRecord;
pub use seq_reader::{SeqReader, SeqReaderBuilder};
#[doc(hidden)]

/// The struct keep meta data of PCD file.
#[derive(Debug)]
pub struct PCDMeta {
    pub version: String,
    pub width: u64,
    pub height: u64,
    pub viewpoint: Vec<u64>,
    pub num_records: u64,
    pub data: DataKind,
    pub field_defs: Vec<FieldDef>,
}

/// The enum specifies one of signed, unsigned integers, and floating point number type to the field.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TypeKind {
    I,
    U,
    F,
}

/// The enum indicates whether the point cloud data is encoded in ASCII or binary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataKind {
    ASCII,
    Binary,
}

/// The enum specifies the exact type for each PCD field.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Define the properties of a PCD field.
#[derive(Debug)]
pub struct FieldDef {
    pub name: String,
    pub kind: ValueKind,
    pub count: u64,
}
