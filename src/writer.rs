use crate::{error::PCDError, DataKind, PCDRecordWrite, ValueKind, ViewPoint};
use failure::Fallible;
use regex::Regex;
use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    marker::PhantomData,
    path::Path,
};

pub struct SeqWriterBuilder<T: PCDRecordWrite> {
    width: u64,
    height: u64,
    viewpoint: ViewPoint,
    data_kind: DataKind,
    field_names: Vec<String>,
    record_spec: Vec<(ValueKind, usize)>,
    _phantom: PhantomData<T>,
}

impl<T: PCDRecordWrite> SeqWriterBuilder<T> {
    pub fn new(
        width: u64,
        height: u64,
        viewpoint: ViewPoint,
        data_kind: DataKind,
        field_names: Vec<String>,
    ) -> Fallible<SeqWriterBuilder<T>> {
        let record_spec = T::write_spec();

        let num_fields = record_spec.len();
        let num_names = field_names.len();

        // Check field names
        if record_spec.len() != field_names.len() {
            let error = PCDError::new_invalid_argument_error(&format!(
                "The record has {} fields, but only {} field names are provided",
                num_fields, num_names
            ));
            return Err(error.into());
        }
        verify_field_names(&field_names)?;

        let builder = SeqWriterBuilder {
            width,
            height,
            viewpoint,
            data_kind,
            field_names,
            record_spec,
            _phantom: PhantomData,
        };

        Ok(builder)
    }

    pub fn from_writer<R: Write + Seek>(self, writer: R) -> Fallible<SeqWriter<R, T>> {
        let seq_writer = SeqWriter::new(self, writer)?;
        Ok(seq_writer)
    }

    pub fn open_path<P: AsRef<Path>>(self, path: P) -> Fallible<SeqWriter<BufWriter<File>, T>> {
        let writer = BufWriter::new(File::open(path.as_ref())?);
        let seq_writer = self.from_writer(writer)?;
        Ok(seq_writer)
    }
}

pub struct SeqWriter<R: Write + Seek, T: PCDRecordWrite> {
    writer: R,
    builder: SeqWriterBuilder<T>,
}

impl<R: Write + Seek, T: PCDRecordWrite> SeqWriter<R, T> {
    pub fn new(builder: SeqWriterBuilder<T>, writer: R) -> Fallible<SeqWriter<R, T>> {
        let mut seq_writer = SeqWriter { builder, writer };
        seq_writer.write_meta()?;
        Ok(seq_writer)
    }

    pub fn write_meta(&mut self) -> Fallible<()> {
        writeln!(self.writer, "# .PCD v.7 - Point Cloud Data file format")?;
        writeln!(self.writer, "VERSION .7")?;
        writeln!(self.writer, "FIELDS {}", self.builder.field_names.join(" "))?;
        Ok(())
    }
}

fn verify_field_names(field_names: &[String]) -> Fallible<()> {
    let name_regex = Regex::new(r"^[[:word:]]+$")?;
    for name in field_names.iter() {
        let error = PCDError::new_invalid_argument_error(&format!(
            "Invalid field name: \"{}\". Name must be composed of word characters",
            name
        ));
        name_regex.find(name).ok_or(error)?;
    }
    Ok(())
}
