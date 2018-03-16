extern crate clap;
extern crate rrecutils;
extern crate rustache;

use std::{fs,io};
use std::convert::From;
use std::string::FromUtf8Error;

mod common;

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


fn rr_format_args() -> clap::ArgMatches<'static> {
    clap::App::new("rr-format")
        .version(common::VERSION)
        .author(common::AUTHOR)
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

        .arg(clap::Arg::with_name("mustache")
             .short("m")
             .long("mustache")
             .value_name("FILE")
             .help("The mustache template to use"))

        .arg(clap::Arg::with_name("type")
             .short("t")
             .long("type")
             .value_name("TYPE")
             .takes_value(true)
             .help("The type of records to pass to the mustache file"))

        .arg(clap::Arg::with_name("joiner")
             .short("j")
             .long("joiner")
             .value_name("STRING")
             .help("The string used to separate each fragment"))

        .get_matches()
}

fn run() -> Result<(), FormatErr> {
    let matches = rr_format_args();

    let input = common::input_from_spec(
        matches.value_of("input"))?;
    let mut output = common::output_from_spec(
        matches.value_of("output"))?;

    let template: String = match matches.value_of("mustache") {
        Some(path) => {
            use io::Read;
            let mut buf = Vec::new();
            fs::File::open(path)?.read_to_end(&mut buf)?;
            String::from_utf8(buf)?
        },
        None => Err(format!("No template specified!"))?,
    };

    let mut recfile = rrecutils::Recfile::parse(input)?;

    if let Some(typ) = matches.value_of("type") {
        recfile.filter_by_type(typ);
    }


    let joiner = matches.value_of("joiner");

    let mut first = true;
    for r in recfile.records.into_iter() {
        if first {
            first = false;
        } else if let Some(j) = joiner {
            output.write(j.as_bytes())?;
            output.write(&['\n' as u8])?;
        }
        R { rec: r }.render(&template, &mut output.as_mut())?;
    }

    Ok(())
}

fn main() {
    use FormatErr::*;
    match run() {
        Ok(()) => (),
        Err(IOError(_)) => panic!("IO Error"),
        Err(Utf8Error(_)) => panic!("Cannot decode as UTF-8"),
        Err(Rustache(r)) => panic!("Rustache error: {:?}", r),
        Err(Generic(s)) => panic!("{}", s),
    }
}
