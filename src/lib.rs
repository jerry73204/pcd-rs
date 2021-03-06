//! Read and write PCD file format.
//!
//! This crate allows you to read and write PCD point cloud.

pub extern crate anyhow;
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
#[cfg(feature = "derive")]
pub use pcd_rs_derive::{PcdDeserialize, PcdSerialize};
pub use reader::{Reader, ReaderBuilder};
pub use record::{DynRecord, Field, PcdDeserialize, PcdSerialize};
pub use writer::{Writer, WriterBuilder};
