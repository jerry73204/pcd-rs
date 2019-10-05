//! [SeqReader](crate::seq_reader::SeqReader) lets you load points sequentially with
//! [Iterator](std::iter::Iterator) interface. The points are stored in
//! types implementing [PCDRecordRead](crate::record::PCDRecordRead) trait.
//! See [record](crate::record) moduel doc to implement your own point type.
//!
//! ```rust
//! use failure::Fallible;
//! use pcd_rs::{
//!     prelude::*,
//!     seq_reader::SeqReaderBuilder,
//!     PCDRecordRead,
//! };
//! use std::path::Path;
//!
//! #[derive(PCDRecordRead)]
//! pub struct Point {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//!     rgb: f32,
//! }
//!
//! fn main() -> Fallible<()> {
//!     let reader = SeqReaderBuilder::<Point, _>::open("test_files/ascii.pcd")?;
//!     let points = reader.collect::<Fallible<Vec<_>>>()?;
//!     assert_eq!(points.len(), 213);
//!     Ok(())
//! }
//! ```

use crate::{
    error::PCDError,
    record::{PCDRecordRead, UntypedRecord},
    DataKind, FieldDef, PCDMeta, SchemaKind, TypedSchema, UntypedSchema,
};
use failure::Fallible;
use std::{
    fs::File,
    io::{prelude::*, BufReader, Cursor},
    marker::PhantomData,
    path::Path,
};

/// A reader type that loads points from PCD data.
pub struct SeqReader<Reader, Record, Kind>
where
    Reader: BufRead,
    Kind: SchemaKind,
{
    meta: PCDMeta,
    record_count: usize,
    finished: bool,
    reader: Reader,
    _phantom: PhantomData<(Reader, Record, Kind)>,
}

impl<Reader, Record, Kind> SeqReader<Reader, Record, Kind>
where
    Reader: BufRead,
    Kind: SchemaKind,
{
    /// Get meta data.
    pub fn meta(&self) -> &PCDMeta {
        &self.meta
    }
}

impl<Reader, Record> Iterator for SeqReader<Reader, Record, TypedSchema>
where
    Reader: BufRead,
    Record: PCDRecordRead,
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

impl<Reader> Iterator for SeqReader<Reader, UntypedRecord, UntypedSchema>
where
    Reader: BufRead,
{
    type Item = Fallible<UntypedRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let record_result = match self.meta.data {
            DataKind::ASCII => UntypedRecord::read_line(&mut self.reader, &self.meta.field_defs),
            DataKind::Binary => UntypedRecord::read_chunk(&mut self.reader, &self.meta.field_defs),
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

pub trait SeqReaderBuilderEx<Record, Kind>
where
    Kind: SchemaKind,
{
    /// Parse PCD data buffer.
    fn from_buffer(buf: &[u8]) -> Fallible<SeqReader<BufReader<Cursor<&[u8]>>, Record, Kind>>;

    /// Load PCD data from a path to file.
    fn open<P>(path: P) -> Fallible<SeqReader<BufReader<File>, Record, Kind>>
    where
        P: AsRef<Path>;

    /// Load PCD data from a reader that implements [BufRead](std::io::BufRead) trait.
    fn from_reader<Reader>(reader: Reader) -> Fallible<SeqReader<Reader, Record, Kind>>
    where
        Reader: BufRead;
}

/// A builder type that builds [SeqReader](crate::seq_reader::SeqReader).
pub struct SeqReaderBuilder<Record, Kind>
where
    Kind: SchemaKind,
{
    _phantom: PhantomData<(Record, Kind)>,
}

impl<Record> SeqReaderBuilderEx<Record, TypedSchema> for SeqReaderBuilder<Record, TypedSchema>
where
    Record: PCDRecordRead,
{
    fn from_buffer(
        buf: &[u8],
    ) -> Fallible<SeqReader<BufReader<Cursor<&[u8]>>, Record, TypedSchema>> {
        let reader = BufReader::new(Cursor::new(buf));
        Ok(SeqReaderBuilder::from_reader(reader)?)
    }

    fn open<P>(path: P) -> Fallible<SeqReader<BufReader<File>, Record, TypedSchema>>
    where
        P: AsRef<Path>,
    {
        let file = BufReader::new(File::open(path.as_ref())?);
        Ok(SeqReaderBuilder::from_reader(file)?)
    }

    fn from_reader<Reader>(mut reader: Reader) -> Fallible<SeqReader<Reader, Record, TypedSchema>>
    where
        Reader: BufRead,
    {
        let mut line_count = 0;
        let meta = crate::utils::load_meta(&mut reader, &mut line_count)?;

        // Checks whether the record schema matches the file meta
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

        let pcd_reader = SeqReader {
            meta,
            reader,
            record_count: 0,
            finished: false,
            _phantom: PhantomData,
        };

        Ok(pcd_reader)
    }
}

impl SeqReaderBuilderEx<UntypedRecord, UntypedSchema>
    for SeqReaderBuilder<UntypedRecord, UntypedSchema>
{
    fn from_buffer(
        buf: &[u8],
    ) -> Fallible<SeqReader<BufReader<Cursor<&[u8]>>, UntypedRecord, UntypedSchema>> {
        let reader = BufReader::new(Cursor::new(buf));
        Ok(SeqReaderBuilder::from_reader(reader)?)
    }

    fn open<P>(path: P) -> Fallible<SeqReader<BufReader<File>, UntypedRecord, UntypedSchema>>
    where
        P: AsRef<Path>,
    {
        let file = BufReader::new(File::open(path.as_ref())?);
        Ok(SeqReaderBuilder::from_reader(file)?)
    }

    fn from_reader<Reader>(
        mut reader: Reader,
    ) -> Fallible<SeqReader<Reader, UntypedRecord, UntypedSchema>>
    where
        Reader: BufRead,
    {
        let mut line_count = 0;
        let meta = crate::utils::load_meta(&mut reader, &mut line_count)?;

        let pcd_reader = SeqReader {
            meta,
            reader,
            record_count: 0,
            finished: false,
            _phantom: PhantomData,
        };

        Ok(pcd_reader)
    }
}
