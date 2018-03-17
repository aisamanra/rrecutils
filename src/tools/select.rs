extern crate clap;
extern crate rrecutils;
extern crate failure;

mod common;

use failure::Error;

fn rr_select_args() -> clap::ArgMatches<'static> {
    clap::App::new("rr-sel")
        .version(common::VERSION)
        .author(common::AUTHOR)
        .about("Print records from a recfile")

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

        .arg(clap::Arg::with_name("type")
             .long("type")
             .short("t")
             .required(false)
             .takes_value(true))

        .arg(clap::Arg::with_name("include-descriptors")
             .long("include-descriptors")
             .short("d")
             .required(false)
             .takes_value(false))

        .arg(clap::Arg::with_name("collapse")
             .long("collapse")
             .short("C")
             .required(false)
             .takes_value(false))

        .arg(clap::Arg::with_name("sort")
             .long("sort")
             .short("S")
             .required(false)
             .takes_value(true))

        .arg(clap::Arg::with_name("group-by")
             .long("group-by")
             .short("G")
             .required(false)
             .takes_value(true))

        .get_matches()
}

fn run() -> Result<(), Error> {
    let matches = rr_select_args();

    let input = common::input_from_spec(
        matches.value_of("input"))?;
    let mut output = common::output_from_spec(
        matches.value_of("output"))?;

    let mut records = rrecutils::Recfile::parse(input)?;

    if let Some(typ) = matches.value_of("type") {
        records.filter_by_type(typ);
    }

    records.write(&mut output)?;

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
