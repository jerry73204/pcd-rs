use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::fs::File;
use std::fmt;
use std::error::Error;
use std::collections::HashSet;

pub struct PCD {
}

pub struct ParseError {
    desc: String,
}

pub enum FieldType {
    I, U, F
}

impl PCD {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<PCD, Box<dyn Error + Send + Sync>> {
        let file = File::open(path)?;
        let lines = BufReader::new(file).lines();

        let mut meta_version = None;
        let mut meta_fields = None;
        let mut meta_size = None;
        let mut meta_type = None;
        // let mut meta_count = None;
        // let mut meta_width = None;
        // let mut meta_height = None;
        // let mut meta_viewpoint = None;
        // let mut meta_points = None;

        // Read meta data
        for line_result in lines {
            let line = line_result?;
            let line_without_comment = line.split('#').nth(0).unwrap();
            let tokens: Vec<&str> = line_without_comment.split_ascii_whitespace()
                .collect();

            match tokens[0] {
                "VERSION" => {
                    if let Some(_) = meta_version {
                        return Err(Box::new(
                            ParseError::new("VERSION is set more than once")
                        ));
                    }

                    if tokens.len() == 2 {
                        if tokens[1] == ".7" {
                            meta_version = Some(tokens[1].to_owned());
                        }
                        else {
                            return Err(Box::new(
                                ParseError::new("Unsupported version. Supported versions are: 0.7")
                            ));
                        }
                    }
                    else {
                        return Err(Box::new(
                            ParseError::new("VERSION line is not understood")
                        ));
                    }
                }

                "FIELDS" => {
                    if let Some(_) = meta_fields {
                        return Err(Box::new(
                            ParseError::new("FIELDS is set more than once")
                        ));
                    }

                    if tokens.len() == 1 {
                        return Err(Box::new(
                            ParseError::new("FIELDS line is not understood")
                        ));
                    }

                    let mut name_set = HashSet::new();
                    let mut field_names: Vec<String> = vec![];

                    for tk in tokens[1..].into_iter() {
                        let field = *tk;
                        if name_set.contains(field) {
                            return Err(Box::new(
                                ParseError::new(&format!("field name {:?} is specified more than once", field))
                            ));
                        }

                        name_set.insert(field);
                        field_names.push(field.to_owned());
                    }

                    meta_fields = Some(field_names);
                }
                "SIZE" => {
                    if let Some(_) = meta_size {
                        return Err(Box::new(
                            ParseError::new("SIZE is set more than once")
                        ));
                    }

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
                    meta_size = Some(sizes);
                }
                "TYPE" => {
                    if let Some(_) = meta_type {
                        return Err(Box::new(
                            ParseError::new("TYPE is set more than once")
                        ));
                    }

                    if tokens.len() == 1 {
                        return Err(Box::new(
                            ParseError::new("TYPE line is not understood")
                        ));
                    }

                    let mut types = vec![];
                    for tk in tokens[1..].into_iter() {
                        let type_char = *tk;
                        let type_ = match type_char {
                            "I" => FieldType::I,
                            "U" => FieldType::U,
                            "F" => FieldType::F,
                            _ => {
                                return Err(Box::new(
                                    ParseError::new(&format!("Invalid type character {:?} in TYPE line", type_char))
                                ));
                            }
                        };
                        types.push(type_);
                    }

                    meta_type = Some(types);
                }
                "COUNT" => {}
                "WIDTH" => {}
                "HEIGHT" => {}
                "VIEWPOINT" => {}
                "POINTS" => {}
                "DATA" => {}
                _ => {}
            }
        }

        let pcd = PCD {};

        Ok(pcd)
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


// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
