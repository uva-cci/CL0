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

// #[test]
// fn test_ignores_comments_and_whitespace() {
//     let input = "    a, b % line comment
//     alt x y   ";
//     let tokens = lexer().parse(input).unwrap();
//     let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

//     assert_eq!(
//         tokens,
//         vec![
//             Token::Par,
//             Token::Identifier("a"),
//             Token::Identifier("b"),
//             Token::Alt,
//             Token::Identifier("x"),
//             Token::Identifier("y"),
//         ]
//     );
// }
