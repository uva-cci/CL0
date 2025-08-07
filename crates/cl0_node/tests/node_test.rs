use cl0_node::node::Node;
use cl0_node::utils::{ReactiveRuleWithArgs, RuleWithArgs, VarValue};
use cl0_parser::ast::{Rule};
use cl0_parser::{
    ast::{AtomicCondition, Condition, PrimitiveCondition},
    lex_and_parse,
};

/// Test that a node can be initialized without any rules.
#[tokio::test]
async fn node_init() {
    // Create an empty node
    let node = Node::new_with_rules(None).await;

    // Get the current set of rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 0);
}

/// Test that a node can be initialized with a set of rules.
#[tokio::test]
async fn node_init_with_rule() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the current rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 1);
}
/// Test that a node with no rules can have new rules added.
#[tokio::test]
async fn node_init_with_rules_added() {
    // Create new node with the rules
    let node = Node::new_with_rules(None).await;

    // Create new rules to add to the node
    let more_rules = lex_and_parse("#e => +a.")
        .into_iter()
        .filter_map(|r| {
            if let Rule::Reactive(rr) = r {
                Some(RuleWithArgs::Reactive(ReactiveRuleWithArgs::new(
                    rr,
                    VarValue::True,
                    None,
                )))
            } else {
                None
            }
        })
        .collect();

    // Add the new rules
    node.api.new_rules.notify(more_rules);

    // Get the current rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 1);
}

/// Test that a node can be initialized with a set of rules and then have new rules added.
#[tokio::test]
async fn rule_added() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create new rules to add to the node
    let more_rules: Vec<RuleWithArgs> = lex_and_parse("#f => +a.")
        .into_iter()
        .filter(|r| matches!(r, Rule::Reactive(_)))
        .filter_map(|r| {
            if let Rule::Reactive(rr) = r {
                Some(RuleWithArgs::Reactive(ReactiveRuleWithArgs::new(
                    rr,
                    VarValue::True,
                    None,
                )))
            } else {
                None
            }
        })
        .collect();

    // Add the new rules
    node.api.new_rules.notify(more_rules);

    // Get the current rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 2);
}

/// Test that a node can be initialized with a set of rules and then have new rules added with the same name.
#[tokio::test]
async fn rule_added_same_name() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create new rules to add to the node
    let more_rules = lex_and_parse("#e => +b.")
        .into_iter()
        .filter_map(|r| {
            if let Rule::Reactive(rr) = r {
                Some(RuleWithArgs::Reactive(ReactiveRuleWithArgs::new(
                    rr,
                    VarValue::True,
                    None,
                )))
            } else {
                None
            }
        })
        .collect();

    // Add the new rules
    node.api.new_rules.notify(more_rules);

    // Get the current rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 2);
}

/// Test that a condition production can be processed by the node.
#[tokio::test]
async fn process_condition_check1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Check to see what happens if a condition that does not exist.
#[tokio::test]
async fn process_condition_check_error1() {
    // Loaded never set
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_err());
}

/// Test that a condition that is consumed can be processed by the node, but it returns false.
#[tokio::test]
async fn process_condition_check2() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> -loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Test that an action can be processed by the node.
#[tokio::test]
async fn process_action_check1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +loaded. => #e.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Check to see if an event contains a condition, but the condition is not yet set.
#[tokio::test]
async fn process_action_check2() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Check to see what happens if a condition that does not exist.
#[tokio::test]
async fn process_action_check_error1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +not_loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_err());
}
