use chumsky::Parser;
use CL0::lexer::lexer;
use CL0::token::Token;

#[test]
fn test_basic_lexer() {
    let input = "#click => a; b % comment here #";
    let tokens = lexer().parse(input).unwrap();
    let tokens: Vec<_> = tokens.into_iter().map(|(tok, _)| tok).collect();

    assert_eq!(
        tokens,
        vec![
            Token::Hash,
            Token::Identifier("click"),
            Token::FatArrow,
            Token::Identifier("a"),
            Token::Semicolon,
            Token::Identifier("b"),
        ]
    );
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
