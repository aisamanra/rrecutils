extern crate clap;
extern crate rrecutils;

fn main() {
    let matches = clap::App::new("rr-sel")
        .version("0.0")
        .author("Getty Ritter <rrecutils@infinitenegativeutility.com>")
        .about("Print records from a recfile")

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

        .get_matches();

    let source = std::io::stdin();
    let mut records = rrecutils::Recfile::parse(source.lock()).unwrap();

    if let Some(typ) = matches.value_of("type") {
        records.filter_by_type(typ);
    }

    records.write(&mut std::io::stdout());
}
