use crate::utils::lex_tokens;
use chumsky::Parser;
use cl0_parser::{
    ast::{
        Action, ActionList, AtomicCondition, CaseRule, Condition, DeclarativeRule, FactRule, PrimitiveCondition, PrimitiveEvent, ReactiveRule, Rule
    },
    parser::rule_parser,
};

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: Rule) {
    let tokens = lex_tokens(src);
    let parsed = rule_parser().parse(tokens.as_slice());
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
    let parsed = rule_parser().parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn create_valid_eca_rule() {
    assert_parses_to(
        "#e: c => +a.",
        Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: Some(Condition::Atomic(AtomicCondition::Primitive(
                PrimitiveCondition::Var("c".to_string()),
            ))),
            action: Action::Primitive(PrimitiveEvent::Production(AtomicCondition::Primitive(
                PrimitiveCondition::Var("a".to_string()),
            ))),
        }),
    );
}

#[test]
fn create_bad_eca_rule1() {
    assert_fails("e: c => +a."); // e not a valid event
}

#[test]
fn create_bad_eca_rule2() {
    assert_fails("#e: c => a."); // a not a valid action
}

#[test]
fn create_bad_eca_rule3() {
    assert_fails("#e: +c => +a."); // c not a valid condition
}

#[test]
fn create_bad_eca_rule4() {
    assert_fails("#e: +c => +a"); // no period at the end
}

#[test]
fn create_valid_eca_rule_no_condition() {
    assert_parses_to(
        "#e => +a .",
        Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: None,
            action: Action::Primitive(PrimitiveEvent::Production(AtomicCondition::Primitive(
                PrimitiveCondition::Var("a".to_string()),
            ))),
        }),
    );
}

#[test]
fn create_valid_ca_rule() {
    assert_parses_to(
        ": c => +a.",
        Rule::Reactive(ReactiveRule::CA {
            condition: Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
                "c".to_string(),
            ))),
            action: Action::Primitive(PrimitiveEvent::Production(AtomicCondition::Primitive(
                PrimitiveCondition::Var("a".to_string()),
            ))),
        }),
    );
}

#[test]
fn create_bad_ca_rule1() {
    assert_fails(":+c => +a."); // c not a valid condition
}

#[test]
fn create_bad_ca_rule2() {
    assert_fails(":c => a."); // a not a valid action
}

#[test]
fn create_valid_cc_rule1() {
    assert_parses_to(
        "-> c.",
        Rule::Declarative(DeclarativeRule::CC {
            premise: None,
            condition: AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string())),
        }),
    );
}

#[test]
fn create_valid_cc_rule2() {
    assert_parses_to(
        "p -> c.",
        Rule::Declarative(DeclarativeRule::CC {
            premise: Some(Condition::Atomic(AtomicCondition::Primitive(
                PrimitiveCondition::Var("p".to_string()),
            ))),
            condition: AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string())),
        }),
    );
}

#[test]
fn create_valid_ct_rule1() {
    assert_parses_to(
        "-o c.",
        Rule::Declarative(DeclarativeRule::CT {
            premise: None,
            condition: Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
                "c".to_string(),
            ))),
        }),
    );
}

#[test]
fn create_valid_ct_rule2() {
    assert_parses_to(
        "p -o c.",
        Rule::Declarative(DeclarativeRule::CT {
            premise: Some(Condition::Atomic(AtomicCondition::Primitive(
                PrimitiveCondition::Var("p".to_string()),
            ))),
            condition: Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
                "c".to_string(),
            ))),
        }),
    );
}

#[test]
fn create_valid_case_rule() {
    assert_parses_to(
        "=> #a; #b.",
        Rule::Case(CaseRule {
            action: Action::List(ActionList::Sequence(vec![
                Action::Primitive(PrimitiveEvent::Trigger("a".to_string())),
                Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
            ])),
        }),
    );
}

#[test]
fn create_valid_fact_rule() {
    assert_parses_to(
        "c.",
        Rule::Fact(FactRule {
            condition: AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string())),
        }),
    );
}

#[test]
fn empty_fail() {
    assert_fails("");
}
