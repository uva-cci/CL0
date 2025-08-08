
use cl0_parser::parse_and_print;

fn main() {
    let mut args = std::env::args();
    let _bin = args.next();
    let input = args.next().expect("Please provide a string to parse as the first argument.");

    parse_and_print(&input);
}