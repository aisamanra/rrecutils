#[macro_use] extern crate failure;

pub mod contlines;

use contlines::ContinuationLines;


struct ParsingContext {
    current_record_type: Option<String>,
}


/// A `Record` is a single bundle of key-value pairs with a few pieces
/// of optional metadata. This preserves the order of the values
/// contained.
#[derive(Eq, PartialEq, Debug)]
pub struct Record {
    pub rec_type: Option<String>,
    pub fields: Vec<(String, String)>,
}

impl Record {
    /// Write the serialized version of this `Record` to the provided `Write`r
    pub fn write<W>(&self, w: &mut W) -> std::io::Result<()>
        where W: std::io::Write
    {
        for &(ref name, ref value) in self.fields.iter() {
            write!(w, "{}: {}\n", name, value)?;
        }

        write!(w, "\n")
    }

    /// Turn this `Record` into a serialized string representation
    pub fn to_string(&self) -> std::io::Result<String> {
        let mut s = std::io::Cursor::new(Vec::new());
        self.write(&mut s)?;
        // XXX: this SHOULD be fine, but make sure!
        Ok(String::from_utf8(s.into_inner()).unwrap())
    }

    /// Return the number of fields in this record
    pub fn size(&self) -> usize {
        self.fields.len()
    }

    /// Return the value of the field named by the argument if it
    /// exists
    pub fn get<'a>(&'a self, name: &str) -> Result<&'a str, RecError> {
        self.fields.iter()
            .find(|&&(ref p, _)| p == name)
            .map(|&(_, ref q)| q.as_ref())
            .ok_or(RecError::MissingField { name: name.to_owned() })
    }
}


/// A `Recfile` is a sequence of `Record`.
#[derive(Eq, PartialEq, Debug)]
pub struct Recfile {
    pub records: Vec<Record>,
}

impl Recfile {
    /// Serialize this `Recfile` to the provided `Write`r
    pub fn write<W>(&self, w: &mut W) -> std::io::Result<()>
        where W: std::io::Write
    {
        for r in self.records.iter() {
            r.write(w)?;
        }

        Ok(())
    }

    /// Turn this `Recfile` into a serialized string representation
    pub fn to_string(&self) -> std::io::Result<String> {
        let mut s = std::io::Cursor::new(Vec::new());
        self.write(&mut s)?;
        // XXX: this SHOULD be fine, but make sure!
        Ok(String::from_utf8(s.into_inner()).unwrap())
    }

    /// Modify this Recfile in-place by only keeping the records of a
    /// particular type
    pub fn filter_by_type(&mut self, type_name: &str) {
        self.records.retain(|r| match r.rec_type {
            Some(ref t) => t == type_name,
            None => false,
        });
    }

    /// Iterate over a subset of the records in this recfile
    pub fn iter_by_type<'a>(&'a self, type_name: &'a str) -> RecIterator<'a> {
        RecIterator {
            typ: type_name,
            rec: self.records.iter(),
        }
    }

    /// Iterate over _all_ the records in this recfile
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, Record> {
        self.records.iter()
    }
}

pub struct RecIterator<'a> {
    typ: &'a str,
    rec: std::slice::Iter<'a, Record>,
}

impl<'a> Iterator for RecIterator<'a> {
    type Item = &'a Record;

    fn next(&mut self) -> Option<&'a Record> {
        while let Some(r) = self.rec.next() {
            match r.rec_type {
                Some(ref n) if n == self.typ => return Some(r),
                _ => (),
            }
        }
        return None;
    }
}

#[derive(Debug, Fail)]
pub enum RecError {
    #[fail(display = "Error parsing records: {}", message)]
    GenericError {
        message: String,
    },

    #[fail(display = "Found cont line in nonsensical place: {}", ln)]
    BadContLine {
        ln: String,
    },

    #[fail(display = "Invalid line: {}", ln)]
    InvalidLine {
        ln: String,
    },

    #[fail(display = "Missing key: {}", name)]
    MissingField {
        name: String,
    },
}


impl Recfile {
    pub fn parse<I>(i: I) -> Result<Recfile, RecError>
        where I: std::io::BufRead
    {
        let mut iter = ContinuationLines::new(i.lines());
        let mut current = Record {
            fields: vec![],
            rec_type: None,
        };
        let mut buf = vec![];
        let mut ctx = ParsingContext {
            current_record_type: None,
        };

        while let Some(Ok(ln)) = iter.next() {
            let ln = ln.trim_left_matches(' ');

            if ln.starts_with('#') {
                // skip comment lines
            } else if ln.is_empty() {
                if !current.fields.is_empty() {
                    buf.push(current);
                    current = Record {
                        rec_type: ctx.current_record_type.clone(),
                        fields: vec![],
                    };
                }
            } else if ln.starts_with('+') {
                if let Some(val) = current.fields.last_mut() {
                    val.1.push_str("\n");
                    val.1.push_str(
                        if ln[1..].starts_with(' ') {
                            &ln[2..]
                        } else {
                            &ln[1..]
                        });
                } else {
                    return Err(RecError::BadContLine{ ln: ln.to_owned() });
                }
            } else if let Some(pos) = ln.find(':') {
                let (key, val) = ln.split_at(pos);
                current.fields.push((
                    key.to_owned(),
                    val[1..].trim_left().to_owned()));
                if key == "%rec" {
                    ctx.current_record_type = Some(val[1..].trim_left().to_owned());
                    current.rec_type = None;
                }
            } else {
                return Err(RecError::InvalidLine { ln: ln.to_owned() });
            }
        }

        if !current.fields.is_empty() {
            buf.push(current);
        }

        Ok(Recfile { records: buf })
    }

}

#[cfg(test)]
mod tests {
    use ::{Recfile,Record};

    fn test_parse(input: &[u8], expected: Vec<Vec<(&str, &str)>>) {
        let file = Recfile {
            records: expected.iter().map( |v| {
                Record {
                    rec_type: None,
                    fields: v.iter().map( |&(k, v)| {
                        (k.to_owned(), v.to_owned())
                    }).collect(),
                }
            }).collect(),
        };
        assert_eq!(Recfile::parse(input), Ok(file));
    }

    #[test]
    fn empty_file() {
        test_parse(b"\n", vec![]);
    }

    #[test]
    fn only_comments() {
        test_parse(b"# an empty file\n", vec![]);
    }

    #[test]
    fn one_section() {
        test_parse(b"hello: yes\n", vec![ vec![ ("hello", "yes") ] ]);
    }

    #[test]
    fn two_sections() {
        test_parse(
            b"hello: yes\n\ngoodbye: no\n",
            vec![
                vec![ ("hello", "yes") ],
                vec![ ("goodbye", "no") ],
            ],
        );
    }

    #[test]
    fn continuation_with_space() {
        test_parse(
            b"hello: yes\n+ but also no\n",
            vec![
                vec![ ("hello", "yes\nbut also no") ],
            ],
        );
    }

    #[test]
    fn continuation_without_space() {
        test_parse(
            b"hello: yes\n+but also no\n",
            vec![
                vec![ ("hello", "yes\nbut also no") ],
            ],
        );
    }

    #[test]
    fn continuation_with_two_spaces() {
        test_parse(
            b"hello: yes\n+  but also no\n",
            vec![
                vec![ ("hello", "yes\n but also no") ],
            ],
        );
    }

}
