//! Read and write PCD file format.
//!
//! `pcd-rs` allows you to read or write PCD point cloud data from either
//! a path or a binary buffer.
//!
//! - [seq_reader](crate::seq_reader) for reading PCD data
//! - [seq_writer](crate::seq_writer) for writing PCD data
//! - [record](crate::record) for building custom _point_ type

#[macro_use]
pub extern crate failure;
pub extern crate byteorder;

pub mod error;
pub mod metas;
pub mod prelude;
pub mod reader;
pub mod record;
mod utils;
pub mod writer;

pub use error::PcdError;
pub use metas::{DataKind, FieldDef, PcdMeta, TypeKind, ValueKind, ViewPoint};
pub use pcd_rs_derive::{PcdDeserialize, PcdSerialize};
pub use reader::{Reader, ReaderBuilder};
pub use record::{DynRecord, Field, PcdDeserialize, PcdSerialize};
pub use writer::{Writer, WriterBuilder};
