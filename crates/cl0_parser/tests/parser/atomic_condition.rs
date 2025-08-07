use chumsky::Parser;
use cl0_parser::{
    ast::{AtomicCondition, Compound, FactRule, PrimitiveCondition, Rule},
    parser::atomic_condition_parser,
};

use crate::utils::lex_tokens;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: AtomicCondition) {
    let tokens = lex_tokens(src);
    let parsed = atomic_condition_parser().parse(tokens.as_slice());
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
    let parsed = atomic_condition_parser().parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn atomic_var() {
    assert_parses_to(
        "foo",
        AtomicCondition::Primitive(PrimitiveCondition::Var("foo".to_string())),
    );
}

#[test]
fn atomic_var_fail() {
    assert_fails("foo foo");
}

#[test]
fn atomic_var_compound_with_alias() {
    assert_parses_to(
        "{ test. } as alias",
        AtomicCondition::Compound(Compound {
            rules: vec![Rule::Fact(FactRule {
                condition: AtomicCondition::Primitive(PrimitiveCondition::Var("test".to_string())),
            })],
            alias: Some("alias".to_string()),
        }),
    );
}

#[test]
fn atomic_var_sub_compound() {
    assert_parses_to(
        "r.{ test. }",
        AtomicCondition::SubCompound {
            namespace: "r".to_string(),
            condition: Box::new(AtomicCondition::Compound(Compound {
                rules: vec![Rule::Fact(FactRule {
                    condition: AtomicCondition::Primitive(PrimitiveCondition::Var(
                        "test".to_string(),
                    )),
                })],
                alias: None,
            })),
        },
    );
}

#[test]
fn atomic_var_sub_sub_compound() {
    assert_parses_to(
        "r.s.{ test. } as a",
        AtomicCondition::SubCompound {
            namespace: "r".to_string(),
            condition: Box::new(AtomicCondition::SubCompound {
                namespace: "s".to_string(),
                condition: Box::new(AtomicCondition::Compound(Compound {
                    rules: vec![Rule::Fact(FactRule {
                        condition: AtomicCondition::Primitive(PrimitiveCondition::Var(
                            "test".to_string(),
                        )),
                    })],
                    alias: Some("a".to_string()),
                })),
            }),
        },
    );
}
