//! Read and write PCD file format.
//!
//! This crate allows you to read and write PCD point cloud.

#[doc(hidden)]
pub use anyhow;
#[doc(hidden)]
pub use byteorder;

pub mod error;
pub mod metas;
pub mod prelude;
pub mod reader;
pub mod record;
mod utils;
pub mod writer;

pub use error::Error;
pub use metas::{DataKind, FieldDef, PcdMeta, Schema, TypeKind, ValueKind, ViewPoint};
#[cfg(feature = "derive")]
pub use pcd_rs_derive::{PcdDeserialize, PcdSerialize};
pub use reader::{DynReader, Reader};
pub use record::{DynRecord, Field, PcdDeserialize, PcdSerialize};
pub use writer::{DynWriter, Writer, WriterInit};
