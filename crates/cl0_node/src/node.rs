use async_recursion::async_recursion;
use cl0_parser::ast::{AtomicCondition, Condition, ReactiveRule, Rule};
use tracing_subscriber::fmt::init;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::sync::{Arc, Weak};
use tokio::sync::{Mutex, broadcast, mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::api::ApiRoute;
use crate::event_handler::EventHandler;
use crate::logger::setup_node_logger;
use crate::utils::VarValue;

#[derive(Debug)]
pub struct Node {
    // State shared across the node
    vars: Arc<Mutex<HashMap<AtomicCondition, VarValue>>>,
    // Event handlers
    pub event_handlers: Arc<Mutex<HashMap<String, Arc<EventHandler>>>>,

    // The typed API routes:
    pub api: NodeApi,
}

#[derive(Debug)]
pub struct NodeApi {
    /// `(vec![Rule_1, Rule_2, ...]) -> vec![Success_bool_1, Success_bool_2, ...]`
    pub new_rules: ApiRoute<Vec<Rule>, Vec<bool>>,

    /// `() -> vec![Rule_1, Rule_2, ...]`
    pub get_rules: ApiRoute<(), Vec<ReactiveRule>>,
}

impl Node {
    /// Creates a new Node instance with the option of passing initial rules.
    pub fn new(rules: Option<Vec<Rule>>) -> Arc<Self> {
        Arc::new_cyclic(|weak_node: &Weak<Node>| {
            // Shared state
            let vars = Arc::new(Mutex::new(HashMap::new()));
            // Event handlers
            let event_handlers = Arc::new(Mutex::new(HashMap::<String, Arc<EventHandler>>::new()));

            let weak_node = weak_node.clone(); // now owned Weak<Node>
            let handlers_for_new = event_handlers.clone(); // Arc<Mutex<…>>

            // API Routes
            // `new_rules` is a route that accepts a Vec<Rule> and returns a Vec<bool>
            // indicating whether each rule was successfully processed.
            let new_rules = ApiRoute::new(move |rules: Vec<Rule>| {
                // Clone per‐call so each async task has its own Arc/Weak
                let weak_node = weak_node.clone();
                let handlers = handlers_for_new.clone();

                async move {
                    let node = weak_node
                        .upgrade()
                        .expect("Node got dropped while handling new_rules");

                    let mut results = Vec::with_capacity(rules.len());
                    let mut guard = handlers.lock().await;

                    for rule in rules.into_iter() {
                        // Only process declarative rules (currently)
                        match &rule {
                            Rule::Reactive(rr) => {
                                // Get the handler ID from the rule
                                let handler_id = rr.get_identifier();
                                // Track the success of processing each rule
                                let mut result = true;
                                
                                // get-or-create the handler
                                match guard.entry(handler_id.clone()) {
                                    Entry::Vacant(entry) => {
                                        // Handler does not exist, so create it
                                        debug!("Creating new handler: {}", handler_id);
                                        let new_handler = Arc::new(EventHandler::new(
                                            node.clone(),
                                            rr.clone(),
                                        ));
                                        entry.insert(new_handler);
                                    }
                                    Entry::Occupied(mut entry) => {
                                        // Key exists, do extra processing
                                        let handler = entry.get_mut();
                                        debug!("Handler already exists: {}", handler.id);
                                        result = handler.api.new_rule.call(rr.clone()).await?;
                                    }
                                }
                                results.push(result);
                            },
                            _ => {
                                results.push(false);
                                continue;
                            }
                        };
                    }
                    Ok(results)
                }
            });

            // `get_rules` is a route that returns all rules as a Vec<ReactiveRule>
            let handlers_for_get = event_handlers.clone();
            let get_rules = ApiRoute::new(move |(): ()| {
                let handlers = handlers_for_get.clone();
                async move {
                    let mut rules: Vec<ReactiveRule> = Vec::new();
                    for handler in handlers.lock().await.values() {
                        debug!("Handler ID: {}", handler.id);
                        let mut handler_rules = handler.api.get_rules.call(()).await?;
                        rules.append(&mut handler_rules);
                    }
                    Ok(rules)
                }
            });

            // Handle the initial rules if provided
            if let Some(initial_rules) = rules {
                new_rules.notify(initial_rules);
            }

            // Pass the new Node
            Node {
                vars,
                event_handlers,
                api: NodeApi {
                    new_rules,
                    get_rules,
                },
            }
        })
    }

    #[async_recursion]
    pub async fn process_condition(&self, condition: &Condition) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // Process the condition and return a boolean result
        match condition {
            Condition::Atomic(val) => {
                let mut vars = self.vars.lock().await;
                match vars.entry(val.clone()) {
                    Entry::Vacant(_) => {
                        return Err(Box::<dyn Error + Send + Sync>::from("Condition variable does not exist"));
                    }
                    Entry::Occupied(entry) => {
                        let value = entry.get();
                        value.to_bool() // Convert VarValue to bool
                    }
                }
            }
            Condition::Not(cond) => {
                let result = self.process_condition(cond).await?;
                Ok(!result) // Negate the result
            }
            Condition::Parentheses(cond) => {
                let result = self.process_condition(cond).await?; // Process the inner condition
                Ok(result)
            }
            Condition::Conjunction(conds) => {
                // Process conjunction (AND) conditions
                for cond in conds {
                    if !self.process_condition(cond).await? {
                        return Ok(false); // If any condition is false, return false
                    }
                }
                Ok(true) // All conditions are true
            }
            Condition::Disjunction(conds) => {
                // Process disjunction (OR) conditions
                for cond in conds {
                    if self.process_condition(cond).await? {
                        return Ok(true); // If any condition is true, return true
                    }
                }
                Ok(false) // All conditions are false
            }
        }
    }

    pub async fn update_var(&self, var: AtomicCondition, value: VarValue) -> Result<(), Box<dyn Error>> {
        if value == VarValue::Unknown {
            return Err(Box::<dyn Error>::from("Cannot update variable to Unknown"));
        }
        let mut vars = self.vars.lock().await;
        vars.insert(var, value);
        Ok(())
    }
}
