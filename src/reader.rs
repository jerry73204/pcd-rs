//! [Reader](crate::seq_reader::Reader) lets you load points sequentially with
//! [Iterator](std::iter::Iterator) interface. The points are stored in
//! types implementing [PcdDeserialize](crate::record::PcdDeserialize) trait.
//! See [record](crate::record) moduel doc to implement your own point type.
//!
//! ```rust
//! use failure::Fallible;
//! use pcd_rs::{
//!     prelude::*,
//!     seq_reader::ReaderBuilder,
//!     PcdDeserialize,
//! };
//! use std::path::Path;
//!
//! #[derive(PcdDeserialize)]
//! pub struct Point {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//!     rgb: f32,
//! }
//!
//! fn main() -> Fallible<()> {
//!     let reader = ReaderBuilder::<Point, _>::open("test_files/ascii.pcd")?;
//!     let points = reader.collect::<Fallible<Vec<_>>>()?;
//!     assert_eq!(points.len(), 213);
//!     Ok(())
//! }
//! ```

use crate::{
    error::PCDError,
    metas::{DataKind, FieldDef, PCDMeta},
    record::PcdDeserialize,
};
use failure::Fallible;
use std::{
    fs::File,
    io::{prelude::*, BufReader, Cursor},
    marker::PhantomData,
    path::Path,
};

/// A reader type that loads points from PCD data.
pub struct Reader<Record, R>
where
    R: Read,
{
    meta: PCDMeta,
    record_count: usize,
    finished: bool,
    reader: R,
    _phantom: PhantomData<Record>,
}

impl<R, Record> Reader<Record, R>
where
    R: BufRead,
{
    /// Get meta data.
    pub fn meta(&self) -> &PCDMeta {
        &self.meta
    }
}

impl<R, Record> Iterator for Reader<Record, R>
where
    R: BufRead,
    Record: PcdDeserialize,
{
    type Item = Fallible<Record>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let record_result = match self.meta.data {
            DataKind::ASCII => Record::read_line(&mut self.reader, &self.meta.field_defs),
            DataKind::Binary => Record::read_chunk(&mut self.reader, &self.meta.field_defs),
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

/// A builder type that builds [Reader](crate::seq_reader::Reader).
pub struct ReaderBuilder;

impl ReaderBuilder {
    pub fn from_bytes<Record>(buf: &[u8]) -> Fallible<Reader<Record, BufReader<Cursor<&[u8]>>>>
    where
        Record: PcdDeserialize,
    {
        let reader = BufReader::new(Cursor::new(buf));
        Ok(Self::from_reader(reader)?)
    }

    pub fn from_path<P, Record>(path: P) -> Fallible<Reader<Record, BufReader<File>>>
    where
        Record: PcdDeserialize,
        P: AsRef<Path>,
    {
        let file = BufReader::new(File::open(path.as_ref())?);
        Ok(Self::from_reader(file)?)
    }

    pub fn from_reader<R, Record>(mut reader: R) -> Fallible<Reader<Record, R>>
    where
        Record: PcdDeserialize,
        R: BufRead,
    {
        let mut line_count = 0;
        let meta = crate::utils::load_meta(&mut reader, &mut line_count)?;

        // Checks whether the record schema matches the file meta
        if !Record::is_dynamic() {
            let record_spec = Record::read_spec();

            let mismatch_error =
                PCDError::new_schema_mismatch_error(record_spec.as_slice(), &meta.field_defs);

            if record_spec.len() != meta.field_defs.len() {
                return Err(mismatch_error.into());
            }

            for (record_field, meta_field) in record_spec.into_iter().zip(meta.field_defs.iter()) {
                let (name_opt, record_kind, record_count_opt) = record_field;
                let FieldDef {
                    name: meta_name,
                    kind: meta_kind,
                    count: meta_count,
                } = meta_field;

                if record_kind != *meta_kind {
                    return Err(mismatch_error.into());
                }

                if let Some(name) = &name_opt {
                    if name != meta_name {
                        return Err(mismatch_error.into());
                    }
                }

                if let Some(record_count) = record_count_opt {
                    if record_count != *meta_count as usize {
                        return Err(mismatch_error.into());
                    }
                }
            }
        }

        let pcd_reader = Reader {
            meta,
            reader,
            record_count: 0,
            finished: false,
            _phantom: PhantomData,
        };

        Ok(pcd_reader)
    }
}
