use crate::{error::PCDError, DataKind, FieldDef, PCDMeta, PCDRecordRead};
use failure::Fallible;
use std::{
    fs::File,
    io::{prelude::*, BufReader, Cursor},
    marker::PhantomData,
    path::Path,
};

pub struct SeqReader<R: BufRead, T: PCDRecordRead> {
    meta: PCDMeta,
    record_count: usize,
    finished: bool,
    reader: R,
    _phantom: PhantomData<T>,
}

impl<R: BufRead, T: PCDRecordRead> SeqReader<R, T> {
    /// Get meta data.
    pub fn meta(&self) -> &PCDMeta {
        &self.meta
    }
}

impl<R: BufRead, T: PCDRecordRead> Iterator for SeqReader<R, T> {
    type Item = Fallible<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let record_result = match self.meta.data {
            DataKind::ASCII => T::read_line(&mut self.reader, &self.meta.field_defs),
            DataKind::Binary => T::read_chunk(&mut self.reader, &self.meta.field_defs),
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

pub struct SeqReaderBuilder;

impl SeqReaderBuilder {
    /// Load PCD data from a reader implementing [BufRead](std::io::BufRead) trait.
    pub fn from_buf_reader<R: BufRead, T: PCDRecordRead>(
        mut reader: R,
    ) -> Fallible<SeqReader<R, T>> {
        let mut line_count = 0;
        let meta = crate::utils::load_meta(&mut reader, &mut line_count)?;

        let record_spec = T::read_spec();
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

    /// Load PCD data from a reader
    pub fn from_reader<Rd: Read, T: PCDRecordRead>(
        reader: Rd,
    ) -> Fallible<SeqReader<BufReader<Rd>, T>> {
        SeqReaderBuilder::from_buf_reader(BufReader::new(reader))
    }

    /// Parse PCD data buffer
    pub fn from_buffer<T: PCDRecordRead>(
        buf: &[u8],
    ) -> Fallible<SeqReader<BufReader<Cursor<&[u8]>>, T>> {
        let reader = Cursor::new(buf);
        Ok(SeqReaderBuilder::from_reader(reader)?)
    }

    /// Load PCD data given by file path
    pub fn open<P: AsRef<Path>, T: PCDRecordRead>(
        path: P,
    ) -> Fallible<SeqReader<BufReader<File>, T>> {
        let file = File::open(path.as_ref())?;
        Ok(SeqReaderBuilder::from_reader(file)?)
    }
}
