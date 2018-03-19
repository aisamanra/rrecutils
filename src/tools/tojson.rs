extern crate clap;
extern crate failure;
extern crate rrecutils;
extern crate serde_json;

mod common;

use serde_json::Value;
use serde_json::map::Map;

fn record_to_json(rec: &rrecutils::Record) -> Value {
    let mut m = Map::new();
    for tup in rec.fields.iter() {
        let k = tup.0.clone();
        let v = tup.1.clone();
        m.insert(k, Value::String(v));
    }
    Value::Object(m)
}

fn rr_tojson_args() -> clap::ArgMatches<'static> {
    clap::App::new("rr-to-json")
        .version(common::VERSION)
        .author(common::AUTHOR)
        .about("Display the Rust AST for a Recutils file")
        .arg(clap::Arg::with_name("pretty")
             .short("p")
             .long("pretty")
             .help("Pretty-print the resulting JSON"))

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

        .get_matches()
}

fn run() -> Result<(), failure::Error> {
    let matches = rr_tojson_args();

    let input = common::input_from_spec(
        matches.value_of("input"))?;
    let mut output = common::output_from_spec(
        matches.value_of("output"))?;

    let json = Value::Array(
        rrecutils::Recfile::parse(input)?
            .records
            .iter()
            .map(|x| record_to_json(x))
            .collect());

    let serialized = if matches.is_present("pretty") {
        serde_json::to_string_pretty(&json)?
    } else {
        json.to_string()
    };

    writeln!(output, "{}", serialized)?;

    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e),
    }
}
