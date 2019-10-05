//! [SeqWriter](crate::seq_writer::SeqWriter) lets you write points sequentially to
//! PCD file or writer given by user. The written point type must implement
//! [PCDRecordWrite](crate::record::PCDRecordWrite) trait.
//! See [record](crate::record) moduel doc to implement your own point type.
//!
//! ```rust
//! use failure::Fallible;
//! use pcd_rs::{
//!     prelude::*,
//!     meta::DataKind,
//!     seq_writer::SeqWriterBuilder,
//!     PCDRecordWrite,
//! };
//! use std::path::Path;
//!
//! #[derive(PCDRecordWrite)]
//! pub struct Point {
//!     x: f32,
//!     y: f32,
//!     z: f32,
//! }
//!
//! fn main() -> Fallible<()> {
//!     let viewpoint = Default::default();
//!     let kind = DataKind::ASCII;
//!     let mut writer = SeqWriterBuilder::<Point, _>::new(300, 1, viewpoint, kind)?
//!         .create("test_files/dump.pcd")?;
//!
//!     let point = Point {
//!         x: 3.14159,
//!         y: 2.71828,
//!         z: -5.0,
//!     };
//!
//!     writer.push(&point)?;
//!
//!     Ok(())
//! }
//! ```

use crate::{
    meta::{DataKind, ValueKind, ViewPoint},
    record::{PCDRecordWrite, UntypedRecord},
    record::{SchemaKind, TypedSchema, UntypedSchema},
};
use failure::Fallible;
use std::{
    borrow::Borrow,
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufWriter, SeekFrom},
    marker::PhantomData,
    path::Path,
};

pub trait SeqWriterBuilderEx<Record, Kind>
where
    Kind: SchemaKind,
{
    fn from_writer<W: Write + Seek>(self, writer: W) -> Fallible<SeqWriter<W, Record, Kind>>;

    fn create<P: AsRef<Path>>(self, path: P) -> Fallible<SeqWriter<BufWriter<File>, Record, Kind>>;
}

/// A builder type that builds [SeqWriter](crate::seq_writer::SeqWriter).
pub struct SeqWriterBuilder<Record, Kind>
where
    Kind: SchemaKind,
{
    width: u64,
    height: u64,
    viewpoint: ViewPoint,
    data_kind: DataKind,
    record_spec: Vec<(String, ValueKind, usize)>,
    _phantom: PhantomData<(Record, Kind)>,
}

impl<Record, Kind> SeqWriterBuilderEx<Record, Kind> for SeqWriterBuilder<Record, Kind>
where
    Kind: SchemaKind,
{
    /// Builds new [SeqWriter](crate::seq_writer::SeqWriter) object from a writer.
    /// The writer must implement both [Write](std::io::Write) and [Write](std::io::Seek)
    /// traits.
    fn from_writer<W: Write + Seek>(self, writer: W) -> Fallible<SeqWriter<W, Record, Kind>> {
        let seq_writer = SeqWriter::new(self, writer)?;
        Ok(seq_writer)
    }

    /// Builds new [SeqWriter](crate::seq_writer::SeqWriter) by creating a new file.
    fn create<P: AsRef<Path>>(self, path: P) -> Fallible<SeqWriter<BufWriter<File>, Record, Kind>> {
        let writer = BufWriter::new(File::create(path.as_ref())?);
        let seq_writer = self.from_writer(writer)?;
        Ok(seq_writer)
    }
}

impl<Record> SeqWriterBuilder<Record, TypedSchema>
where
    Record: PCDRecordWrite,
{
    /// Create new [SeqWriterBuilder](crate::seq_writer::SeqWriterBuilder) that
    /// stores header data.
    pub fn new(
        width: u64,
        height: u64,
        viewpoint: ViewPoint,
        data_kind: DataKind,
    ) -> Fallible<Self> {
        let builder = Self {
            width,
            height,
            viewpoint,
            data_kind,
            record_spec: Record::write_spec(),
            _phantom: PhantomData,
        };

        Ok(builder)
    }
}

impl SeqWriterBuilder<UntypedRecord, UntypedSchema> {
    /// Create new [SeqWriterBuilder](crate::seq_writer::SeqWriterBuilder) that
    /// stores header data.
    pub fn new<Name, Spec>(
        width: u64,
        height: u64,
        viewpoint: ViewPoint,
        data_kind: DataKind,
        spec: Spec,
    ) -> Fallible<Self>
    where
        Name: Borrow<str>,
        Spec: Borrow<[(Name, ValueKind, usize)]>,
    {
        let record_spec = spec
            .borrow()
            .into_iter()
            .map(|(name, kind, size)| (name.borrow().to_owned(), *kind, *size))
            .collect::<Vec<(String, ValueKind, usize)>>();

        // Sanity check
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

        let builder = Self {
            width,
            height,
            viewpoint,
            data_kind,
            record_spec,
            _phantom: PhantomData,
        };

        Ok(builder)
    }
}

pub trait SeqWriterEx<Writer, Record, Kind>
where
    Writer: Write + Seek,
    Kind: SchemaKind,
    Self: Sized,
{
    fn push(&mut self, record: &Record) -> Fallible<()>;
}

/// A Writer type that write points to PCD data.
pub struct SeqWriter<Writer, Record, Kind>
where
    Writer: Write + Seek,
    Kind: SchemaKind,
{
    writer: Writer,
    builder: SeqWriterBuilder<Record, Kind>,
    num_records: usize,
    points_arg_begin: u64,
    points_arg_width: usize,
}

impl<Writer, Record, Kind> SeqWriter<Writer, Record, Kind>
where
    Writer: Write + Seek,
    Kind: SchemaKind,
{
    fn new(builder: SeqWriterBuilder<Record, Kind>, mut writer: Writer) -> Fallible<Self> {
        let (points_arg_begin, points_arg_width) = Self::write_meta(&builder, &mut writer)?;
        let seq_writer = Self {
            builder,
            writer,
            num_records: 0,
            points_arg_begin,
            points_arg_width,
        };
        Ok(seq_writer)
    }

    fn write_meta(
        builder: &SeqWriterBuilder<Record, Kind>,
        writer: &mut Writer,
    ) -> Fallible<(u64, usize)> {
        let fields_args = builder
            .record_spec
            .iter()
            .map(|(name, _, _)| name.to_owned())
            .collect::<Vec<_>>();

        let size_args = builder
            .record_spec
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

        let type_args = builder
            .record_spec
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

        let count_args = builder
            .record_spec
            .iter()
            .map(|(_, _, count)| count.to_string())
            .collect::<Vec<_>>();

        let viewpoint_args = {
            let viewpoint = &builder.viewpoint;
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
        writeln!(writer, "WIDTH {}", builder.width)?;
        writeln!(writer, "HEIGHT {}", builder.height)?;
        writeln!(writer, "VIEWPOINT {}", viewpoint_args.join(" "))?;

        write!(writer, "POINTS ")?;
        let points_arg_begin = writer.seek(SeekFrom::Current(0))?;
        writeln!(writer, "{:width$}", " ", width = points_arg_width)?;

        match builder.data_kind {
            DataKind::Binary => writeln!(writer, "DATA binary")?,
            DataKind::ASCII => writeln!(writer, "DATA ascii")?,
        }

        Ok((points_arg_begin, points_arg_width))
    }

    fn increase_record_count(&mut self) -> Fallible<()> {
        self.num_records += 1;
        let eof_pos = self.writer.seek(SeekFrom::Current(0))?;

        self.writer.seek(SeekFrom::Start(self.points_arg_begin))?;
        write!(
            self.writer,
            "{:<width$}",
            self.num_records,
            width = self.points_arg_width
        )?;
        self.writer.seek(SeekFrom::Start(eof_pos))?;

        Ok(())
    }
}

impl<Writer, Record> SeqWriterEx<Writer, Record, TypedSchema>
    for SeqWriter<Writer, Record, TypedSchema>
where
    Writer: Write + Seek,
    Record: PCDRecordWrite,
{
    /// Writes a new point to PCD data.
    fn push(&mut self, record: &Record) -> Fallible<()> {
        match self.builder.data_kind {
            DataKind::Binary => record.write_chunk(&mut self.writer)?,
            DataKind::ASCII => record.write_line(&mut self.writer)?,
        }

        self.increase_record_count()?;
        Ok(())
    }
}

impl<Writer> SeqWriterEx<Writer, UntypedRecord, UntypedSchema>
    for SeqWriter<Writer, UntypedRecord, UntypedSchema>
where
    Writer: Write + Seek,
{
    /// Writes a new point to PCD data.
    fn push(&mut self, record: &UntypedRecord) -> Fallible<()> {
        match self.builder.data_kind {
            DataKind::Binary => record.write_chunk(&mut self.writer, &self.builder.record_spec)?,
            DataKind::ASCII => record.write_line(&mut self.writer, &self.builder.record_spec)?,
        }

        self.increase_record_count()?;
        Ok(())
    }
}
