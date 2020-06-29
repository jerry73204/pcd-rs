//! The module defines most error type used by this crate.

use crate::metas::{FieldDef, ValueKind};

/// The error returned from the crate.
#[derive(Debug, thiserror::Error)]
pub enum PcdError {
    #[error("parsing error at line {line}: {desc}")]
    ParseError { line: usize, desc: String },
    #[error("schema mismatch error, expect {expect:?}, but found {found:?}")]
    SchemaMismatchError {
        expect: Vec<(Option<String>, ValueKind, Option<usize>)>,
        found: Vec<(String, ValueKind, usize)>,
    },
    #[error(
        "field size mismatch, expect {expect} elements in \"{field_name}\" field, but found {found} elements in record",
    )]
    FieldSizeMismatchError {
        field_name: String,
        expect: usize,
        found: usize,
    },
    #[error("record has {expect} fields, but the line has {found} tokens")]
    TextTokenMismatchError { expect: usize, found: usize },
    #[error("Invalid argument: {desc}")]
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
