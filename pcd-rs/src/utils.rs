use crate::{
    error::Error,
    metas::{DataKind, FieldDef, PcdMeta, Schema, TypeKind, ValueKind, ViewPoint},
};
use anyhow::Result;
use std::{collections::HashSet, io::prelude::*};

pub fn load_meta<R: BufRead>(reader: &mut R, line_count: &mut usize) -> Result<PcdMeta> {
    let mut get_meta_line = |expect_entry: &str| -> Result<_> {
        loop {
            let mut line = String::new();
            let read_size = reader.read_line(&mut line)?;
            *line_count += 1;

            if read_size == 0 {
                return Err(Error::new_parse_error(*line_count, "Unexpected end of file").into());
            }

            let line_stripped = match line.split('#').next() {
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
                return Err(Error::new_parse_error(*line_count, &desc).into());
            }

            if tokens[0] != expect_entry {
                let desc = format!(
                    "Expect {:?} entry, found {:?} at line {}",
                    expect_entry,
                    tokens[0],
                    *line_count + 1
                );
                return Err(Error::new_parse_error(*line_count, &desc).into());
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
                    return Err(Error::new_parse_error(*line_count, &desc).into());
                }
            }
        } else {
            return Err(
                Error::new_parse_error(*line_count, "VERSION line is not understood").into(),
            );
        }
    };

    let meta_fields = {
        let tokens = get_meta_line("FIELDS")?;
        if tokens.len() == 1 {
            return Err(
                Error::new_parse_error(*line_count, "FIELDS line is not understood").into(),
            );
        }

        let mut name_set = HashSet::new();
        let mut field_names: Vec<String> = vec![];

        for (idx, tk) in tokens[1..].iter().enumerate() {
            let mut field = tk.clone();
            // If this field is just an underscore, it was meant to be skipped. Label it as
            // unknown_field_{idx}
            if field == String::from("_") {
                field = format!("unknown_field_{idx}");
            }

            if name_set.contains(&field.clone()) {
                let desc = format!("field name {:?} is specified more than once", field);
                return Err(Error::new_parse_error(*line_count, &desc).into());
            }

            name_set.insert(field.clone());
            field_names.push(field.to_owned());
        }

        field_names
    };

    let meta_size = {
        let tokens = get_meta_line("SIZE")?;
        if tokens.len() == 1 {
            return Err(Error::new_parse_error(*line_count, "SIZE line is not understood").into());
        }

        let mut sizes = vec![];
        for tk in tokens[1..].iter() {
            let size: u64 = tk.parse()?;
            sizes.push(size);
        }

        sizes
    };

    let meta_type = {
        let tokens = get_meta_line("TYPE")?;

        if tokens.len() == 1 {
            return Err(Error::new_parse_error(*line_count, "TYPE line is not understood").into());
        }

        let mut types = vec![];
        for type_char in tokens[1..].iter() {
            let type_ = match type_char.as_str() {
                "I" => TypeKind::I,
                "U" => TypeKind::U,
                "F" => TypeKind::F,
                _ => {
                    let desc = format!("Invalid type character {:?} in TYPE line", type_char);
                    return Err(Error::new_parse_error(*line_count, &desc).into());
                }
            };
            types.push(type_);
        }

        types
    };

    let meta_count = {
        let tokens = get_meta_line("COUNT")?;

        if tokens.len() == 1 {
            return Err(Error::new_parse_error(*line_count, "COUNT line is not understood").into());
        }

        let mut counts = vec![];
        for tk in tokens[1..].iter() {
            let count: u64 = tk.parse()?;
            counts.push(count);
        }

        counts
    };

    let meta_width = {
        let tokens = get_meta_line("WIDTH")?;

        if tokens.len() != 2 {
            return Err(Error::new_parse_error(*line_count, "WIDTH line is not understood").into());
        }

        let width: u64 = tokens[1].parse()?;
        width
    };

    let meta_height = {
        let tokens = get_meta_line("HEIGHT")?;
        if tokens.len() != 2 {
            return Err(
                Error::new_parse_error(*line_count, "HEIGHT line is not understood").into(),
            );
        }

        let height: u64 = tokens[1].parse()?;
        height
    };

    let meta_viewpoint = {
        let tokens = get_meta_line("VIEWPOINT")?;

        if tokens.len() != 8 {
            return Err(
                Error::new_parse_error(*line_count, "VIEWPOINT line is not understood").into(),
            );
        }

        let tx = tokens[1].parse()?;
        let ty = tokens[2].parse()?;
        let tz = tokens[3].parse()?;
        let qw = tokens[4].parse()?;
        let qx = tokens[5].parse()?;
        let qy = tokens[6].parse()?;
        let qz = tokens[7].parse()?;
        ViewPoint {
            tx,
            ty,
            tz,
            qw,
            qx,
            qy,
            qz,
        }
    };

    let meta_points = {
        let tokens = get_meta_line("POINTS")?;

        if tokens.len() != 2 {
            return Err(
                Error::new_parse_error(*line_count, "POINTS line is not understood").into(),
            );
        }

        let count: u64 = tokens[1].parse()?;
        count
    };

    let meta_data = {
        let tokens = get_meta_line("DATA")?;

        if tokens.len() != 2 {
            return Err(Error::new_parse_error(*line_count, "DATA line is not understood").into());
        }

        match tokens[1].as_str() {
            "ascii" => DataKind::Ascii,
            "binary" => DataKind::Binary,
            _ => {
                return Err(
                    Error::new_parse_error(*line_count, "DATA line is not understood").into(),
                );
            }
        }
    };

    // Check integrity
    if meta_size.len() != meta_fields.len() {
        return Err(
            Error::new_parse_error(*line_count, "SIZE entry conflicts with FIELD entry").into(),
        );
    }

    if meta_type.len() != meta_fields.len() {
        return Err(
            Error::new_parse_error(*line_count, "TYPE entry conflicts with FIELD entry").into(),
        );
    }

    if meta_count.len() != meta_fields.len() {
        return Err(
            Error::new_parse_error(*line_count, "COUNT entry conflicts with FIELD entry").into(),
        );
    }

    // Organize field type
    let field_defs: Result<Schema> = {
        meta_fields
            .iter()
            .zip(meta_type.iter())
            .zip(meta_size.iter())
            .zip(meta_count.iter())
            .map(|(((name, type_), size), &count)| {
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
                        return Err(Error::new_parse_error(*line_count, &desc).into());
                    }
                };

                let meta = FieldDef {
                    name: name.to_owned(),
                    kind,
                    count,
                };

                Ok(meta)
            })
            .collect()
    };

    let meta = PcdMeta {
        version: meta_version,
        field_defs: field_defs?,
        width: meta_width,
        height: meta_height,
        viewpoint: meta_viewpoint,
        num_points: meta_points,
        data: meta_data,
    };

    Ok(meta)
}
