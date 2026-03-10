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
        version: None,
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
    lzf,
    metas::{DataKind, FieldDef, Schema, ValueKind, ViewPoint},
    record::{DynRecord, PcdSerialize},
    Error, Result,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufWriter, Cursor, SeekFrom},
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
    /// PCD version to write (defaults to "0.7")
    pub version: Option<String>,
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

        let version = self.version.unwrap_or_else(|| "0.7".to_string());

        let seq_writer = Writer::new(
            self.width,
            self.height,
            self.data_kind,
            self.viewpoint,
            record_spec,
            writer,
            version,
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
    compressed_buffer: Option<Vec<u8>>,
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
        version: String,
    ) -> Result<Self, Error> {
        macro_rules! ensure {
            ($cond:expr, $desc:expr) => {
                if !$cond {
                    return Err(Error::new_invalid_writer_configuration_error($desc));
                }
            };
        }

        // Validate version
        match version.as_str() {
            "0.5" | ".5" | "0.6" | ".6" | "0.7" | ".7" => {}
            _ => {
                return Err(Error::new_invalid_writer_configuration_error(
                    "Unsupported PCD version. Supported versions: 0.5, 0.6, 0.7",
                ))
            }
        }

        // Check version-specific constraints
        if version == "0.5" || version == ".5" || version == "0.6" || version == ".6" {
            // Legacy versions don't support binary_compressed
            if matches!(data_kind, DataKind::BinaryCompressed) {
                return Err(Error::new_invalid_writer_configuration_error(
                    "binary_compressed format is only supported in PCD v0.7",
                ));
            }
        }

        // Run sanity check on the schema.
        {
            for FieldDef { name, count, .. } in &record_spec {
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

            let points_arg_width = (usize::MAX as f64).log10().floor() as usize + 1;

            // Normalize version format for header
            let header_version = match version.as_str() {
                "0.5" | ".5" => ".5",
                "0.6" | ".6" => ".6",
                "0.7" | ".7" => ".7",
                _ => ".7",
            };

            let header_comment = match header_version {
                ".5" => "# .PCD v.5 - Point Cloud Data file format",
                ".6" => "# .PCD v.6 - Point Cloud Data file format",
                _ => "# .PCD v.7 - Point Cloud Data file format",
            };

            writeln!(writer, "{}", header_comment)?;
            writeln!(writer, "VERSION {}", header_version)?;
            writeln!(writer, "FIELDS {}", fields_args.join(" "))?;
            writeln!(writer, "SIZE {}", size_args.join(" "))?;
            writeln!(writer, "TYPE {}", type_args.join(" "))?;
            writeln!(writer, "COUNT {}", count_args.join(" "))?;
            writeln!(writer, "WIDTH {}", width)?;
            writeln!(writer, "HEIGHT {}", height)?;

            // Only write VIEWPOINT for v0.7
            if header_version == ".7" {
                writeln!(writer, "VIEWPOINT {}", viewpoint_args.join(" "))?;
            }

            write!(writer, "POINTS ")?;
            let points_arg_begin = writer.stream_position()?;
            writeln!(writer, "{:width$}", " ", width = points_arg_width)?;

            match data_kind {
                DataKind::Binary => writeln!(writer, "DATA binary")?,
                DataKind::Ascii => writeln!(writer, "DATA ascii")?,
                DataKind::BinaryCompressed => writeln!(writer, "DATA binary_compressed")?,
            }

            (points_arg_begin, points_arg_width)
        };

        let compressed_buffer = if data_kind == DataKind::BinaryCompressed {
            Some(Vec::new())
        } else {
            None
        };

        let seq_writer = Self {
            data_kind,
            record_spec,
            writer,
            num_records: 0,
            points_arg_begin,
            points_arg_width,
            finished: false,
            compressed_buffer,
            _phantom: PhantomData,
        };
        Ok(seq_writer)
    }

    /// Finish the writer.
    ///
    /// The method consumes the writer must be called once when finished.
    /// Otherwise it will panic when it drops.
    pub fn finish(mut self) -> Result<()> {
        // Write compressed data if using compression
        if self.data_kind == DataKind::BinaryCompressed {
            if let Some(ref row_major_data) = self.compressed_buffer {
                if row_major_data.is_empty() {
                    // For empty data, write zeros for sizes
                    self.writer.write_u32::<LittleEndian>(0)?;
                    self.writer.write_u32::<LittleEndian>(0)?;
                } else {
                    // Transpose from row-major to column-major (SoA) layout
                    let num_points = self.num_records;
                    let field_byte_sizes: Vec<usize> = self
                        .record_spec
                        .iter()
                        .map(|f| f.kind.byte_size() * f.count as usize)
                        .collect();
                    let record_size: usize = field_byte_sizes.iter().sum();

                    // column_start[f] is the byte offset where field f's column begins
                    let mut column_start = Vec::with_capacity(field_byte_sizes.len());
                    let mut offset = 0usize;
                    for &fbs in &field_byte_sizes {
                        column_start.push(offset);
                        offset += fbs * num_points;
                    }

                    // field_offset_in_record[f] is the byte offset of field f within a single record
                    let mut field_offset_in_record = Vec::with_capacity(field_byte_sizes.len());
                    let mut rec_offset = 0usize;
                    for &fbs in &field_byte_sizes {
                        field_offset_in_record.push(rec_offset);
                        rec_offset += fbs;
                    }

                    let mut col_major = vec![0u8; row_major_data.len()];
                    for i in 0..num_points {
                        for (f, &fbs) in field_byte_sizes.iter().enumerate() {
                            let src = i * record_size + field_offset_in_record[f];
                            let dst = column_start[f] + i * fbs;
                            col_major[dst..dst + fbs]
                                .copy_from_slice(&row_major_data[src..src + fbs]);
                        }
                    }

                    // Compress the column-major data
                    let compressed_data = lzf::compress(&col_major)?;

                    // Write compressed size and uncompressed size
                    self.writer
                        .write_u32::<LittleEndian>(compressed_data.len() as u32)?;
                    self.writer
                        .write_u32::<LittleEndian>(col_major.len() as u32)?;

                    // Write compressed data
                    self.writer.write_all(&compressed_data)?;
                }
            }
        }

        // Update the points count in the header
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
            DataKind::BinaryCompressed => {
                // Buffer the binary data for compression
                if let Some(ref mut buffer) = self.compressed_buffer {
                    // Create a temporary buffer with cursor to write the record
                    let mut temp_buffer = Vec::new();
                    let mut cursor = Cursor::new(&mut temp_buffer);
                    record.write_chunk(&mut cursor, &self.record_spec)?;
                    buffer.extend_from_slice(&temp_buffer);
                } else {
                    return Err(Error::ParseError {
                        line: 0,
                        desc: "Compressed buffer not initialized".into(),
                    });
                }
            }
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
