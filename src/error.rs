//! The module defines most error type used by this crate.

use crate::metas::{FieldDef, ValueKind};

/// The error returned from the crate.
#[derive(Debug, Fail)]
pub enum PcdError {
    #[fail(display = "Failed to parse PCD format at line {}: {}", line, desc)]
    ParseError { line: usize, desc: String },
    #[fail(
        display = "File schema and record schema mismatch. Expect {:?}, but found {:?}",
        expect, found
    )]
    SchemaMismatchError {
        expect: Vec<(Option<String>, ValueKind, Option<usize>)>,
        found: Vec<(String, ValueKind, usize)>,
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
    #[fail(display = "Invalid argument: {}", desc)]
    InvalidArgumentError { desc: String },
}

impl PcdError {
    pub fn new_parse_error(line: usize, desc: &str) -> PcdError {
        PcdError::ParseError {
            line,
            desc: desc.to_owned(),
        }
    }

    pub fn new_schema_mismatch_error(
        record_fields: &[(Option<String>, ValueKind, Option<usize>)],
        file_fields: &[FieldDef],
    ) -> PcdError {
        let expect = record_fields.to_vec();
        let found = file_fields
            .iter()
            .map(|field_def| {
                (
                    field_def.name.to_owned(),
                    field_def.kind,
                    field_def.count as usize,
                )
            })
            .collect::<Vec<_>>();
        PcdError::SchemaMismatchError { expect, found }
    }

    pub fn new_field_size_mismatch_error(
        field_name: &str,
        expect: usize,
        found: usize,
    ) -> PcdError {
        PcdError::FieldSizeMismatchError {
            field_name: field_name.to_owned(),
            expect,
            found,
        }
    }

    pub fn new_text_token_mismatch_error(expect: usize, found: usize) -> PcdError {
        PcdError::TextTokenMismatchError { expect, found }
    }

    pub fn new_invalid_argument_error(desc: &str) -> PcdError {
        PcdError::InvalidArgumentError {
            desc: desc.to_owned(),
        }
    }
}
