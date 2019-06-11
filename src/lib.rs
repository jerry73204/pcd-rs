extern crate byteorder;
// extern crate memmap;
#[macro_use] extern crate log;

use std::io::prelude::*;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::fs::File;
use std::fmt;
use std::error::Error;
use std::collections::HashSet;
use byteorder::{LittleEndian, ReadBytesExt};

type PCDResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Used to configure LCD file readers.
pub struct ReaderOptions {}

/// Sequential point loader for PCD files.
pub struct SeqReader<T> {
    pub meta: PCDMeta,
    line_count: usize,
    point_count: u64,
    reader: BufReader<T>,
}

/// The struct keep meta data of PCD file.
#[derive(Debug)]
pub struct PCDMeta {
    pub version: String,
    pub width: u64,
    pub height: u64,
    pub viewpoint: Vec<u64>,
    pub num_points: u64,
    pub data: DataKind,
    pub field_defs: Vec<FieldDef>,
}

/// The error is raised when a PCD file is not understood by parser.
#[derive(Clone)]
pub struct ParseError {
    desc: String,
}

/// The enum specifies one of signed, unsigned integers, and floating point number type to the field.
#[derive(Debug)]
enum TypeKind {
    I, U, F
}

/// The enum indicates whether the point cloud data is encoded in ASCII or binary.
#[derive(Debug)]
pub enum DataKind {
    ASCII,
    Binary,
}

/// The enum specifies the exact type for each PCD field.
#[derive(Debug)]
pub enum ValueKind {
    U8, U16, U32,
    I8, I16, I32,
    F32, F64,
}

/// The enum holds the exact value of each PCD field.
#[derive(Debug)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    I8(i8),
    I16(i16),
    I32(i32),
    F32(f32),
    F64(f64),
    U8V(Vec<u8>),
    U16V(Vec<u16>),
    U32V(Vec<u32>),
    I8V(Vec<i8>),
    I16V(Vec<i16>),
    I32V(Vec<i32>),
    F32V(Vec<f32>),
    F64V(Vec<f64>),
}

/// Define the properties of a PCD field.
#[derive(Debug)]
pub struct FieldDef {
    name: String,
    kind: ValueKind,
    count: u64,
}

impl ReaderOptions {
    /// Load PCD file from path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> PCDResult<SeqReader<File>> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    /// Parse PCD file buffer.
    pub fn from_buffer(buf: &[u8]) -> PCDResult<SeqReader<Cursor<&[u8]>>> {
        let reader = Cursor::new(buf);
        Self::from_reader(reader)
    }

    /// Load PCD data from reader.
    pub fn from_reader<R: Read>(reader: R) -> PCDResult<SeqReader<R>> {
        let mut buf_reader = BufReader::new(reader);
        let mut line_count = 0;
        let meta = Self::load_meta(&mut buf_reader, &mut line_count)?;
        let pcd = SeqReader {
            meta,
            line_count,
            point_count: 0,
            reader: buf_reader,
        };

        Ok(pcd)
    }

    fn load_meta<R: BufRead>(reader: &mut R, line_count: &mut usize) -> PCDResult<PCDMeta> {
        let mut get_meta_line = |expect_entry: &str| -> PCDResult<_> {
            loop {
                let mut line = String::new();
                let read_size = reader.read_line(&mut line)?;
                *line_count += 1;

                if read_size == 0 {
                    let desc = format!("Unexpected end of file");
                    return Err(Box::new(
                        ParseError::new(&desc)
                    ));
                }

                let line_stripped = match line.split('#').nth(0) {
                    Some("") => continue,
                    Some(remaining) => remaining,
                    None => continue,
                };

                let tokens: Vec<String> = line_stripped.split_ascii_whitespace()
                    .map(|s| s.to_owned())
                    .collect();

                if tokens.is_empty() {
                    let desc = format!("Cannot parse empty line at line {}", *line_count + 1);
                    return Err(Box::new(
                        ParseError::new(&desc)
                    ));
                }

                if tokens[0] != expect_entry {
                    let desc = format!("Expect {:?} entry, found {:?} at line {}", expect_entry, tokens[0], *line_count + 1);
                    return Err(Box::new(
                        ParseError::new(&desc)
                    ));
                }

                return Ok(tokens)
            }
        };


        let meta_version = {
            let tokens = get_meta_line("VERSION")?;
            if tokens.len() == 2 {
                match tokens[1].as_str() {
                    "0.7" => String::from("0.7"),
                    ".7" => String::from("0.7"),
                    _ => {
                        return Err(Box::new(
                            ParseError::new(&format!("Unsupported version {:?}. Supported versions are: 0.7", tokens[1]))
                        ));
                    }
                }
            }
            else {
                return Err(Box::new(
                    ParseError::new("VERSION line is not understood")
                ));
            }
        };

        let meta_fields = {
            let tokens = get_meta_line("FIELDS")?;
            if tokens.len() == 1 {
                return Err(Box::new(
                    ParseError::new("FIELDS line is not understood")
                ));
            }

            let mut name_set = HashSet::new();
            let mut field_names: Vec<String> = vec![];

            for tk in tokens[1..].into_iter() {
                let field = tk;
                if name_set.contains(field) {
                    return Err(Box::new(
                        ParseError::new(&format!("field name {:?} is specified more than once", field))
                    ));
                }

                name_set.insert(field);
                field_names.push(field.to_owned());
            }

            field_names
        };

        let meta_size = {
            let tokens = get_meta_line("SIZE")?;
            if tokens.len() == 1 {
                return Err(Box::new(
                    ParseError::new("SIZE line is not understood")
                ));
            }

            let mut sizes = vec![];
            for tk in tokens[1..].into_iter() {
                let size: u64 = tk.parse()?;
                sizes.push(size);
            }

            sizes
        };

        let meta_type = {
            let tokens = get_meta_line("TYPE")?;

            if tokens.len() == 1 {
                return Err(Box::new(
                    ParseError::new("TYPE line is not understood")
                ));
            }

            let mut types = vec![];
            for type_char in tokens[1..].into_iter() {
                let type_ = match type_char.as_str() {
                    "I" => TypeKind::I,
                    "U" => TypeKind::U,
                    "F" => TypeKind::F,
                    _ => {
                        return Err(Box::new(
                            ParseError::new(&format!("Invalid type character {:?} in TYPE line", type_char))
                        ));
                    }
                };
                types.push(type_);
            }

            types
        };

        let meta_count = {
            let tokens = get_meta_line("COUNT")?;

            if tokens.len() == 1 {
                return Err(Box::new(
                    ParseError::new("COUNT line is not understood")
                ));
            }

            let mut counts = vec![];
            for tk in tokens[1..].into_iter() {
                let count: u64 = tk.parse()?;
                counts.push(count);
            }

            counts
        };

        let meta_width = {
            let tokens = get_meta_line("WIDTH")?;

            if tokens.len() != 2 {
                return Err(Box::new(
                    ParseError::new("WIDTH line is not understood")
                ));
            }

            let width: u64 = tokens[1].parse()?;
            width
        };

        let meta_height = {
            let tokens = get_meta_line("HEIGHT")?;
            if tokens.len() != 2 {
                return Err(Box::new(
                    ParseError::new("HEIGHT line is not understood")
                ));
            }

            let height: u64 = tokens[1].parse()?;
            height
        };

        let meta_viewpoint = {
            let tokens = get_meta_line("VIEWPOINT")?;

            if tokens.len() == 1 {
                return Err(Box::new(
                    ParseError::new("VIEWPOINT line is not understood")
                ));
            }

            let mut params = vec![];
            for tk in tokens[1..].into_iter() {
                let param: u64 = tk.parse()?;
                params.push(param);
            }

            params
        };

        let meta_points = {
            let tokens = get_meta_line("POINTS")?;

            if tokens.len() != 2 {
                return Err(Box::new(
                    ParseError::new("POINTS line is not understood")
                ));
            }

            let count: u64 = tokens[1].parse()?;
            count
        };

        let meta_data = {
            let tokens = get_meta_line("DATA")?;

            if tokens.len() != 2 {
                return Err(Box::new(
                    ParseError::new("DATA line is not understood")
                ));
            }

            match tokens[1].as_str() {
                "ascii" => DataKind::ASCII,
                "binary" => DataKind::Binary,
                _ => {
                    return Err(Box::new(
                        ParseError::new("DATA line is not understood")
                    ));
                }
            }
        };

        // Check integrity
        if meta_size.len() != meta_fields.len() {
            return Err(Box::new(
                ParseError::new("SIZE entry conflicts with FIELD entry")
            ));
        }

        if meta_type.len() != meta_fields.len() {
            return Err(Box::new(
                ParseError::new("TYPE entry conflicts with FIELD entry")
            ));
        }

        if meta_count.len() != meta_fields.len() {
            return Err(Box::new(
                ParseError::new("COUNT entry conflicts with FIELD entry")
            ));
        }

        // Organize field type
        let field_defs = {
            let mut field_defs = vec![];
            for (((name, type_), size), count) in meta_fields.iter().zip(
                meta_type.iter()
            ).zip(
                meta_size.iter()
            ).zip(
                meta_count.iter()
            )
            {
                let kind = match (type_, size) {
                    (TypeKind::U, 1) => ValueKind::U8,
                    (TypeKind::U, 2) => ValueKind::U16,
                    (TypeKind::U, 4) => ValueKind::U32,
                    (TypeKind::I, 1) => ValueKind::I8,
                    (TypeKind::I, 2) => ValueKind::I16,
                    (TypeKind::I, 4) => ValueKind::I32,
                    (TypeKind::F, 4) => ValueKind::F32,
                    (TypeKind::F, 8) => ValueKind::F64,
                    _ => {
                        let desc = format!("Field type {:?} with size {} is not supported", type_, size);
                        return Err(Box::new(
                            ParseError::new(&desc)
                        ));
                    }
                };

                let meta = FieldDef {
                    name: name.to_owned(),
                    kind,
                    count: *count,
                };

                field_defs.push(meta);
            }

            field_defs
        };

        let meta = PCDMeta {
            version: meta_version,
            field_defs,
            width: meta_width,
            height: meta_height,
            viewpoint: meta_viewpoint,
            num_points: meta_points,
            data: meta_data,
        };

        Ok(meta)
    }
}

impl<T: Read> SeqReader<T> {
    pub fn read_point(&mut self) -> PCDResult<Option<Vec<Value>>> {
        if self.point_count == self.meta.num_points {
            return Ok(None);
        }

        let row = match self.meta.data {
            DataKind::ASCII => {
                scan_line(
                    &mut self.reader,
                    &self.meta.field_defs,
                    &mut self.line_count,
                )?

            }
            DataKind::Binary => {
                scan_chunk(
                    &mut self.reader,
                    &self.meta.field_defs,
                )?
            }
        };

        self.point_count += 1;
        Ok(Some(row))
    }

    pub fn read_all(&mut self) -> PCDResult<Vec<Vec<Value>>> {
        match self.meta.data {
            DataKind::ASCII => {
                self.read_all_ascii()
            }
            DataKind::Binary => {
                self.read_all_binary()
            }
        }
    }


    fn read_all_ascii(&mut self) -> PCDResult<Vec<Vec<Value>>> {
        let mut points = vec![];

        while self.point_count < self.meta.num_points {
            let row = scan_line(
                &mut self.reader,
                &self.meta.field_defs,
                &mut self.line_count,
            )?;
            self.point_count += 1;
            points.push(row);
        }

        Ok(points)
    }

    fn read_all_binary(&mut self) -> PCDResult<Vec<Vec<Value>>> {
        let mut points = vec![];

        while self.point_count < self.meta.num_points {
            let row = scan_chunk(
                &mut self.reader,
                &self.meta.field_defs,
            )?;
            self.point_count += 1;
            points.push(row);
        }

        Ok(points)
    }
}

impl ParseError {
    fn new(desc: &str) -> ParseError {
        ParseError {
            desc: desc.to_owned()
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.desc
    }
}

fn scan_line<R: BufRead>(
    reader: &mut R,
    field_defs: &[FieldDef],
    line_count: &mut usize,
) -> PCDResult<Vec<Value>>
{
    let make_error = |line_num| {
        let desc = format!("Invalid data line at line {}", line_num);
        ParseError::new(&desc)
    };

    // Tokenize line
    let mut line = String::new();
    let read_size = reader.read_line(&mut line)?;
    *line_count += 1;

    if read_size == 0 {
        let desc = format!("Unexpected end of file at line {}", line_count);
        return Err(Box::new(ParseError::new(&desc)));
    }

    let mut tokens = line.split_ascii_whitespace();

    let mut row = vec![];

    for meta in field_defs {
        let value = match (&meta.kind, meta.count) {
            (ValueKind::U8, 1) => {
                let val: u8 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::U8(val)
            }
            (ValueKind::U16, 1) => {
                let val: u16 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::U16(val)
            }
            (ValueKind::U32, 1) => {
                let val: u32 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::U32(val)
            }
            (ValueKind::I8, 1) => {
                let val: i8 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::I8(val)
            }
            (ValueKind::I16, 1) => {
                let val: i16 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::I16(val)
            }
            (ValueKind::I32, 1) => {
                let val: i32 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::I32(val)
            }
            (ValueKind::F32, 1) => {
                let val: f32 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::F32(val)
            }
            (ValueKind::F64, 1) => {
                let val: f64 = tokens.next()
                    .ok_or(make_error(*line_count))?
                    .parse()?;
                Value::F64(val)
            }
            (ValueKind::U8, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: u8 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::U8V(values)
            }
            (ValueKind::U16, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: u16 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::U16V(values)
            }
            (ValueKind::U32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: u32 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::U32V(values)
            }
            (ValueKind::I8, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: i8 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::I8V(values)
            }
            (ValueKind::I16, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: i16 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::I16V(values)
            }
            (ValueKind::I32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: i32 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::I32V(values)
            }
            (ValueKind::F32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: f32 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::F32V(values)
            }
            (ValueKind::F64, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: f64 = tokens.next()
                        .ok_or(make_error(*line_count))?
                        .parse()?;
                    values.push(val);
                }
                Value::F64V(values)
            }
        };

        row.push(value);
    }

    Ok(row)
}



fn scan_chunk<R: BufRead>(
    reader: &mut R,
    field_defs: &[FieldDef],
) -> PCDResult<Vec<Value>>
{
    let mut row = vec![];

    for meta in field_defs {
        let value = match (&meta.kind, meta.count) {
            (ValueKind::U8, 1) => {
                let val = reader.read_u8()?;
                Value::U8(val)
            }
            (ValueKind::U16, 1) => {
                let val = reader.read_u16::<LittleEndian>()?;
                Value::U16(val)
            }
            (ValueKind::U32, 1) => {
                let val = reader.read_u32::<LittleEndian>()?;
                Value::U32(val)
            }
            (ValueKind::I8, 1) => {
                let val = reader.read_i8()?;
                Value::I8(val)
            }
            (ValueKind::I16, 1) => {
                let val = reader.read_i16::<LittleEndian>()?;
                Value::I16(val)
            }
            (ValueKind::I32, 1) => {
                let val = reader.read_i32::<LittleEndian>()?;
                Value::I32(val)
            }
            (ValueKind::F32, 1) => {
                let val = reader.read_f32::<LittleEndian>()?;
                Value::F32(val)
            }
            (ValueKind::F64, 1) => {
                let val = reader.read_f64::<LittleEndian>()?;
                Value::F64(val)
            }
            (ValueKind::U8, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_u8()?;
                    values.push(val);
                }
                Value::U8V(values)
            }
            (ValueKind::U16, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_u16::<LittleEndian>()?;
                    values.push(val);
                }
                Value::U16V(values)
            }
            (ValueKind::U32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_u32::<LittleEndian>()?;
                    values.push(val);
                }
                Value::U32V(values)
            }
            (ValueKind::I8, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_i8()?;
                    values.push(val);
                }
                Value::I8V(values)
            }
            (ValueKind::I16, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_i16::<LittleEndian>()?;
                    values.push(val);
                }
                Value::I16V(values)
            }
            (ValueKind::I32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_i32::<LittleEndian>()?;
                    values.push(val);
                }
                Value::I32V(values)
            }
            (ValueKind::F32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_f32::<LittleEndian>()?;
                    values.push(val);
                }
                Value::F32V(values)
            }
            (ValueKind::F64, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val = reader.read_f64::<LittleEndian>()?;
                    values.push(val);
                }
                Value::F64V(values)
            }
        };

        row.push(value);
    }

    Ok(row)
}
