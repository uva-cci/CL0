use chumsky::Parser;
use cl0_parser::{lexer::lexer, token::Token};

/// Lex `src` and return the raw `Vec<Token>` (panicking on lexer‐errors).
pub fn lex_tokens(src: &str) -> Vec<Token<'_>> {
    let lex = lexer().parse(src);
    if lex.has_errors() {
        let errs: Vec<_> = lex.errors().collect();
        panic!("lexer errors for {:?}: {:#?}", src, errs);
    }
    // grab owned Vec<(Token,Span)>
    let spanned = lex.output().cloned().expect("no tokens");
    // strip spans
    spanned.into_iter().map(|(t, _)| t).collect()
}