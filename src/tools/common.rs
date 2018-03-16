use std::{fs,io};

pub const VERSION: &'static str = "0.0";
pub const AUTHOR: &'static str =
    "Getty Ritter <rrecutils@infinitenegativeutility.com>";

pub enum Input {
    Stdin(io::BufReader<io::Stdin>),
    File(io::BufReader<fs::File>),
}

impl io::Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            &mut Input::Stdin(ref mut stdin) =>
                stdin.read(buf),
            &mut Input::File(ref mut file) =>
                file.read(buf),
        }
    }
}

impl io::BufRead for Input {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            &mut Input::Stdin(ref mut stdin) =>
                stdin.fill_buf(),
            &mut Input::File(ref mut file) =>
                file.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            &mut Input::Stdin(ref mut stdin) =>
                stdin.consume(amt),
            &mut Input::File(ref mut file) =>
                file.consume(amt),
        }
    }
}

pub fn input_from_spec<'a>(
    spec: Option<&'a str>
) -> io::Result<Input> {
    match spec.unwrap_or("-") {
        "-" => Ok(Input::Stdin(io::BufReader::new(io::stdin()))),
        path => {
            let f = fs::File::open(path)?;
            Ok(Input::File(io::BufReader::new(f)))
        }
    }
}

pub fn output_from_spec<'a>(
    spec: Option<&'a str>
) -> io::Result<Box<io::Write>>
{
    match spec.unwrap_or("-") {
        "-" => Ok(Box::new(io::stdout())),
        path => Ok(Box::new(fs::File::open(path)?)),
    }
}
