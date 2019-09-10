//! Read and write PCD file format.
//!
//! `pcd-rs` allows you to read or write PCD point cloud data from either
//! a path or a binary buffer.
//!
//! - [seq_reader](crate::seq_reader) for reading PCD data
//! - [seq_writer](crate::seq_writer) for writing PCD data
//! - [record](crate::record) for building custom _point_ type

pub extern crate byteorder;
#[macro_use]
pub extern crate failure;
extern crate pcd_rs_derive;
extern crate regex;

pub mod error;
pub mod record;
pub mod seq_reader;
pub mod seq_writer;
mod utils;

pub use pcd_rs_derive::{PCDRecordRead, PCDRecordWrite};

/// The struct keep meta data of PCD file.
#[derive(Debug)]
pub struct PCDMeta {
    pub version: String,
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub num_points: u64,
    pub data: DataKind,
    pub field_defs: Vec<FieldDef>,
}

/// Represents VIEWPOINT field in meta data.
#[derive(Debug, Clone, PartialEq)]
pub struct ViewPoint {
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
    pub qw: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
}

impl Default for ViewPoint {
    fn default() -> Self {
        ViewPoint {
            tx: 0.0,
            ty: 0.0,
            tz: 0.0,
            qw: 1.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
        }
    }
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
