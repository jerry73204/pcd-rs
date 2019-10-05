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
pub mod meta;
pub mod prelude;
pub mod record;
pub mod seq_reader;
pub mod seq_writer;
mod utils;

pub use pcd_rs_derive::{PCDRecordRead, PCDRecordWrite};
