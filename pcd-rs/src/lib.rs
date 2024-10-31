//! Read and write PCD file format.
//!
//! This crate provides data serializer and deserializer for PCD
//! (Point Cloud Data) file format. The [DynReader] and [DynWriter]
//! can read and write PCD files with any valid schemas. It also
//! supports deserializing to static types if the `derive` feature is
//! enabled.
//!
//! # Supported Format Versions
//!
//! - 0.7
//! - Older versions are not supported yet.
//!
//!
//! # Any Schema Example
//!
//! In the case of any schema, the points are represented by an array
//! or a slice of [DynRecord]s, where is record wraps a sequence of
//! data [Field]s.
//!
//! ## Reader
//!
//! The reader is created by [DynReader::open()], returning an
//! iterator, which generates a sequence of
//! [Result\<DynRecord\>](DynRecord).
//!
//! ```rust
//! # fn main() -> pcd_rs::pcd_rsResult<()>  {
//! use pcd_rs::DynReader;
//!
//! // Declare the reader
//! let reader = DynReader::open("test_files/binary.pcd")?;
//!
//! // The reader itself is an iterator of records
//! let points: Result<Vec<_>> = reader.collect();
//! println!("There are {} points found.", points?.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Writer
//!
//! The writer is first configured by [WriterInit], and then call
//! [WriterInit::create()] to construct the [DynWriter]. The
//! [.push()](DynWriter::push) is used to append the data to the
//! writer. The writer must be finished by
//! [.finish()](DynWriter::finish) in the end.
//!
//! ```rust
//! # fn main() -> pcd_rs::pcd_rs::Result<()>  {
//! use pcd_rs::{DataKind, DynRecord, DynWriter, Field, Schema, ValueKind, WriterInit};
//!
//! // Declare point data
//! let points = [
//!     DynRecord(vec![
//!         Field::F32(vec![3.14159]),
//!         Field::U8(vec![2, 1, 7]),
//!         Field::I32(vec![-5]),
//!     ]),
//!     DynRecord(vec![
//!         Field::F32(vec![-0.0]),
//!         Field::U8(vec![254, 6, 98]),
//!         Field::I32(vec![7]),
//!     ]),
//!     DynRecord(vec![
//!         Field::F32(vec![5.6]),
//!         Field::U8(vec![4, 0, 111]),
//!         Field::I32(vec![-100000]),
//!     ]),
//! ];
//!
//! // Declare the schema
//! let schema = vec![
//!     ("x", ValueKind::F32, 1),
//!     ("y", ValueKind::U8, 3),
//!     ("z", ValueKind::I32, 1),
//! ];
//!
//! // Build a writer
//! let mut writer: DynWriter<_> = WriterInit {
//!     width: 300,
//!     height: 1,
//!     viewpoint: Default::default(),
//!     data_kind: DataKind::Ascii,
//!     schema: Some(Schema::from_iter(schema)),
//! }
//! .create("test_files/dump_ascii_untyped.pcd")?;
//!
//! // Send the points to the writer
//! for point in points {
//!     writer.push(&point)?;
//! }
//!
//! // Finalize the writer
//! writer.finish()?;
//! # Ok(())
//! # }
//! ```
#![cfg_attr(
    feature = "derive",
    doc = r##"
# Static Schema Example

The serde-like derives [PcdSerialize] and [PcdDeserialize] allows the
[Reader] and [Writer] to read from to write to the annotated
types. Both are available if the `derive` feature is enabled. The type
must be a `struct` with named fields, where each field type is either
a primitive type, an array or a `Vec`.

## Reader

The reader is constructed by [Reader::open()], which itself is an
iterator. The [.push()](Writer::push) is used to append the data to
the writer. The writer must be finished by [.finish()](Writer::finish)
in the end.

```rust
# pub fn main() -> pcd_rs::Result<()> {
use pcd_rs::{PcdDeserialize, Reader};

#[derive(PcdDeserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub rgb: f32,
}

let reader = Reader::open("test_files/ascii.pcd")?;
let points: Result<Vec<Point>> = reader.collect();
println!("{} points found", points?.len());
# Ok(())
# }
```

## Writer

The writer is configured by [WriterInit] and then created by
[WriterInit::create()].

```rust
# pub fn main() -> pcd_rs::Result<()> {
use pcd_rs::{DataKind, PcdDeserialize, PcdSerialize, WriterInit};

#[derive(PcdSerialize)]
pub struct Point {
    #[pcd(rename = "new_x")]
    x: f32,
    y: [u8; 3],
    z: i32,
}

// point data
let points = [
    Point {
        x: 3.14159,
        y: [2, 1, 7],
        z: -5,
    },
    Point {
        x: -0.0,
        y: [254, 6, 98],
        z: 7,
    },
    Point {
        x: 5.6,
        y: [4, 0, 111],
        z: -100000,
    },
];

// serialize points
let mut writer = WriterInit {
    width: 300,
    height: 1,
    viewpoint: Default::default(),
    data_kind: DataKind::Ascii,
    schema: None,
}
.create("test_files/dump_ascii_static.pcd")?;

for point in points {
    writer.push(&point)?;
}

writer.finish()?;
# Ok(())
# }
```

# Derives

Both [PcdSerialize] and [PcdDeserialize] supports the following field
attributes.

- `#[pcd(rename = "NEW_NAME")]` sets the field name on the written PCD data.
- `#[pcd(ignore)]` instructs the de/serializer to ignore the field.
"##
)]

#[doc(hidden)]
pub use byteorder;

pub mod error;
pub mod metas;
pub mod prelude;
pub mod reader;
pub mod record;
pub mod traits;
mod utils;
pub mod writer;

pub use error::{Error, Result};
pub use metas::{DataKind, FieldDef, PcdMeta, Schema, TypeKind, ValueKind, ViewPoint};
#[cfg(feature = "derive")]
pub use pcd_rs_derive::{PcdDeserialize, PcdSerialize};
pub use reader::{DynReader, Reader};
pub use record::{DynRecord, Field, PcdDeserialize, PcdSerialize};
pub use traits::Value;
pub use writer::{DynWriter, Writer, WriterInit};
