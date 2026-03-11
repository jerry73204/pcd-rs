use crate::{
    error::Error,
    metas::{DataKind, FieldDef, PcdMeta, Schema, TypeKind, ValueKind, ViewPoint},
    Result,
};
use std::{
    collections::{HashMap, HashSet},
    io::prelude::*,
};

/// Known header entry names. DATA must be last (signals end of header).
const KNOWN_ENTRIES: &[&str] = &[
    "VERSION",
    "FIELDS",
    "COLUMNS",
    "SIZE",
    "TYPE",
    "COUNT",
    "WIDTH",
    "HEIGHT",
    "VIEWPOINT",
    "POINTS",
    "DATA",
];

pub fn load_meta<R: BufRead>(reader: &mut R, line_count: &mut usize) -> Result<PcdMeta> {
    // Parse all header lines into a map. Stop when we see DATA.
    let mut header: HashMap<String, Vec<String>> = HashMap::new();
    loop {
        let mut line = String::new();
        let read_size = reader.read_line(&mut line)?;
        *line_count += 1;

        if read_size == 0 {
            return Err(Error::new_parse_error(
                *line_count,
                "Unexpected end of file before DATA line",
            ));
        }

        // Strip comments
        let line_stripped = match line.split('#').next() {
            Some(remaining) => remaining,
            None => continue,
        };

        let tokens: Vec<String> = line_stripped
            .split_ascii_whitespace()
            .map(|s| s.to_owned())
            .collect();

        if tokens.is_empty() {
            continue;
        }

        let key = tokens[0].to_uppercase();

        if !KNOWN_ENTRIES.contains(&key.as_str()) {
            // Skip unknown header entries
            continue;
        }

        header.insert(key.clone(), tokens);

        if key == "DATA" {
            break;
        }
    }

    // Helper to get a required entry
    let get_required = |key: &str| -> Result<&Vec<String>> {
        header.get(key).ok_or_else(|| {
            Error::new_parse_error(*line_count, &format!("{} entry not found in header", key))
        })
    };

    // VERSION (required)
    let meta_version = {
        let tokens = get_required("VERSION")?;
        if tokens.len() != 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "VERSION line is not understood",
            ));
        }
        match tokens[1].as_str() {
            "0.7" | ".7" => String::from("0.7"),
            "0.6" | ".6" => String::from("0.6"),
            "0.5" | ".5" => String::from("0.5"),
            _ => {
                let desc = format!(
                    "Unsupported version {:?}. Supported versions are: 0.5, 0.6, 0.7",
                    tokens[1]
                );
                return Err(Error::new_parse_error(*line_count, &desc));
            }
        }
    };

    // FIELDS or COLUMNS (required, COLUMNS is a PCL backward-compat alias)
    let meta_fields = {
        let tokens = header
            .get("FIELDS")
            .or_else(|| header.get("COLUMNS"))
            .ok_or_else(|| {
                Error::new_parse_error(*line_count, "FIELDS (or COLUMNS) entry not found in header")
            })?;

        if tokens.len() == 1 {
            return Err(Error::new_parse_error(
                *line_count,
                "FIELDS line is not understood",
            ));
        }

        let mut name_set = HashSet::new();
        let mut field_names: Vec<String> = vec![];

        for tk in tokens[1..].iter() {
            let field = tk.clone();

            // Padding fields (`_`) are allowed to repeat
            if field != "_" {
                if name_set.contains(&field) {
                    let desc = format!("field name {:?} is specified more than once", field);
                    return Err(Error::new_parse_error(*line_count, &desc));
                }
                name_set.insert(field.clone());
            }

            field_names.push(field);
        }

        field_names
    };

    let num_fields = meta_fields.len();

    // SIZE (required)
    let meta_size = {
        let tokens = get_required("SIZE")?;
        if tokens.len() < 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "SIZE line is not understood",
            ));
        }
        tokens[1..]
            .iter()
            .map(|tk| Ok(tk.parse::<u64>()?))
            .collect::<Result<Vec<_>>>()?
    };

    // TYPE (required)
    let meta_type = {
        let tokens = get_required("TYPE")?;
        if tokens.len() < 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "TYPE line is not understood",
            ));
        }
        tokens[1..]
            .iter()
            .map(|type_char| match type_char.as_str() {
                "I" => Ok(TypeKind::I),
                "U" => Ok(TypeKind::U),
                "F" => Ok(TypeKind::F),
                _ => {
                    let desc = format!("Invalid type character {:?} in TYPE line", type_char);
                    Err(Error::new_parse_error(*line_count, &desc))
                }
            })
            .collect::<Result<Vec<_>>>()?
    };

    // COUNT (optional, defaults to 1 for each field)
    let meta_count: Vec<u64> = if let Some(tokens) = header.get("COUNT") {
        if tokens.len() < 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "COUNT line is not understood",
            ));
        }
        tokens[1..]
            .iter()
            .map(|tk| {
                let count: u64 = tk.parse()?;
                // PCL treats COUNT 0 as 1
                Ok(if count == 0 { 1 } else { count })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![1; num_fields]
    };

    // WIDTH (required)
    let meta_width = {
        let tokens = get_required("WIDTH")?;
        if tokens.len() != 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "WIDTH line is not understood",
            ));
        }
        tokens[1].parse::<u64>()?
    };

    // HEIGHT (required)
    let meta_height = {
        let tokens = get_required("HEIGHT")?;
        if tokens.len() != 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "HEIGHT line is not understood",
            ));
        }
        tokens[1].parse::<u64>()?
    };

    // VIEWPOINT (optional, defaults to identity)
    let meta_viewpoint = if let Some(tokens) = header.get("VIEWPOINT") {
        if tokens.len() != 8 {
            return Err(Error::new_parse_error(
                *line_count,
                "VIEWPOINT line is not understood",
            ));
        }
        ViewPoint {
            tx: tokens[1].parse()?,
            ty: tokens[2].parse()?,
            tz: tokens[3].parse()?,
            qw: tokens[4].parse()?,
            qx: tokens[5].parse()?,
            qy: tokens[6].parse()?,
            qz: tokens[7].parse()?,
        }
    } else {
        ViewPoint::default()
    };

    // POINTS (required)
    let meta_points = {
        let tokens = get_required("POINTS")?;
        if tokens.len() != 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "POINTS line is not understood",
            ));
        }
        tokens[1].parse::<u64>()?
    };

    // DATA (required, already confirmed present)
    let meta_data = {
        let tokens = get_required("DATA")?;
        if tokens.len() != 2 {
            return Err(Error::new_parse_error(
                *line_count,
                "DATA line is not understood",
            ));
        }
        match tokens[1].as_str() {
            "ascii" => DataKind::Ascii,
            "binary" => DataKind::Binary,
            "binary_compressed" => {
                if meta_version != "0.7" {
                    let desc = format!(
                        "binary_compressed format is only supported in PCD version 0.7, found version {}",
                        meta_version
                    );
                    return Err(Error::new_parse_error(*line_count, &desc));
                }
                DataKind::BinaryCompressed
            }
            _ => {
                return Err(Error::new_parse_error(
                    *line_count,
                    "DATA line is not understood",
                ));
            }
        }
    };

    // Validate field counts match
    if meta_size.len() != num_fields {
        return Err(Error::new_parse_error(
            *line_count,
            "SIZE entry conflicts with FIELDS entry",
        ));
    }

    if meta_type.len() != num_fields {
        return Err(Error::new_parse_error(
            *line_count,
            "TYPE entry conflicts with FIELDS entry",
        ));
    }

    if meta_count.len() != num_fields {
        return Err(Error::new_parse_error(
            *line_count,
            "COUNT entry conflicts with FIELDS entry",
        ));
    }

    // Note: WIDTH * HEIGHT may not equal POINTS in files from some writers.
    // PCL validates this strictly, but many tools (including our own writer,
    // which pre-declares WIDTH) may produce mismatches. We trust POINTS as
    // the authoritative count.

    // Build field definitions
    let field_defs: Result<Schema> = meta_fields
        .iter()
        .zip(meta_type.iter())
        .zip(meta_size.iter())
        .zip(meta_count.iter())
        .map(|(((name, type_), size), &count)| {
            let kind = match (type_, size) {
                (TypeKind::U, 1) => ValueKind::U8,
                (TypeKind::U, 2) => ValueKind::U16,
                (TypeKind::U, 4) => ValueKind::U32,
                (TypeKind::U, 8) => ValueKind::U64,
                (TypeKind::I, 1) => ValueKind::I8,
                (TypeKind::I, 2) => ValueKind::I16,
                (TypeKind::I, 4) => ValueKind::I32,
                (TypeKind::I, 8) => ValueKind::I64,
                (TypeKind::F, 4) => ValueKind::F32,
                (TypeKind::F, 8) => ValueKind::F64,
                _ => {
                    let desc =
                        format!("Field type {:?} with size {} is not supported", type_, size);
                    return Err(Error::new_parse_error(*line_count, &desc));
                }
            };

            Ok(FieldDef {
                name: name.to_owned(),
                kind,
                count,
            })
        })
        .collect();

    Ok(PcdMeta {
        version: meta_version,
        field_defs: field_defs?,
        width: meta_width,
        height: meta_height,
        viewpoint: meta_viewpoint,
        num_points: meta_points,
        data: meta_data,
    })
}
