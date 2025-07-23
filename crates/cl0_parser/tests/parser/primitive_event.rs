use chumsky::Parser;
use cl0_parser::{ast::{PrimitiveCondition, PrimitiveEvent}, parser::primitive_event_parser};
use crate::utils::lex_tokens;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(
    src: &str,
    want: PrimitiveEvent,
) {
    let tokens = lex_tokens(src);
    let parsed = primitive_event_parser()
        .parse(tokens.as_slice());
    assert!(
        !parsed.has_errors(),
        "expected success on {:?}, got errors: {:#?}",
        src,
        parsed.errors().collect::<Vec<_>>()
    );
    let (got, _span) = parsed
        .output()
        .cloned()
        .expect("parser returned no output");
    assert_eq!(got, want);
}

/// Assert that `parser` fails (i.e. leaves leftover/unconsumed or unexpected tokens).
fn assert_fails(src: &str) {
    let tokens = lex_tokens(src);
    let parsed = primitive_event_parser()
        .parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn create_valid_trigger() {
    assert_parses_to("#trigger", PrimitiveEvent::Trigger("trigger".to_string()));
}

#[test]
fn create_valid_trigger_fail() {
    assert_fails("#trigger #trigger");
}

#[test]
fn create_valid_production() {
    assert_parses_to("+produce", PrimitiveEvent::Production(PrimitiveCondition::Var("produce".to_string())));
}

#[test]
fn create_valid_production_fail() {
    assert_fails("+produce +produce");
}

#[test]
fn create_valid_consumption() {
    assert_parses_to("-consume", PrimitiveEvent::Consumption(PrimitiveCondition::Var("consume".to_string())));
}

#[test]
fn create_valid_consumption_production_fail() {
    assert_fails("-+consume");
}

#[test]
fn empty_fail() {
    assert_fails("");
}