extern crate clap;
extern crate rrecutils;

fn main() {
    let matches = clap::App::new("rr-pretty")
        .version("0.0")
        .author("Getty Ritter <rrecutils@infinitenegativeutility.com>")
        .about("Display the Rust AST for a Recutils file")
        .get_matches();
    let source = std::io::stdin();
    let records = rrecutils::Recfile::parse(source.lock());
    println!("{:#?}", records);
}
