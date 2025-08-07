use chumsky::Parser;
use cl0_parser::lexer::lexer;
use cl0_parser::token::Token;

#[test]
fn test_basic_lexer() {
    let input = "#click => a; b % comment here #";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(
        tokens,
        vec![
            Token::Hash,
            Token::Descriptor("click"),
            Token::FatArrow,
            Token::Descriptor("a"),
            Token::Semicolon,
            Token::Descriptor("b"),
        ]
    );
}

#[test]
fn negate_condition() {
    let input = "not condition";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(tokens, vec![Token::Not, Token::Descriptor("condition")]);
}

#[test]
fn dot_vs_endrule1() {
    let input = "not condition.";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(tokens, vec![Token::Not, Token::Descriptor("condition"), Token::EndRule]);
}

#[test]
fn dot_vs_endrule2() {
    let input = "not condition.test";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(tokens, vec![Token::Not, Token::Descriptor("condition"), Token::Dot, Token::Descriptor("test")]);
}

#[test]
fn dot_vs_endrule3() {
    let input = "not condition. ";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(tokens, vec![Token::Not, Token::Descriptor("condition"), Token::EndRule]);
}

#[test]
fn dot_vs_endrule4() {
    let input = "not condition.{";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(tokens, vec![Token::Not, Token::Descriptor("condition"), Token::Dot, Token::LeftCBracket]);
}

