
use std::sync::Arc;
use cl0_parser::ast::{AtomicCondition, PrimitiveEvent, ReactiveRule, Rule};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};


use crate::{api::ApiRoute, node::{Node}, utils::VarValue};

#[derive(Debug)]
pub struct EventHandler {
    pub id: String,
    rules: Arc<Mutex<Vec<ReactiveRule>>>,
    node: Arc<Node>,

    pub api: EventHandlerApi,

}

// I will most likely need to consolidate to another type that will take in both `ReactiveRule` and `DeclarativeRule`. This type will can then allow
// the user to see if a `ReactiveRule` is in fact a `DeclarativeRule` or not. Maybe something like this?

// pub enum TransformedReactiveRule {
//     NativeReactive(ReactiveRule),
//     Declarative {
//         original: DeclarativeRule,
//         transformed: ReactiveRule,
//     }
// }
#[derive(Debug)]
pub struct EventHandlerApi {
    /// `(rule) -> success bool`
    pub new_rule: ApiRoute<ReactiveRule, bool>,

    /// `(action) -> success bool`
    pub process_action: ApiRoute<PrimitiveEvent, bool>,

    /// `() -> vec![Rule_1, Rule_2, ...]`
    pub get_rules: ApiRoute<(), Vec<ReactiveRule>>,
}

impl EventHandler {
    pub fn new(node: Arc<Node>, rule: ReactiveRule) -> Self {
        // Rules
        let rules = Arc::new(Mutex::new(vec![rule.clone()]));

        // Get the identifier from the rule
        let id = rule.get_identifier().clone();

        // API Routes
        // `new_rule` is a route that accepts a ReactiveRule and returns a bool indicating whether each rule was successfully processed.
        let nr_rules = rules.clone();
        let nr_node = node.clone();
        let new_rule_route = ApiRoute::new(move |rule: ReactiveRule| {
            let rules = nr_rules.clone();
            let node = nr_node.clone();
            async move {
                {
                    let mut guard = rules.lock().await;
                    guard.push(rule.clone());
                }
                let ok = Self::process_rule_internal(node,rule.clone()).await;
                Ok(ok)
            }
        });

        // `process_action` is a route that accepts an Action and returns a bool indicating success
        let pa_rules = rules.clone();
        let pa_node = node.clone();
        let process_action_route = ApiRoute::new(move |action: PrimitiveEvent| {
            let rules = pa_rules.clone();
            let node = pa_node.clone();
            async move {
                let ok = match action {
                    PrimitiveEvent::Trigger(_) => {
                        // Get status of all rules
                        let mut valid = true;
                        // Process the trigger event
                        for rule in rules.lock().await.iter() {
                            let node = node.clone();
                            let result = Self::process_rule_internal(node, rule.clone()).await;
                            valid &= result;
                        }
                        valid
                    }
                    PrimitiveEvent::Production(event) => {
                        // Process the production event
                        let atomic_condition = AtomicCondition::Primitive(event);
                        let _ = node.update_var(atomic_condition, VarValue::True).await;
                        true
                    }
                    PrimitiveEvent::Consumption(event) => {
                        // Process the consumption event
                        let atomic_condition = AtomicCondition::Primitive(event);
                        let _ = node.update_var(atomic_condition, VarValue::False).await;
                        true
                    }
                };
                Ok(ok)
            }
        }); 

        // `get_rules` is a route that returns all rules as a Vec<ReactiveRule>
        let gr_rules = rules.clone();
        let get_rules_route = ApiRoute::new(move |(): ()| {
            let rules = gr_rules.clone();
            async move {
                let guard = rules.lock().await;
                Ok(guard.clone())
            }
        });

        // 5) Construct the handler
        EventHandler {
            id,
            rules,
            node,
            api: EventHandlerApi {
                new_rule:  new_rule_route,
                process_action: process_action_route,
                get_rules: get_rules_route,
            },
        }
    }

    /// Your actual async rule‐processing logic
    async fn process_rule_internal(
        node: Arc<Node>,
        rule: ReactiveRule,
    ) -> bool {
        // Check if there are any conditions to process
        let (condition, action) = match rule {
            ReactiveRule::CA { condition, action } => (Some(condition), action),
            ReactiveRule::ECA { event: _, condition, action } => (condition, action),
        };

        // Check if the condition was provided
        let condition_result = match condition {
            Some(c) => node.process_condition(&c).await,
            None => Ok(true),
        };

        // Process the condition and action
        match condition_result {
            Err(e) => {
                error!("Failed to process condition: {}", e);
                false
            }
            Ok(result) => {
                if result {
                    // If the condition is true, process the action
                    // Create a new rule (case) for the action
                    debug!("Condition is true, processing action: {:?}", action);
                    let case: Rule = Rule::Case { action: action.clone() };
                    match node.api.new_rules.call(vec![case]).await {
                        Ok(r) => {
                            let r_val = r[0];
                            if r_val {
                                // Successfully created a new rule (case)
                                info!("Successfully created new rule (case) for action: {:?}", action);
                            }
                            else {
                                // Failed to create a new rule (case)
                                warn!("Failed to create new rule (case) for action: {:?}", action);
                            }
                            r_val
                        }
                        Err(e) => {
                            error!("Failed to create new rule (case) for action: {:?}, error: {:?}", action, e);
                            false
                        }
                    }
                } else {
                    // Condition is false, no action needed
                    debug!("Condition is false, no action processed for: {:?}", action);
                    true
                }
            }
        }
    }
}