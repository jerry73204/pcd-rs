/// The error is raised when a PCD file is not understood by parser.
use crate::{FieldDef, ValueKind};

#[derive(Debug, Fail)]
pub enum PCDError {
    #[fail(display = "Failed to parse PCD format at line {}: {}", line, desc)]
    ParseError { line: usize, desc: String },
    #[fail(
        display = "File schema and record schema mismatch. Expect {:?}, but found {:?}",
        expect, found
    )]
    SchemaMismatchError {
        expect: Vec<(ValueKind, Option<usize>)>,
        found: Vec<(ValueKind, usize)>,
    },
    #[fail(
        display = "Expects {:?} elements in \"field\", but found {:?} elements in record",
        expect, found
    )]
    FieldSizeMismatchError {
        field_name: String,
        expect: usize,
        found: usize,
    },
    #[fail(
        display = "Record has {} fields, but the line has {} tokens",
        expect, found
    )]
    TextTokenMismatchError { expect: usize, found: usize },
}

impl PCDError {
    pub fn new_parse_error(line: usize, desc: &str) -> PCDError {
        PCDError::ParseError {
            line,
            desc: desc.to_owned(),
        }
    }

    pub fn new_schema_mismatch_error(
        record_fields: &[(ValueKind, Option<usize>)],
        file_fields: &[FieldDef],
    ) -> PCDError {
        let expect = record_fields.to_vec();
        let found = file_fields
            .iter()
            .map(|field_def| (field_def.kind, field_def.count as usize))
            .collect::<Vec<_>>();
        PCDError::SchemaMismatchError { expect, found }
    }

    pub fn new_field_size_mismatch_error(
        field_name: &str,
        expect: usize,
        found: usize,
    ) -> PCDError {
        PCDError::FieldSizeMismatchError {
            field_name: field_name.to_owned(),
            expect,
            found,
        }
    }

    pub fn new_text_token_mismatch_error(expect: usize, found: usize) -> PCDError {
        PCDError::TextTokenMismatchError { expect, found }
    }
}