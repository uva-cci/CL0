use cl0_node::node::Node;
use cl0_parser::{ast::{AtomicCondition, Condition, PrimitiveCondition}, lex_and_parse};


#[tokio::test]
async fn node_init() {
    // Create an empty node
    let node = Node::new_with_rules(None).await;

    // Get the current set of rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();
    
    assert_eq!(node_rules.len(), 0);
}

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

#[tokio::test]
async fn rule_added() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create new rules to add to the node
    let more_rules = lex_and_parse("#f => +a.");

    // Add the new rules
    node.api.new_rules.notify(more_rules);

    // Get the current rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 2);
}

#[tokio::test]
async fn rule_added_same_name() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create new rules to add to the node
    let more_rules = lex_and_parse("#e => +b.");

    // Add the new rules
    node.api.new_rules.notify(more_rules);

    // Get the current rules
    let node_rules = node.api.get_rules.call(()).await.unwrap();

    assert_eq!(node_rules.len(), 2);
}

#[tokio::test]
async fn process_condition_check1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("loaded".to_string())));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

#[tokio::test]
async fn process_condition_check_error1() { // Loaded never set
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +a.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("loaded".to_string())));

    let res = node.process_condition(&condition).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn process_condition_check2() {
     // Define new rules to init the node with
    let rules = lex_and_parse("=> -loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var("loaded".to_string())));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}