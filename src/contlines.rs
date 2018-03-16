use std::io;

/// An iterator that abstracts over continuation characters on
/// subsequent lines
pub struct ContinuationLines<R: Iterator<Item=io::Result<String>>> {
    underlying: R,
}

impl<R: Iterator<Item=io::Result<String>>> ContinuationLines<R> {
    fn join_next(&mut self, mut past: String) -> Option<io::Result<String>> {
        let next = self.underlying.next();
        match next {
            None => Some(Ok(past)),
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(ref new)) => {
                if new.ends_with("\\") {
                    let end = new.len() - 1;
                    past.push_str(&new[(0..end)]);
                    self.join_next(past)
                } else {
                    past.push_str(&new);
                    Some(Ok(past))
                }
            }
        }
    }

    pub fn new(iter: R) -> ContinuationLines<R> {
        ContinuationLines { underlying: iter }
    }
}

impl<R: Iterator<Item=io::Result<String>>> Iterator for ContinuationLines<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        let next = self.underlying.next();
        match next {
            None => None,
            Some(Err(err)) => Some(Err(err)),
            Some(Ok(x)) => {
                if x.ends_with("\\") {
                    let end = x.len() - 1;
                    self.join_next(x[(0..end)].to_owned())
                } else {
                    Some(Ok(x))
                }
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::ContinuationLines;
    use std::io::{BufRead, Cursor};

    fn test_contlines(input: &[u8], expected: Vec<&str>) {
        // build a ContinuationLines iterator from our input buffer,
        // and unwrap all the IO exceptions we would get
        let mut i = ContinuationLines::new(Cursor::new(input).lines())
            .map(Result::unwrap);
        // walk the expected values and make sure those are the ones
        // we're getting
        for e in expected.into_iter() {
            assert_eq!(i.next(), Some(e.to_owned()));
        }
        // and then make sure we're at the end
        assert_eq!(i.next(), None);
    }

    #[test]
    fn no_contlines() {
        test_contlines(b"foo\nbar\n", vec!["foo", "bar"]);
    }

    #[test]
    fn two_joined_lines() {
        test_contlines(b"foo\\\nbar\n", vec!["foobar"]);
    }

    #[test]
    fn three_joined_lines() {
        test_contlines(
            b"foo\\\nbar\\\nbaz\n",
            vec!["foobarbaz"],
        );
    }

    #[test]
    fn mixed_joins() {
        test_contlines(
            b"foo\nbar\\\nbaz\nquux\n",
            vec!["foo", "barbaz", "quux"],
        );
    }

}
