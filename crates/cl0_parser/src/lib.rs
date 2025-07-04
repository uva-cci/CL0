pub mod token;
pub mod lexer;
pub mod parser;
pub mod ast;

use chumsky::Parser;

use crate::lexer::lexer;
use crate::parser::program_parser;

pub fn parse_and_print<'src>(
    src: &'src str,
){
    // Lex
    let lex_result = lexer().parse(src);
    if lex_result.has_errors() {
        let errs: Vec<_> = lex_result.errors().collect();
        eprintln!("Lexer errors: {:#?}", errs);
        std::process::exit(1);
    }
    let spanned = lex_result.output().cloned().expect("no tokens");
    let tokens: Vec<_> = spanned.into_iter().map(|(t, _)| t).collect();

    // Parse
    let parse_result = program_parser().parse(tokens.as_slice());
    if parse_result.has_errors() {
        let errs: Vec<_> = parse_result.errors().collect();
        eprintln!("Parse errors: {:#?}", errs);
        std::process::exit(1);
    }
    let output = parse_result.output().expect("No output from parser");
    println!("{:#?}", output);
}