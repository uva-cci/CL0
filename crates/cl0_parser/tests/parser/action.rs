use crate::utils::lex_tokens;
use cl0_parser::{
    ast::{Action, ActionList, PrimitiveCondition, PrimitiveEvent},
    parser::action_parser,
};
use chumsky::Parser;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: Action) {
    let tokens = lex_tokens(src);
    let parsed = action_parser().parse(tokens.as_slice());
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
    let parsed = action_parser().parse(tokens.as_slice());
    assert!(
        parsed.has_errors(),
        "expected parse to fail on {:?}, but it succeeded with value {:?}",
        src,
        parsed.output()
    );
}

#[test]
fn create_valid_primitive_action() {
    println!("Testing valid primitive action creation...");
    assert_parses_to(
        "+primitive",
        Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var(
            "primitive".to_string(),
        ))),
    );
    println!("Parsed valid primitive action successfully.");
}

#[test]
fn create_valid_primitive_action_fail() {
    assert_fails("+primitive +primitive");
}
#[test]
fn simple_create_valid_list_action_seq() {
    assert_parses_to(
        "+a; #b",
        Action::List(ActionList::Sequence(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
        ])),
    );
}

#[test]
fn simple_create_valid_list_action_seq_trailing() {
    assert_parses_to(
        "+a; #b;",
        Action::List(ActionList::Sequence(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
        ])),
    );
}

#[test]
fn simple_create_valid_list_action_seq_keyword() {
    assert_parses_to(
        "+a seq #b",
        Action::List(ActionList::Sequence(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
        ])),
    );
}

#[test]
fn create_valid_list_action_seq() {
    assert_parses_to(
        "+a; #b;-c;-d;  #e; +f",
        Action::List(ActionList::Sequence(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
            Action::Primitive(PrimitiveEvent::Consumption(PrimitiveCondition::Var("c".to_string()))),
            Action::Primitive(PrimitiveEvent::Consumption(PrimitiveCondition::Var("d".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("e".to_string())),
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("f".to_string()))),
        ])),
    );
}

#[test]
fn simple_create_valid_list_action_par() {
    assert_parses_to(
        "+a, #b",
        Action::List(ActionList::Parallel(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
        ])),
    );
}

#[test]
fn simple_create_valid_list_action_par_keyword() {
    assert_parses_to(
        "+a par #b",
        Action::List(ActionList::Parallel(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
        ])),
    );
}

#[test]
fn create_valid_list_action_par() {
    assert_parses_to(
        "+a, #b,-c,-d,  #e, +f",
        Action::List(ActionList::Parallel(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
            Action::Primitive(PrimitiveEvent::Consumption(PrimitiveCondition::Var("c".to_string()))),
            Action::Primitive(PrimitiveEvent::Consumption(PrimitiveCondition::Var("d".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("e".to_string())),
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("f".to_string()))),
        ])),
    );
}

#[test]
fn simple_create_valid_list_action_alt() {
    assert_parses_to(
        "+a alt #b",
        Action::List(ActionList::Alternative(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
        ])),
    );
}

#[test]
fn create_valid_list_action_alt() {
    assert_parses_to(
        "+a alt #b alt -c alt -d alt  #e alt +f",
        Action::List(ActionList::Alternative(vec![
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("a".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
            Action::Primitive(PrimitiveEvent::Consumption(PrimitiveCondition::Var("c".to_string()))),
            Action::Primitive(PrimitiveEvent::Consumption(PrimitiveCondition::Var("d".to_string()))),
            Action::Primitive(PrimitiveEvent::Trigger("e".to_string())),
            Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var("f".to_string()))),
        ])),
    );
}

#[test]
fn create_valid_list_action_order_of_ops() {
    assert_parses_to(
        "#a par #b alt #c par #d seq #e par #f alt #g par #h",
        Action::List(ActionList::Sequence(vec![
            Action::List(ActionList::Alternative(vec![
                Action::List(ActionList::Parallel(vec![
                    Action::Primitive(PrimitiveEvent::Trigger("a".to_string())),
                    Action::Primitive(PrimitiveEvent::Trigger("b".to_string())),
                ])),
                Action::List(ActionList::Parallel(vec![
                    Action::Primitive(PrimitiveEvent::Trigger("c".to_string())),
                    Action::Primitive(PrimitiveEvent::Trigger("d".to_string())),
                ])),
            ])),
            Action::List(ActionList::Alternative(vec![
                Action::List(ActionList::Parallel(vec![
                    Action::Primitive(PrimitiveEvent::Trigger("e".to_string())),
                    Action::Primitive(PrimitiveEvent::Trigger("f".to_string())),
                ])),
                Action::List(ActionList::Parallel(vec![
                    Action::Primitive(PrimitiveEvent::Trigger("g".to_string())),
                    Action::Primitive(PrimitiveEvent::Trigger("h".to_string())),
                ])),
            ])),
        ])),
    );
}

#[test]
fn empty_fail() {
    assert_fails("");
}
