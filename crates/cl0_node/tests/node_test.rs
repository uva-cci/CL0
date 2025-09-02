use cl0_node::node::Node;
use cl0_node::types::{ReactiveRuleWithArgs, RuleWithArgs, ActivationStatus};
use cl0_parser::ast::{Action, Compound, PrimitiveEvent, ReactiveRule, Rule};
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
    let node_rules = node.api.get_rules.call(false).await.unwrap();

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
    let node_rules = node.api.get_rules.call(false).await.unwrap();

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
                    ActivationStatus::True,
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
    let node_rules = node.api.get_rules.call(false).await.unwrap();

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
                    ActivationStatus::True,
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
    let node_rules = node.api.get_rules.call(false).await.unwrap();

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
                    ActivationStatus::True,
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
    let node_rules = node.api.get_rules.call(false).await.unwrap();

    assert_eq!(node_rules.len(), 2);
}

/// Test that a condition production can be processed by the node.
#[tokio::test]
async fn process_condition_check1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
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

    // Get the condition to check
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

    // Get the condition to check
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

    // Get the condition to check
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

    // Get the condition to check
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

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_err());
}

/// Test that a complex atomic condition can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +{#f => +loaded.}. => #e.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Test that a complex atomic condition can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition2() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +{#f => +loaded.}. => #e. => #f.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("{=> +loaded.} as r. => +r.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias2() {
    // Define new rules to init the node with
    let rules = lex_and_parse("{=> +loaded.} as r1. {=> -loaded.} as r2. => +r1. => +r2.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias3_1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e => +action.} as r. => #e.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "action".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias3_2() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e => +action.} as r. => -r. => #e.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "action".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias_err1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("{=> +loaded.} as r.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias4() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +{#f => +loaded.} as r. => #e. => -r. => #f.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(!res);
}

/// Test that a complex atomic condition with aliasing can be processed by the node.
#[tokio::test]
async fn process_complex_atomic_condition_alias5() {
    // Define new rules to init the node with
    let rules = lex_and_parse("#e => +{#f => +loaded.} as r. => #e. => +r. => #f.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Test a simple primitive condition.
#[tokio::test]
async fn test_fact_primitive() {
    // Define new rules to init the node with
    let rules = lex_and_parse("loaded.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "loaded".to_string(),
    )));

    let res = node.process_condition(&condition).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert!(res);
}

/// Test a simple compound condition.
#[tokio::test]
async fn test_fact_compound() {
    // Define new rules to init the node with
    let rules = lex_and_parse("{#e => #a.} as r.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the condition to check
    let condition = AtomicCondition::Compound(Compound {
        rules: vec![Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: None,
            action: Action::Primitive(PrimitiveEvent::Trigger("a".to_string())),
        })],
        alias: Some("r".to_string()),
    });

    let res = node.get_atomic_condition(condition, None).await;
    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::False);
}

#[tokio::test]
async fn test_fact_sub_compound1() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e => #a1. #e => #a2.} as r.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create a new rule to remove the second event
    let more_rules = lex_and_parse("=> -r.{#e => #a2.}.")
        .into_iter()
        .filter_map(|r| {
            if let Rule::Case(cr) = r {
                Some(RuleWithArgs::Case(cr))
            } else {
                None
            }
        })
        .collect();

    // Get the condition to check
    let condition1 = AtomicCondition::Compound(Compound {
        rules: vec![Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: None,
            action: Action::Primitive(PrimitiveEvent::Trigger("a1".to_string())),
        })],
        alias: Some("r".to_string()),
    });

    // Get the condition to check
    let condition2 = AtomicCondition::Compound(Compound {
        rules: vec![Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: None,
            action: Action::Primitive(PrimitiveEvent::Trigger("a2".to_string())),
        })],
        alias: Some("r".to_string()),
    });

    let res = node
        .clone()
        .get_atomic_condition(condition1.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);

    let res = node
        .clone()
        .get_atomic_condition(condition2.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);

    node.api.new_rules.notify(more_rules);

    let res = node
        .clone()
        .get_atomic_condition(condition1.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);

    let res = node
        .clone()
        .get_atomic_condition(condition2.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::False);
}

#[tokio::test]
async fn test_fact_sub_compound2() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e => #a1. #e => #a2.} as r.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create a new rule to remove the second event
    let more_rules = lex_and_parse("=> -{#e => #a2.}.")
        .into_iter()
        .filter_map(|r| {
            if let Rule::Case(cr) = r {
                Some(RuleWithArgs::Case(cr))
            } else {
                None
            }
        })
        .collect();

    // Get the condition to check
    let condition1 = AtomicCondition::Compound(Compound {
        rules: vec![Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: None,
            action: Action::Primitive(PrimitiveEvent::Trigger("a1".to_string())),
        })],
        alias: Some("r".to_string()),
    });

    // Get the condition to check
    let condition2 = AtomicCondition::Compound(Compound {
        rules: vec![Rule::Reactive(ReactiveRule::ECA {
            event: PrimitiveEvent::Trigger("e".to_string()),
            condition: None,
            action: Action::Primitive(PrimitiveEvent::Trigger("a2".to_string())),
        })],
        alias: Some("r".to_string()),
    });

    let res = node
        .clone()
        .get_atomic_condition(condition1.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);

    let res = node
        .clone()
        .get_atomic_condition(condition2.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);

    node.api.new_rules.notify(more_rules);

    let res = node
        .clone()
        .get_atomic_condition(condition1.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);

    let res = node
        .clone()
        .get_atomic_condition(condition2.clone(), None)
        .await;
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, ActivationStatus::True);
}


#[tokio::test]
async fn test_action_list_sequence_simple() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e1=>+a1.}; +{#e2=>+a2.}; +{#e3=>+a3.}.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the current rules
    let node_rules = node.api.get_rules.call(false).await.unwrap();

    assert_eq!(node_rules.len(), 3);
}

#[tokio::test]
async fn test_action_list_parallel_simple() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e1=>+a1.}, +{#e2=>+a2.}, +{#e3=>+a3.}.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the current rules
    let node_rules = node.api.get_rules.call(false).await.unwrap();

    assert_eq!(node_rules.len(), 3);
}

#[tokio::test]
async fn test_action_list_parallel() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e1: not (a2 or a3) => +a1.}. => +{#e2: not (a1 or a3) => +a2.}. => +{#e3: not (a1 or a2) => +a3.}.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Create new rules to add to the node
    let more_rules = lex_and_parse("=> #e1, #e2, #e3.")
        .into_iter()
        .filter_map(|r| {
            if let Rule::Case(cr) = r {
                Some(RuleWithArgs::Case(cr))
            } else {
                None
            }
        })
        .collect();

    // Add the new rules
    let r = node.api.new_rules.call(more_rules).await;
    println!("New rules added: {:?}", r);

    // Get the condition to check
    let condition1 = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "a1".to_string(),
    )));
    let condition2 = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "a2".to_string(),
    )));
    let condition3 = Condition::Atomic(AtomicCondition::Primitive(PrimitiveCondition::Var(
        "a3".to_string(),
    )));

    let res = node.clone().process_condition(&condition1).await;
    assert!(res.is_ok());
    let res1 = res.unwrap();


    let res = node.clone().process_condition(&condition2).await;
    assert!(res.is_ok());
    let res2 = res.unwrap();


    let res = node.clone().process_condition(&condition3).await;
    assert!(res.is_ok());
    let res3 = res.unwrap();

    assert!(res1 || res2 || res3, "At least one action should be true");
}

#[tokio::test]
async fn test_action_list_alternate_simple() {
    // Define new rules to init the node with
    let rules = lex_and_parse("=> +{#e1=>+a1.} alt +{#e2=>+a2.} alt +{#e3=>+a3.}.");

    // Create new node with the rules
    let node = Node::new_with_rules(Some(rules)).await;

    // Get the current rules
    let node_rules = node.api.get_rules.call(false).await.unwrap();

    assert_eq!(node_rules.len(), 1);
}
