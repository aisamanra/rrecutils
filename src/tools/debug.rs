extern crate clap;
extern crate failure;
extern crate rrecutils;

mod common;

fn rr_debug_args() -> clap::ArgMatches<'static> {
    clap::App::new("rr-debug")
        .version("0.0")
        .author("Getty Ritter <rrecutils@infinitenegativeutility.com>")

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

        .arg(clap::Arg::with_name("pretty")
             .short("p")
             .long("pretty")
             .takes_value(false)
             .help("Whether to pretty-print the Rust AST"))

        .about("Display the Rust AST for a Recutils file")
        .get_matches()
}

fn main() {
    fn run() -> Result<(), failure::Error> {
        let matches = rr_debug_args();

        let input = common::input_from_spec(
            matches.value_of("input"))?;
        let mut output = common::output_from_spec(
            matches.value_of("output"))?;

        let records = rrecutils::Recfile::parse(input)?;

        if matches.is_present("pretty") {
            writeln!(output, "{:#?}", records)?;
        } else {
            writeln!(output, "{:?}", records)?;
        }

        Ok(())
    }

    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
