extern crate clap;
extern crate rrecutils;
extern crate rustache;
#[macro_use] extern crate failure;

use std::{fs,io};

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

        .arg(clap::Arg::with_name("output-files")
             .short("O")
             .long("output-files")
             .value_name("TEMPLATE")
             .help("The desired output "))

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


fn create_output_files(
    filename_template: &str,
    recfile: rrecutils::Recfile,
    template: String,
) -> Result<(), failure::Error> {
    for r in recfile.records.into_iter().map( |rec| R { rec } ) {
        let mut filename = std::io::Cursor::new(Vec::new());
        r.render(filename_template, &mut filename)
            .map_err(|e| format_err!("Rustache error: {:?}", e))?;
        let filename = String::from_utf8(filename.into_inner())?;
        println!("writing file `{}'", &filename);

        let mut file = std::fs::File::create(&filename)?;
        r.render(&template, &mut file)
            .map_err(|e| format_err!("Rustache error: {:?}", e))?;
    }
    Ok(())
}


fn render_to_single_file(
    mut output: Box<std::io::Write>,
    joiner: Option<&str>,
    recfile: rrecutils::Recfile,
    template: String,
) -> Result<(), failure::Error> {
    let mut first = true;
    for r in recfile.records.into_iter() {
        if first {
            first = false;
        } else if let Some(j) = joiner {
            output.write(j.as_bytes())?;
            output.write(&['\n' as u8])?;
        }
        R { rec: r }.render(&template, &mut output.as_mut())
            .map_err(|e| format_err!("Rustache error: {:?}", e))?;
        }

    Ok(())
}


fn run() -> Result<(), failure::Error> {
    let matches = rr_format_args();

    let input = common::input_from_spec(
        matches.value_of("input"))?;

    let mut recfile = rrecutils::Recfile::parse(input)?;
    if let Some(typ) = matches.value_of("type") {
        recfile.filter_by_type(typ);
    }

    let template: String = match matches.value_of("mustache") {
        Some(path) => {
            use io::Read;
            let mut buf = Vec::new();
            fs::File::open(path)?.read_to_end(&mut buf)?;
            String::from_utf8(buf)?
        },
        None => bail!("No template specified!"),
    };

    if let Some(filename) = matches.value_of("output-files") {
        create_output_files(filename, recfile, template)?;
    } else {
        render_to_single_file(
            common::output_from_spec(matches.value_of("output"))?,
            matches.value_of("joiner"),
            recfile,
            template,
        )?;
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
