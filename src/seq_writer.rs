use crate::{DataKind, PCDRecordWrite, ValueKind, ViewPoint};
use failure::Fallible;
use std::{
    fs::File,
    io::{prelude::*, BufWriter, SeekFrom},
    marker::PhantomData,
    path::Path,
};

pub struct SeqWriterBuilder<T: PCDRecordWrite> {
    width: u64,
    height: u64,
    viewpoint: ViewPoint,
    data_kind: DataKind,
    record_spec: Vec<(String, ValueKind, usize)>,
    _phantom: PhantomData<T>,
}

impl<T: PCDRecordWrite> SeqWriterBuilder<T> {
    pub fn new(
        width: u64,
        height: u64,
        viewpoint: ViewPoint,
        data_kind: DataKind,
    ) -> Fallible<SeqWriterBuilder<T>> {
        let record_spec = T::write_spec();

        let builder = SeqWriterBuilder {
            width,
            height,
            viewpoint,
            data_kind,
            record_spec,
            _phantom: PhantomData,
        };

        Ok(builder)
    }

    pub fn from_writer<R: Write + Seek>(self, writer: R) -> Fallible<SeqWriter<R, T>> {
        let seq_writer = SeqWriter::new(self, writer)?;
        Ok(seq_writer)
    }

    pub fn create<P: AsRef<Path>>(self, path: P) -> Fallible<SeqWriter<BufWriter<File>, T>> {
        let writer = BufWriter::new(File::create(path.as_ref())?);
        let seq_writer = self.from_writer(writer)?;
        Ok(seq_writer)
    }
}

pub struct SeqWriter<R: Write + Seek, T: PCDRecordWrite> {
    writer: R,
    builder: SeqWriterBuilder<T>,
    num_records: usize,
    points_arg_begin: u64,
    points_arg_width: usize,
}

impl<R: Write + Seek, T: PCDRecordWrite> SeqWriter<R, T> {
    pub fn new(builder: SeqWriterBuilder<T>, mut writer: R) -> Fallible<SeqWriter<R, T>> {
        let (points_arg_begin, points_arg_width) = Self::write_meta(&builder, &mut writer)?;
        dbg!(points_arg_begin, points_arg_width);
        let seq_writer = SeqWriter {
            builder,
            writer,
            num_records: 0,
            points_arg_begin,
            points_arg_width,
        };
        Ok(seq_writer)
    }

    pub fn write_meta(builder: &SeqWriterBuilder<T>, writer: &mut R) -> Fallible<(u64, usize)> {
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

    pub fn push(&mut self, record: &T) -> Fallible<()> {
        match self.builder.data_kind {
            DataKind::Binary => record.write_chunk(&mut self.writer)?,
            DataKind::ASCII => record.write_line(&mut self.writer)?,
        }
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
