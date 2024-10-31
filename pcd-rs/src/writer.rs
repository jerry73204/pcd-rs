//! Types for writing PCD data.
//!
//! [Writer](crate::writer::Writer) lets you write points sequentially to
//! PCD file or writer given by user. The written point type must implement
//! [PcdSerialize](crate::record::PcdSerialize) trait.
//! See [record](crate::record) moduel doc to implement your own point type.
#![cfg_attr(
    feature = "derive",
    doc = r##"
```rust
use eyre::Result;
use pcd_rs::{DataKind, PcdSerialize, Writer, WriterInit};
use std::path::Path;

#[derive(PcdSerialize)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32,
}

fn main() -> Result<()> {
    let mut writer: Writer<Point, _> = WriterInit {
        height: 300,
        width: 1,
        viewpoint: Default::default(),
        data_kind: DataKind::Ascii,
        schema: None,
    }
    .create("test_files/dump.pcd")?;

    let point = Point {
        x: 3.14159,
        y: 2.71828,
        z: -5.0,
    };

    writer.push(&point)?;
    writer.finish()?;

#  std::fs::remove_file("test_files/dump.pcd").unwrap();

    Ok(())
}
```
"##
)]

use crate::{
    metas::{DataKind, FieldDef, Schema, ValueKind, ViewPoint},
    record::{DynRecord, PcdSerialize},
    Error, Result,
};
use std::{
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufWriter, SeekFrom},
    marker::PhantomData,
    path::Path,
};

/// The `DynReader` struct writes points with schema determined in runtime.
pub type DynWriter<W> = Writer<DynRecord, W>;

/// A builder type that builds [Writer](crate::writer::Writer).
pub struct WriterInit {
    pub width: u64,
    pub height: u64,
    pub viewpoint: ViewPoint,
    pub data_kind: DataKind,
    pub schema: Option<Schema>,
}

impl WriterInit {
    /// Builds new [Writer](crate::writer::Writer) object from a writer.
    /// The writer must implement both [Write](std::io::Write) and [Write](std::io::Seek)
    /// traits.
    pub fn build_from_writer<Record: PcdSerialize, W: Write + Seek>(
        self,
        writer: W,
    ) -> Result<Writer<Record, W>, Error> {
        let record_spec = if Record::is_dynamic() {
            // Check if the schema is set.
            let Some(schema) = self.schema else {
                return Err(Error::new_invalid_writer_configuration_error(
                    "The schema is not set on the writer. It is required for the dynamic record type."
                ));
            };

            schema
        } else {
            if self.schema.is_some() {
                return Err(Error::new_invalid_writer_configuration_error(
                    "schema should not be set for static record type",
                ));
            }
            Record::write_spec()
        };

        let seq_writer = Writer::new(
            self.width,
            self.height,
            self.data_kind,
            self.viewpoint,
            record_spec,
            writer,
        )?;
        Ok(seq_writer)
    }

    /// Builds new [Writer](crate::writer::Writer) by creating a new file.
    pub fn create<Record, P>(self, path: P) -> Result<Writer<Record, BufWriter<File>>>
    where
        Record: PcdSerialize,
        P: AsRef<Path>,
    {
        let writer = BufWriter::new(File::create(path.as_ref())?);
        let seq_writer = self.build_from_writer(writer)?;
        Ok(seq_writer)
    }
}

/// The `Writer` struct writes points in type `T` to writer `W`.
pub struct Writer<T, W>
where
    W: Write + Seek,
{
    data_kind: DataKind,
    record_spec: Schema,
    writer: W,
    num_records: usize,
    points_arg_begin: u64,
    points_arg_width: usize,
    finished: bool,
    _phantom: PhantomData<T>,
}

impl<W, Record> Writer<Record, W>
where
    Record: PcdSerialize,
    W: Write + Seek,
{
    fn new(
        width: u64,
        height: u64,
        data_kind: DataKind,
        viewpoint: ViewPoint,
        record_spec: Schema,
        mut writer: W,
    ) -> Result<Self, Error> {
        macro_rules! ensure {
            ($cond:expr, $desc:expr) => {
                if !$cond {
                    return Err(Error::new_invalid_writer_configuration_error($desc));
                }
            };
        }

        // Run sanity check on the schema.
        {
            for FieldDef { name, count, .. } in &record_spec {
                if name.is_empty() {}
                ensure!(!name.is_empty(), "field name must not be empty");
                ensure!(*count > 0, "The field count must be nonzero");
            }

            let names: HashSet<_> = record_spec.iter().map(|field| &field.name).collect();
            ensure!(
                names.len() == record_spec.len(),
                "schema names must be unique"
            );
        }

        let (points_arg_begin, points_arg_width) = {
            let fields_args: Vec<_> = record_spec
                .iter()
                .map(|field| field.name.to_owned())
                .collect();

            let size_args: Vec<_> = record_spec
                .iter()
                .map(|field| {
                    use ValueKind::*;
                    let size = match field.kind {
                        U8 | I8 => 1,
                        U16 | I16 => 2,
                        U32 | I32 | F32 => 4,
                        F64 => 8,
                    };
                    size.to_string()
                })
                .collect();

            let type_args: Vec<_> = record_spec
                .iter()
                .map(|field| {
                    use ValueKind::*;
                    match field.kind {
                        U8 | U16 | U32 => "U",
                        I8 | I16 | I32 => "I",
                        F32 | F64 => "F",
                    }
                })
                .collect();

            let count_args: Vec<_> = record_spec
                .iter()
                .map(|field| field.count.to_string())
                .collect();

            let viewpoint_args: Vec<_> = {
                [
                    viewpoint.tx,
                    viewpoint.ty,
                    viewpoint.tz,
                    viewpoint.qw,
                    viewpoint.qx,
                    viewpoint.qy,
                    viewpoint.qz,
                ]
                .iter()
                .map(|value| value.to_string())
                .collect()
            };

            let points_arg_width = (usize::max_value() as f64).log10().floor() as usize + 1;

            writeln!(writer, "# .PCD v.7 - Point Cloud Data file format")?;
            writeln!(writer, "VERSION .7")?;
            writeln!(writer, "FIELDS {}", fields_args.join(" "))?;
            writeln!(writer, "SIZE {}", size_args.join(" "))?;
            writeln!(writer, "TYPE {}", type_args.join(" "))?;
            writeln!(writer, "COUNT {}", count_args.join(" "))?;
            writeln!(writer, "WIDTH {}", width)?;
            writeln!(writer, "HEIGHT {}", height)?;
            writeln!(writer, "VIEWPOINT {}", viewpoint_args.join(" "))?;

            write!(writer, "POINTS ")?;
            let points_arg_begin = writer.seek(SeekFrom::Current(0))?;
            writeln!(writer, "{:width$}", " ", width = points_arg_width)?;

            match data_kind {
                DataKind::Binary => writeln!(writer, "DATA binary")?,
                DataKind::Ascii => writeln!(writer, "DATA ascii")?,
            }

            (points_arg_begin, points_arg_width)
        };

        let seq_writer = Self {
            data_kind,
            record_spec,
            writer,
            num_records: 0,
            points_arg_begin,
            points_arg_width,
            finished: false,
            _phantom: PhantomData,
        };
        Ok(seq_writer)
    }

    /// Finish the writer.
    ///
    /// The method consumes the writer must be called once when finished.
    /// Otherwise it will panic when it drops.
    pub fn finish(mut self) -> Result<()> {
        self.writer.seek(SeekFrom::Start(self.points_arg_begin))?;
        write!(
            self.writer,
            "{:<width$}",
            self.num_records,
            width = self.points_arg_width
        )?;
        self.finished = true;
        Ok(())
    }

    /// Writes a new point to PCD data.
    pub fn push(&mut self, record: &Record) -> Result<()> {
        match self.data_kind {
            DataKind::Binary => record.write_chunk(&mut self.writer, &self.record_spec)?,
            DataKind::Ascii => record.write_line(&mut self.writer, &self.record_spec)?,
        }

        self.num_records += 1;
        Ok(())
    }
}

impl<W, Record> Drop for Writer<Record, W>
where
    W: Write + Seek,
{
    fn drop(&mut self) {
        if !self.finished {
            panic!("call finish() before Writer drops");
        }
    }
}
