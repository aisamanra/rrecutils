extern crate clap;
extern crate rrecutils;
extern crate rustache;

use std::{fs,io};
use std::convert::From;
use std::string::FromUtf8Error;

use rustache::Render;

struct R {
    rec: rrecutils::Record
}

impl Render for R {
    fn render<W: io::Write>(
        &self,
        template: &str,
        writer: &mut W,
    ) -> Result<(), rustache::RustacheError>
    {
        use rustache::HashBuilder;
        let mut hb = HashBuilder::new();
        if let Some(ref t) = self.rec.rec_type {
            hb = hb.insert("%rec", t.clone());
        }
        for field in self.rec.fields.iter() {
            hb = hb.insert(&field.0, field.1.clone());
        }
        hb.render(template, writer)
    }
}

enum FormatErr {
    IOError(io::Error),
    Utf8Error(FromUtf8Error),
    Rustache(rustache::RustacheError),
    Generic(String),
}

impl From<io::Error> for FormatErr {
    fn from(err: io::Error) -> FormatErr {
        FormatErr::IOError(err)
    }
}

impl From<FromUtf8Error> for FormatErr {
    fn from(err: FromUtf8Error) -> FormatErr {
        FormatErr::Utf8Error(err)
    }
}

impl From<rustache::RustacheError> for FormatErr {
    fn from(err: rustache::RustacheError) -> FormatErr {
        FormatErr::Rustache(err)
    }
}

impl From<String> for FormatErr {
    fn from(err: String) -> FormatErr {
        FormatErr::Generic(err)
    }
}


fn run() -> Result<(), FormatErr> {
    let matches = clap::App::new("rr-format")
        .version("0.0")
        .author("Getty Ritter <rrecutils@infinitenegativeutility.com>")
        .about("Display the Rust AST for a Recutils file")
        .arg(clap::Arg::with_name("input")
             .short("i")
             .long("input")
             .value_name("FILE")
             .help("The input recfile (or - for stdin)"))
        .arg(clap::Arg::with_name("output")
             .short("o")
             .long("output")
             .value_name("FILE")
             .help("The desired output location (or - for stdout)"))
        .arg(clap::Arg::with_name("template")
             .short("t")
             .long("template")
             .value_name("FILE")
             .help("The template to use"))
        .arg(clap::Arg::with_name("joiner")
             .short("j")
             .long("joiner")
             .value_name("STRING")
             .help("The string used to separate each fragment"))
        .get_matches();

    let stdin = io::stdin();

    let input: Box<io::BufRead> =
        match matches.value_of("input").unwrap_or("-") {
            "-" => Box::new(stdin.lock()),
            path =>
                Box::new(io::BufReader::new(fs::File::open(path)?)),
        };

    let template: String = match matches.value_of("template") {
        Some(path) => {
            use io::Read;
            let mut buf = Vec::new();
            fs::File::open(path)?.read_to_end(&mut buf)?;
            String::from_utf8(buf)?
        },
        None => Err(format!("No template specified!"))?,
    };

    let recfile = rrecutils::Recfile::parse(input)?;

    let mut output: Box<io::Write> =
        match matches.value_of("output").unwrap_or("-") {
            "-" => Box::new(io::stdout()),
            path => Box::new(fs::File::open(path)?),
        };

    for r in recfile.records.into_iter() {
        R { rec: r }.render(&template, &mut output.as_mut())?;
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(err) => panic!(err),
    }
}
