use chumsky::Parser;
use cl0_parser::{ast::PrimitiveCondition, parser::primitive_condition_parser};

use crate::utils::lex_tokens;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(
    src: &str,
    want: PrimitiveCondition<'_>,
) {
    let tokens = lex_tokens(src);
    let parsed = primitive_condition_parser()
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
    let parsed = primitive_condition_parser()
        .parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn atomic_var() {
    assert_parses_to("foo", PrimitiveCondition::Var("foo"));
}

#[test]
fn atomic_var_fail() {
    assert_fails("foo foo");
}