use crate::utils::lex_tokens;
use cl0_parser::{
    ast::{AtomicCondition, Condition, PrimitiveCondition},
    parser::condition_parser,
};
use chumsky::Parser;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: Condition) {
    let tokens = lex_tokens(src);
    println!("{:#?}", tokens);
    let parsed = condition_parser().parse(tokens.as_slice());
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
    println!("{:#?}", tokens);
    let parsed = condition_parser().parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn create_valid_atomic_condition() {
    assert_parses_to(
        "condition",
        Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
            "condition".to_string(),
        ))),
    );
}

#[test]
fn create_valid_atomic_condition_fail() {
    assert_fails("condition condition");
}

#[test]
fn create_valid_not_condition() {
    assert_parses_to(
        "not condition",
        Condition::Not(Box::new(Condition::Atomic(AtomicCondition::Primitive(
            PrimitiveCondition::Var("condition".to_string()),
        )))),
    );
}

#[test]
fn create_valid_not_condition_fail() {
    assert_fails("not condition not condition");
}

#[test]
fn create_valid_not_not_condition() {
    assert_parses_to(
        "not not condition",
        Condition::Not(Box::new(Condition::Not(Box::new(Condition::Atomic(
            AtomicCondition::Primitive(PrimitiveCondition::Var("condition".to_string())),
        ))))),
    );
}

#[test]
fn create_valid_not_not_condition_fail() {
    assert_fails("not not condition condition");
}

#[test]
fn create_valid_parentheses_condition() {
    assert_parses_to(
        "(condition)",
        Condition::Parentheses(Box::new(Condition::Atomic(AtomicCondition::Primitive(
            PrimitiveCondition::Var("condition".to_string()),
        )))),
    );
}
#[test]
fn create_valid_parentheses_condition_fail() {
    assert_fails("(condition)(condition)");
}

#[test]
fn create_complex_valid_parentheses_condition() {
    assert_parses_to(
        "(not condition)",
        Condition::Parentheses(Box::new(Condition::Not(Box::new(Condition::Atomic(
            AtomicCondition::Primitive(PrimitiveCondition::Var("condition".to_string())),
        ))))),
    );
}

#[test]
fn create_valid_conjunction_condition() {
    assert_parses_to(
        "a , b",
        Condition::Conjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
        ]),
    );
}

#[test]
fn create_valid_conjunction_condition_keyword() {
    assert_parses_to(
        "a and b",
        Condition::Conjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
        ]),
    );
}

#[test]
fn create_valid_conjunction_condition_keyword_trailing() {
    assert_fails("a and b and");
}

#[test]
fn create_valid_conjunction_condition_keyword_leading() {
    assert_fails("and a and b");
}

#[test]
fn create_valid_disjunction_condition_keyword_trailing() {
    assert_fails("a or b or");
}

#[test]
fn create_valid_disjunction_condition_keyword_leading() {
    assert_fails("or a or b");
}

#[test]
fn create_valid_long_conjunction_condition() {
    assert_parses_to(
        "a , b, c , d,e",
        Condition::Conjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("d".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("e".to_string()))),
        ]),
    );
}

#[test]
fn create_valid_disjunction_condition() {
    assert_parses_to(
        "a ; b",
        Condition::Disjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
        ]),
    );
}

#[test]
fn create_valid_disjunction_condition_keyword() {
    assert_parses_to(
        "a or b",
        Condition::Disjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
        ]),
    );
}

#[test]
fn create_valid_long_disjunction_condition() {
    assert_parses_to(
        "a ; b; c ; d;e",
        Condition::Disjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("d".to_string()))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("e".to_string()))),
        ]),
    );
}

#[test]
fn create_valid_order_ops_conjunction_disjunction_condition() {
    assert_parses_to(
        "a , b ; c , d; e, f",
        Condition::Disjunction(vec![
            Condition::Conjunction(vec![
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
            ]),
            Condition::Conjunction(vec![
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string()))),
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("d".to_string()))),
            ]),
            Condition::Conjunction(vec![
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("e".to_string()))),
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("f".to_string()))),
            ]),
        ]),
    );
}

#[test]
fn create_valid_order_ops_parentheses_conjunction_disjunction_condition() {
    assert_parses_to(
        "(a ; b) , (c ; d), (e; f)",
        Condition::Conjunction(vec![
            Condition::Parentheses(Box::new(Condition::Disjunction(vec![
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
            ]))),
            Condition::Parentheses(Box::new(Condition::Disjunction(vec![
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("c".to_string()))),
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("d".to_string()))),
            ]))),
            Condition::Parentheses(Box::new(Condition::Disjunction(vec![
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("e".to_string()))),
                Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("f".to_string()))),
            ]))),
        ]),
    );
}

#[test]
fn create_not_with_parentheses_condition() {
    assert_parses_to(
        "not (condition)",
        Condition::Not(Box::new(Condition::Parentheses(Box::new(
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
                "condition".to_string(),
            ))),
        )))),
    );
}

#[test]
fn create_not_with_parentheses_complex_condition() {
    assert_parses_to(
        "not a and b",
        Condition::Conjunction(vec![
            Condition::Not(Box::new(Condition::Atomic(AtomicCondition::Primitive(
                PrimitiveCondition::Var("a".to_string()),
            )))),
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("b".to_string()))),
        ]),
    );
}

#[test]
fn create_not_with_parentheses_complex_condition2() {
    assert_parses_to(
        "a and not b",
        Condition::Conjunction(vec![
            Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("a".to_string()))),
            Condition::Not(Box::new(Condition::Atomic(AtomicCondition::Primitive(
                PrimitiveCondition::Var("b".to_string()),
            )))),
        ]),
    );
}

#[test]
fn empty_fail() {
    assert_fails("");
}
