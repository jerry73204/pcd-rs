//! Types for reading PCD data.
//!
//! [Reader](crate::reader::Reader) lets you load points sequentially with
//! [Iterator](std::iter::Iterator) interface. The points are stored in
//! types implementing [PcdDeserialize](crate::record::PcdDeserialize) trait.
//! See [record](crate::record) moduel doc to implement your own point type.
#![cfg_attr(
    feature = "derive",
    doc = r##"
```rust
use pcd_rs::{PcdDeserialize, Reader};
use std::path::Path;

#[derive(PcdDeserialize)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
    rgb: f32,
}

fn main() -> pcd_rs::Result<()> {
    let reader = Reader::open("test_files/ascii.pcd")?;
    let points: Result<Vec<Point>> = reader.collect();
    assert_eq!(points?.len(), 213);
    Ok(())
}
```
"##
)]

use crate::{
    error::Error,
    lzf,
    metas::{DataKind, FieldDef, PcdMeta},
    record::{DynRecord, PcdDeserialize},
    Result,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    fs::File,
    io::{prelude::*, BufReader, Cursor},
    marker::PhantomData,
    path::Path,
};

/// The `DynReader` struct loads points with schema determined in runtime.
pub type DynReader<R> = Reader<DynRecord, R>;

/// The `Reader<T, R>` struct loads points into type `T` from reader `R`.
pub struct Reader<T, R>
where
    R: Read,
{
    meta: PcdMeta,
    record_count: usize,
    finished: bool,
    reader: R,
    decompressed_buffer: Option<Cursor<Vec<u8>>>,
    _phantom: PhantomData<T>,
}

impl<'a, Record> Reader<Record, BufReader<Cursor<&'a [u8]>>>
where
    Record: PcdDeserialize,
{
    pub fn from_bytes(buf: &'a [u8]) -> Result<Self> {
        let reader = BufReader::new(Cursor::new(buf));
        Self::from_reader(reader)
    }
}

impl<Record, R> Reader<Record, R>
where
    Record: PcdDeserialize,
    R: BufRead,
{
    pub fn from_reader(mut reader: R) -> Result<Self> {
        let mut line_count = 0;
        let meta = crate::utils::load_meta(&mut reader, &mut line_count)?;

        // Checks whether the record schema matches the file meta
        if !Record::is_dynamic() {
            let record_spec = Record::read_spec();

            macro_rules! bail {
                () => {
                    return Err(Error::new_reader_schema_mismatch_error(
                        record_spec.clone(),
                        meta.field_defs.fields.clone(),
                    ));
                };
            }

            if record_spec.len() != meta.field_defs.len() {
                bail!();
            }

            for (record_field, meta_field) in record_spec.iter().zip(meta.field_defs.iter()) {
                let (ref name_opt, record_kind, record_count_opt) = *record_field;
                let FieldDef {
                    name: ref meta_name,
                    kind: meta_kind,
                    count: meta_count,
                } = *meta_field;

                if record_kind != meta_kind {
                    bail!();
                }

                if let Some(name) = &name_opt {
                    if name != meta_name {
                        bail!();
                    }
                }

                if let Some(record_count) = record_count_opt {
                    if record_count != meta_count as usize {
                        bail!();
                    }
                }
            }
        }

        // For compressed data, read and decompress the entire data section
        let decompressed_buffer = if meta.data == DataKind::BinaryCompressed {
            // Read compressed size and uncompressed size
            let compressed_size = reader.read_u32::<LittleEndian>()?;
            let uncompressed_size = reader.read_u32::<LittleEndian>()?;

            if compressed_size == 0 && uncompressed_size == 0 {
                // Empty compressed data
                Some(Cursor::new(Vec::new()))
            } else {
                // Read compressed data
                let mut compressed_data = vec![0u8; compressed_size as usize];
                reader.read_exact(&mut compressed_data)?;

                // Decompress
                let decompressed = lzf::decompress(&compressed_data, uncompressed_size as usize)?;
                Some(Cursor::new(decompressed))
            }
        } else {
            None
        };

        let pcd_reader = Reader {
            meta,
            reader,
            record_count: 0,
            finished: false,
            decompressed_buffer,
            _phantom: PhantomData,
        };

        Ok(pcd_reader)
    }
}

impl<Record> Reader<Record, BufReader<File>>
where
    Record: PcdDeserialize,
{
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = BufReader::new(File::open(path.as_ref())?);
        Self::from_reader(file)
    }
}

impl<R, Record> Reader<Record, R>
where
    R: BufRead,
{
    /// Get meta data.
    pub fn meta(&self) -> &PcdMeta {
        &self.meta
    }
}

impl<R, Record> Iterator for Reader<Record, R>
where
    R: BufRead,
    Record: PcdDeserialize,
{
    type Item = Result<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        // Check if we've already read all points or if there are no points
        if self.record_count >= self.meta.num_points as usize {
            self.finished = true;
            return None;
        }

        let record_result = match self.meta.data {
            DataKind::Ascii => Record::read_line(&mut self.reader, &self.meta.field_defs),
            DataKind::Binary => Record::read_chunk(&mut self.reader, &self.meta.field_defs),
            DataKind::BinaryCompressed => {
                // Read from decompressed buffer
                if let Some(ref mut buffer) = self.decompressed_buffer {
                    Record::read_chunk(buffer, &self.meta.field_defs)
                } else {
                    return Some(Err(Error::ParseError {
                        line: 0,
                        desc: "Compressed data buffer not initialized".into(),
                    }));
                }
            }
        };

        match record_result {
            Ok(_) => {
                self.record_count += 1;
                if self.record_count == self.meta.num_points as usize {
                    self.finished = true;
                }
            }
            Err(_) => {
                self.finished = true;
            }
        }

        Some(record_result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.meta.num_points as usize;
        (size, Some(size))
    }
}
