use crate::{error::ParseError, DataKind, FieldDef, PCDMeta, TypeKind, Value, ValueKind};
use failure::Fallible;
use std::{
    collections::HashSet,
    fs::File,
    io::{prelude::*, BufReader, Cursor},
    path::Path,
};

/// Used to configure LCD file readers.
pub struct SeqReaderOptions;

impl SeqReaderOptions {
    /// Load PCD file from path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Fallible<SeqReader<File>> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    /// Parse PCD file buffer.
    pub fn from_buffer(buf: &[u8]) -> Fallible<SeqReader<Cursor<&[u8]>>> {
        let reader = Cursor::new(buf);
        Self::from_reader(reader)
    }

    /// Load PCD data from reader.
    pub fn from_reader<R: Read>(reader: R) -> Fallible<SeqReader<R>> {
        let mut buf_reader = BufReader::new(reader);
        let mut line_count = 0;
        let meta = Self::load_meta(&mut buf_reader, &mut line_count)?;
        let pcd = SeqReader {
            meta,
            line_count,
            point_count: 0,
            finished: false,
            reader: buf_reader,
        };

        Ok(pcd)
    }

    fn load_meta<R: BufRead>(reader: &mut R, line_count: &mut usize) -> Fallible<PCDMeta> {
        let mut get_meta_line = |expect_entry: &str| -> Fallible<_> {
            loop {
                let mut line = String::new();
                let read_size = reader.read_line(&mut line)?;
                *line_count += 1;

                if read_size == 0 {
                    return Err(ParseError::new("Unexpected end of file").into());
                }

                let line_stripped = match line.split('#').nth(0) {
                    Some("") => continue,
                    Some(remaining) => remaining,
                    None => continue,
                };

                let tokens: Vec<String> = line_stripped
                    .split_ascii_whitespace()
                    .map(|s| s.to_owned())
                    .collect();

                if tokens.is_empty() {
                    let desc = format!("Cannot parse empty line at line {}", *line_count + 1);
                    return Err(ParseError::new(&desc).into());
                }

                if tokens[0] != expect_entry {
                    let desc = format!(
                        "Expect {:?} entry, found {:?} at line {}",
                        expect_entry,
                        tokens[0],
                        *line_count + 1
                    );
                    return Err(ParseError::new(&desc).into());
                }

                return Ok(tokens);
            }
        };

        let meta_version = {
            let tokens = get_meta_line("VERSION")?;
            if tokens.len() == 2 {
                match tokens[1].as_str() {
                    "0.7" => String::from("0.7"),
                    ".7" => String::from("0.7"),
                    _ => {
                        let desc = format!(
                            "Unsupported version {:?}. Supported versions are: 0.7",
                            tokens[1]
                        );
                        return Err(ParseError::new(&desc).into());
                    }
                }
            } else {
                return Err(ParseError::new("VERSION line is not understood").into());
            }
        };

        let meta_fields = {
            let tokens = get_meta_line("FIELDS")?;
            if tokens.len() == 1 {
                return Err(ParseError::new("FIELDS line is not understood").into());
            }

            let mut name_set = HashSet::new();
            let mut field_names: Vec<String> = vec![];

            for tk in tokens[1..].into_iter() {
                let field = tk;
                if name_set.contains(field) {
                    let desc = format!("field name {:?} is specified more than once", field);
                    return Err(ParseError::new(&desc).into());
                }

                name_set.insert(field);
                field_names.push(field.to_owned());
            }

            field_names
        };

        let meta_size = {
            let tokens = get_meta_line("SIZE")?;
            if tokens.len() == 1 {
                return Err(ParseError::new("SIZE line is not understood").into());
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
                return Err(ParseError::new("TYPE line is not understood").into());
            }

            let mut types = vec![];
            for type_char in tokens[1..].into_iter() {
                let type_ = match type_char.as_str() {
                    "I" => TypeKind::I,
                    "U" => TypeKind::U,
                    "F" => TypeKind::F,
                    _ => {
                        let desc = format!("Invalid type character {:?} in TYPE line", type_char);
                        return Err(ParseError::new(&desc).into());
                    }
                };
                types.push(type_);
            }

            types
        };

        let meta_count = {
            let tokens = get_meta_line("COUNT")?;

            if tokens.len() == 1 {
                return Err(ParseError::new("COUNT line is not understood").into());
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
                return Err(ParseError::new("WIDTH line is not understood").into());
            }

            let width: u64 = tokens[1].parse()?;
            width
        };

        let meta_height = {
            let tokens = get_meta_line("HEIGHT")?;
            if tokens.len() != 2 {
                return Err(ParseError::new("HEIGHT line is not understood").into());
            }

            let height: u64 = tokens[1].parse()?;
            height
        };

        let meta_viewpoint = {
            let tokens = get_meta_line("VIEWPOINT")?;

            if tokens.len() == 1 {
                return Err(ParseError::new("VIEWPOINT line is not understood").into());
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
                return Err(ParseError::new("POINTS line is not understood").into());
            }

            let count: u64 = tokens[1].parse()?;
            count
        };

        let meta_data = {
            let tokens = get_meta_line("DATA")?;

            if tokens.len() != 2 {
                return Err(ParseError::new("DATA line is not understood").into());
            }

            match tokens[1].as_str() {
                "ascii" => DataKind::ASCII,
                "binary" => DataKind::Binary,
                _ => {
                    return Err(ParseError::new("DATA line is not understood").into());
                }
            }
        };

        // Check integrity
        if meta_size.len() != meta_fields.len() {
            return Err(ParseError::new("SIZE entry conflicts with FIELD entry").into());
        }

        if meta_type.len() != meta_fields.len() {
            return Err(ParseError::new("TYPE entry conflicts with FIELD entry").into());
        }

        if meta_count.len() != meta_fields.len() {
            return Err(ParseError::new("COUNT entry conflicts with FIELD entry").into());
        }

        // Organize field type
        let field_defs = {
            let mut field_defs = vec![];
            for (((name, type_), size), count) in meta_fields
                .iter()
                .zip(meta_type.iter())
                .zip(meta_size.iter())
                .zip(meta_count.iter())
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
                        let desc =
                            format!("Field type {:?} with size {} is not supported", type_, size);
                        return Err(ParseError::new(&desc).into());
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

/// Sequential point loader for PCD files.
pub struct SeqReader<T> {
    meta: PCDMeta,
    line_count: usize,
    point_count: u64,
    finished: bool,
    reader: BufReader<T>,
}

impl<T: Read> SeqReader<T> {
    pub fn meta(&self) -> &PCDMeta {
        &self.meta
    }
}

impl<T: Read> Iterator for SeqReader<T> {
    type Item = Fallible<Vec<Value>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let row_result = match self.meta.data {
            DataKind::ASCII => crate::utils::scan_line(
                &mut self.reader,
                &self.meta.field_defs,
                &mut self.line_count,
            ),
            DataKind::Binary => crate::utils::scan_chunk(&mut self.reader, &self.meta.field_defs),
        };

        match row_result {
            Ok(row) => {
                self.point_count += 1;
                if self.point_count == self.meta.num_points {
                    self.finished = true;
                }
                Some(Ok(row))
            }
            Err(err) => {
                // Fuse the iterator if error occurs
                self.finished = true;
                Some(Err(err))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.meta.num_points as usize;
        (size, Some(size))
    }
}
