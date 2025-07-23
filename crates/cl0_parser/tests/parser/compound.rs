use crate::utils::lex_tokens;
use cl0_parser::{
    ast::{Action, Compound, PrimitiveCondition, PrimitiveEvent, ReactiveRule, Rule},
    parser::compound_parser,
};
use chumsky::Parser;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: Compound) {
    let tokens = lex_tokens(src);
    let parsed = compound_parser().parse(tokens.as_slice());
    assert!(
        !parsed.has_errors(),
        "expected success on {:?}, got errors: {:#?}",
        src,
        parsed.errors().collect::<Vec<_>>()
    );
    let (got, _span) = parsed.output().cloned().expect("parser returned no output");
    assert_eq!(got, want);
}

/// Assert that `parser` fails (i.e. leaves leftover/unconsumed or unexpected tokens).
fn assert_fails(src: &str) {
    let tokens = lex_tokens(src);
    let parsed = compound_parser().parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn create_valid_compound() {
    assert_parses_to(
        "{ #event => +a. }",
        Compound {
            rules: vec![Rule::Reactive(ReactiveRule::ECA {
                event: PrimitiveEvent::Trigger("event".to_string()),
                condition: None,
                action: Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            })],
            alias: None,
        },
    );
}

#[test]
fn create_valid_compound_with_alias() {
    assert_parses_to(
        "{ #event => +a. } as alias",
        Compound {
            rules: vec![Rule::Reactive(ReactiveRule::ECA {
                event: PrimitiveEvent::Trigger("event".to_string()),
                condition: None,
                action: Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            })],
            alias: Some("alias".to_string()),
        },
    );
}

#[test]
fn empty_fail() {
    assert_fails("");
}
