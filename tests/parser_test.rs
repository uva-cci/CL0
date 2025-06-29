use chumsky::Parser;
use CL0::lexer::lexer;
use CL0::parser::action_parser;
use CL0::ast::Action;

fn lex(input: &str) -> Vec<(CL0::token::Token, chumsky::span::SimpleSpan)> {
    lexer().parse(input).unwrap()
}

// Action parser tests
#[test]
fn test_atomic_trigger() {
    let tokens = lex("#foo");
    let parser = action_parser();
    let result = parser.parse(tokens);
    assert!(result.is_ok());
    let (action, _) = result.unwrap();
    assert_eq!(action, Action::Trigger("foo"));
}

#[test]
fn test_parallel_actions() {
    let tokens = lex("+a, +b");
    let parser = action_parser();
    let result = parser.parse(tokens);
    assert!(result.is_ok());
    let (action, _) = result.unwrap();
    assert_eq!(
        action,
        Action::Parallel(vec![
            Action::Production("a"),
            Action::Production("b"),
        ])
    );
}

#[test]
fn test_sequence_actions() {
    let tokens = lex("-x; -y");
    let parser = action_parser();
    let result = parser.parse(tokens);
    assert!(result.is_ok());
    let (action, _) = result.unwrap();
    assert_eq!(
        action,
        Action::Sequence(vec![
            Action::Consumption("x"),
            Action::Consumption("y"),
        ])
    );
}