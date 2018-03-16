extern crate clap;
extern crate rrecutils;
extern crate serde_json;

mod common;

use std::fmt;

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

fn unwrap_err<L, R: fmt::Debug>(value: Result<L, R>) -> L {
    match value {
        Ok(v) => v,
        Err(err) => {
            println!("{:?}", err);
            std::process::exit(99)
        }
    }
}

fn main() {
    let matches = clap::App::new("rr-to-json")
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
        .get_matches();

    let input = unwrap_err(common::input_from_spec(
        matches.value_of("input")));
    let mut output = unwrap_err(common::output_from_spec(
        matches.value_of("output")));

    let json = Value::Array(
        unwrap_err(rrecutils::Recfile::parse(input))
            .records
            .iter()
            .map(|x| record_to_json(x))
            .collect());

    let serialized = if matches.is_present("pretty") {
        unwrap_err(serde_json::to_string_pretty(&json))
    } else {
        json.to_string()
    };

    unwrap_err(writeln!(output, "{}", serialized));

}
