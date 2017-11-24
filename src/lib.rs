struct ParsingContext {
    continuation_line: bool,
    current_record_type: Option<String>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Record {
    pub rec_type: Option<String>,
    pub fields: Vec<(String, String)>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Recfile {
    pub records: Vec<Record>,
}


impl Recfile {
    pub fn parse<I>(i: I) -> Result<Recfile, String>
        where I: std::io::BufRead
    {
        let mut iter = i.lines();
        let mut current = Record {
            fields: vec![],
            rec_type: None,
        };
        let mut buf = vec![];
        let mut ctx = ParsingContext {
            continuation_line: false,
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
                    return Err(format!(
                        "Found continuation line in nonsensical place: {}",
                        ln));
                }
            } else if let Some(pos) = ln.find(':') {
                let (key, val) = ln.split_at(pos);
                current.fields.push((
                    key.to_owned(),
                    val[1..].trim_left().to_owned()));
                if key == "%rec" {
                    ctx.current_record_type = Some(val[1..].trim_left().to_owned());
                }
            } else {
                return Err(format!("Invalid line: {:?}", ln));
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
