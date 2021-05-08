//! Types for writing PCD data.
//!
//! [Writer](crate::writer::Writer) lets you write points sequentially to
//! PCD file or writer given by user. The written point type must implement
//! [PcdSerialize](crate::record::PcdSerialize) trait.
//! See [record](crate::record) moduel doc to implement your own point type.
//!
//! ```rust
//! use anyhow::Result;
//! use pcd_rs::{DataKind, PcdSerialize, Writer, WriterBuilder};
//! use std::path::Path;
//!
//! #[derive(PcdSerialize)]
//! pub struct Point {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! fn main() -> Result<()> {
//!     let viewpoint = Default::default();
//!     let kind = DataKind::Ascii;
//!     let mut writer: Writer<Point, _> =
//!         WriterBuilder::new(300, 1, viewpoint, kind)?.create("test_files/dump.pcd")?;
//!
//!     let point = Point {
//!         x: 3.14159,
//!         y: 2.71828,
//!         z: -5.0,
//!     };
//!
//!     writer.push(&point)?;
//!     writer.finish()?;
//!     Ok(())
//! }
//! ```

use crate::{
    metas::{DataKind, FieldDef, ValueKind, ViewPoint},
    record::{DynRecord, PcdSerialize},
};
use anyhow::{bail, Result};
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
pub struct WriterBuilder {
    width: u64,
    height: u64,
    viewpoint: ViewPoint,
    data_kind: DataKind,
    record_spec: Option<Vec<(String, ValueKind, usize)>>,
}

impl WriterBuilder {
    fn write_meta<W>(&self, writer: &mut W) -> Result<(u64, usize)>
    where
        W: Write + Seek,
    {
        let record_spec = self.record_spec.as_ref().unwrap();

        let fields_args = record_spec
            .iter()
            .map(|(name, _, _)| name.to_owned())
            .collect::<Vec<_>>();

        let size_args = record_spec
            .iter()
            .map(|(_, kind, _)| {
                use ValueKind::*;
                let size = match kind {
                    U8 | I8 => 1,
                    U16 | I16 => 2,
                    U32 | I32 | F32 => 4,
                    F64 => 8,
                };
                size.to_string()
            })
            .collect::<Vec<_>>();

        let type_args = record_spec
            .iter()
            .map(|(_, kind, _)| {
                use ValueKind::*;
                match kind {
                    U8 | U16 | U32 => "U",
                    I8 | I16 | I32 => "I",
                    F32 | F64 => "F",
                }
            })
            .collect::<Vec<_>>();

        let count_args = record_spec
            .iter()
            .map(|(_, _, count)| count.to_string())
            .collect::<Vec<_>>();

        let viewpoint_args = {
            let viewpoint = &self.viewpoint;
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
            .collect::<Vec<_>>()
        };

        let points_arg_width = (usize::max_value() as f64).log10().floor() as usize + 1;

        writeln!(writer, "# .PCD v.7 - Point Cloud Data file format")?;
        writeln!(writer, "VERSION .7")?;
        writeln!(writer, "FIELDS {}", fields_args.join(" "))?;
        writeln!(writer, "SIZE {}", size_args.join(" "))?;
        writeln!(writer, "TYPE {}", type_args.join(" "))?;
        writeln!(writer, "COUNT {}", count_args.join(" "))?;
        writeln!(writer, "WIDTH {}", self.width)?;
        writeln!(writer, "HEIGHT {}", self.height)?;
        writeln!(writer, "VIEWPOINT {}", viewpoint_args.join(" "))?;

        write!(writer, "POINTS ")?;
        let points_arg_begin = writer.seek(SeekFrom::Current(0))?;
        writeln!(writer, "{:width$}", " ", width = points_arg_width)?;

        match self.data_kind {
            DataKind::Binary => writeln!(writer, "DATA binary")?,
            DataKind::Ascii => writeln!(writer, "DATA ascii")?,
        }

        Ok((points_arg_begin, points_arg_width))
    }

    /// Create new [WriterBuilder](crate::writer::WriterBuilder) that
    /// stores header data.
    pub fn new(width: u64, height: u64, viewpoint: ViewPoint, data_kind: DataKind) -> Result<Self> {
        let builder = Self {
            width,
            height,
            viewpoint,
            data_kind,
            record_spec: None,
            // record_spec: Record::write_spec(),
        };

        Ok(builder)
    }

    /// Create new [WriterBuilder](crate::writer::WriterBuilder) that
    /// stores header data.
    pub fn schema<Schema, Field>(mut self, spec: Schema) -> Result<Self>
    where
        Schema: IntoIterator<Item = Field>,
        Field: Into<FieldDef>,
    {
        let record_spec = spec
            .into_iter()
            .map(|field_def| field_def.into())
            .map(|FieldDef { name, kind, count }| (name, kind, count as usize))
            .collect::<Vec<(String, ValueKind, usize)>>();

        // Sanity check
        {
            let mut names = HashSet::new();

            for (name, _kind, count) in record_spec.iter() {
                if name.is_empty() {
                    bail!("Field name cannot be empty");
                }

                if *count == 0 {
                    bail!("The count of field {:?} cannot be zero", name);
                }

                if names.contains(name) {
                    bail!("The field name {:?} apprears more than once", name);
                }

                names.insert(name);
            }
        }

        self.record_spec = Some(record_spec);

        Ok(self)
    }

    /// Builds new [Writer](crate::writer::Writer) object from a writer.
    /// The writer must implement both [Write](std::io::Write) and [Write](std::io::Seek)
    /// traits.
    pub fn build_from_writer<Record: PcdSerialize, W: Write + Seek>(
        mut self,
        writer: W,
    ) -> Result<Writer<Record, W>> {
        if !Record::is_dynamic() {
            match self.record_spec {
                Some(_) => bail!("do not call schema() for static schema"),
                None => self.record_spec = Some(Record::write_spec()),
            }
        }

        let seq_writer = Writer::new(self, writer)?;
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
    record_spec: Vec<(String, ValueKind, usize)>,
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
    fn new(builder: WriterBuilder, mut writer: W) -> Result<Self> {
        let (points_arg_begin, points_arg_width) = builder.write_meta(&mut writer)?;

        let WriterBuilder {
            data_kind,
            record_spec,
            ..
        } = builder;

        let seq_writer = Self {
            data_kind,
            record_spec: record_spec.unwrap(),
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
