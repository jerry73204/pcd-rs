/// The error is raised when a PCD file is not understood by parser.
#[derive(Debug, Fail)]
#[fail(display = "Failed to parse PCD data: {}", desc)]
pub struct ParseError {
    desc: String,
}

impl ParseError {
    pub fn new(desc: &str) -> ParseError {
        ParseError {
            desc: desc.to_owned(),
        }
    }
}
