use crate::utils::lex_tokens;
use chumsky::Parser;
use cl0_parser::{
    ast::{
        Action, ActionList, AtomicCondition, Compound, Directive, FactRule, PrimitiveCondition,
        PrimitiveEvent, Rule,
    },
    parser::directive_parser,
};

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: Directive) {
    let tokens = lex_tokens(src);
    let parsed = directive_parser().parse(tokens.as_slice());
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
    let parsed = directive_parser().parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn create_valid_directive_scale() {
    println!("Testing valid directive scale creation...");
    assert_parses_to(
        "@scale(4){f.}",
        Directive::Scale {
            number: 4,
            policy: Compound {
                rules: vec![Rule::Fact(FactRule {
                    condition: AtomicCondition::Primitive(PrimitiveCondition::Var(
                        "f".to_string(),
                    )),
                })],
                alias: None,
            },
        },
    );
    println!("Parsed valid directive scale successfully.");
}

#[test]
fn create_valid_directive_scale_fail() {
    assert_fails("@scale(test){f.}");
}
