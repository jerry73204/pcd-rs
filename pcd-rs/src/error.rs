//! The module defines most error type used by this crate.

use crate::metas::{FieldDef, ValueKind};
use std::{
    io,
    num::{ParseFloatError, ParseIntError},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The error returned from the crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parsing error at line {line}: {desc}")]
    ParseError { line: usize, desc: String },

    #[error("reader schema mismatch error, expect {expect:?}, but found {found:?}")]
    ReaderSchemaMismatchError {
        expect: Vec<(Option<String>, ValueKind, Option<usize>)>,
        found: Vec<FieldDef>,
    },

    #[error("writer schema mismatch error, expect {expect:?}, but found {found:?}")]
    WriterSchemaMismatchError {
        expect: Vec<FieldDef>,
        found: Vec<FieldDef>,
    },

    #[error(
        r#"field size mismatch, expect {expect} elements in "{field_name}" field, but found {found} elements in record"#,
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

    #[error("Invalid writer configuration: {desc}")]
    InvalidWriterConfiguration { desc: String },

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("{0}")]
    ParseFloatError(#[from] ParseFloatError),
}

impl Error {
    pub fn new_parse_error(line: usize, desc: &str) -> Error {
        Error::ParseError {
            line,
            desc: desc.to_owned(),
        }
    }

    pub fn new_reader_schema_mismatch_error(
        expect: Vec<(Option<String>, ValueKind, Option<usize>)>,
        found: Vec<FieldDef>,
    ) -> Error {
        Error::ReaderSchemaMismatchError { expect, found }
    }

    pub fn new_writer_schema_mismatch_error(expect: Vec<FieldDef>, found: Vec<FieldDef>) -> Error {
        Error::WriterSchemaMismatchError { expect, found }
    }

    pub fn new_field_size_mismatch_error(field_name: &str, expect: usize, found: usize) -> Error {
        Error::FieldSizeMismatchError {
            field_name: field_name.to_owned(),
            expect,
            found,
        }
    }

    pub fn new_text_token_mismatch_error(expect: usize, found: usize) -> Error {
        Error::TextTokenMismatchError { expect, found }
    }

    pub fn new_invalid_argument_error(desc: &str) -> Error {
        Error::InvalidArgumentError {
            desc: desc.to_owned(),
        }
    }

    pub fn new_invalid_writer_configuration_error(desc: &str) -> Error {
        Error::InvalidWriterConfiguration {
            desc: desc.to_owned(),
        }
    }
}
