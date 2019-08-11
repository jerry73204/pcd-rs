use crate::{error::ParseError, FieldDef, Value, ValueKind};
use byteorder::{LittleEndian, ReadBytesExt};
use failure::Fallible;
use std::io::prelude::*;

pub fn scan_line<R: BufRead>(
    reader: &mut R,
    field_defs: &[FieldDef],
    line_count: &mut usize,
) -> Fallible<Vec<Value>> {
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
        return Err(ParseError::new(&desc).into());
    }

    let mut tokens = line.split_ascii_whitespace();

    let mut row = vec![];

    for meta in field_defs {
        let value = match (&meta.kind, meta.count) {
            (ValueKind::U8, 1) => {
                let val: u8 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::U8(val)
            }
            (ValueKind::U16, 1) => {
                let val: u16 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::U16(val)
            }
            (ValueKind::U32, 1) => {
                let val: u32 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::U32(val)
            }
            (ValueKind::I8, 1) => {
                let val: i8 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::I8(val)
            }
            (ValueKind::I16, 1) => {
                let val: i16 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::I16(val)
            }
            (ValueKind::I32, 1) => {
                let val: i32 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::I32(val)
            }
            (ValueKind::F32, 1) => {
                let val: f32 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::F32(val)
            }
            (ValueKind::F64, 1) => {
                let val: f64 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                Value::F64(val)
            }
            (ValueKind::U8, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: u8 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::U8V(values)
            }
            (ValueKind::U16, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: u16 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::U16V(values)
            }
            (ValueKind::U32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: u32 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::U32V(values)
            }
            (ValueKind::I8, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: i8 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::I8V(values)
            }
            (ValueKind::I16, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: i16 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::I16V(values)
            }
            (ValueKind::I32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: i32 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::I32V(values)
            }
            (ValueKind::F32, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: f32 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::F32V(values)
            }
            (ValueKind::F64, _) => {
                let mut values = vec![];
                for _ in 0..(meta.count) {
                    let val: f64 = tokens.next().ok_or(make_error(*line_count))?.parse()?;
                    values.push(val);
                }
                Value::F64V(values)
            }
        };

        row.push(value);
    }

    Ok(row)
}

pub fn scan_chunk<R: BufRead>(reader: &mut R, field_defs: &[FieldDef]) -> Fallible<Vec<Value>> {
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
