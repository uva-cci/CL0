use crate::utils::lex_tokens;
use CL0::{
    ast::{Action, ActionList, PrimitiveCondition, PrimitiveEvent},
    parser::action_parser,
};
use chumsky::Parser;

/// Assert that `parser` succeeds on `src` and returns exactly `want`.
fn assert_parses_to(src: &str, want: Action<'_>) {
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
            "primitive",
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
        Action::List(ActionList::Sequence(vec![Action::Primitive(
            PrimitiveEvent::Production(PrimitiveCondition::Var("a"))),
            Action::Primitive(PrimitiveEvent::Trigger("b")),
        ])),
    );
}

// #[test]
// fn create_valid_list_action_seq() {
//     assert_parses_to(
//         "+a; #b; -c",
//         Action::List(Action::Primitive(PrimitiveEvent::Production(PrimitiveCondition::Var(
//             "primitive",
//         ))),
//     );
// }

// #[test]
// fn empty_fail() {
//     assert_fails("");
// }
