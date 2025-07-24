use cl0_node::node::Node;
use cl0_parser::lex_and_parse;


#[tokio::test]
async fn rule_added() {
    let rules = lex_and_parse("#e => +a.");

    let node = Node::new(Some(rules));

    let node_rules = node.api.get_rules.call(()).await.unwrap();
    assert_eq!(node_rules.len(), 1);
}