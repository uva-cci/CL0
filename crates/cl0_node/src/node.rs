use async_recursion::async_recursion;
use cl0_parser::ast::{
    Action, ActionList, AtomicCondition, Condition, PrimitiveEvent, ReactiveRule, Rule,
};
use rand::seq::IndexedRandom;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::error::Error;
use std::sync::{Arc, Weak};
use tokio::sync::{Mutex, Notify};
use tracing::{debug, error, info, instrument, warn};

use crate::api::ApiRoute;
use crate::event_handler::EventHandler;
use crate::utils::{VarValue, collect_conjunction};

/// Public API surface for a Node: adding/getting rules.
#[derive(Debug)]
pub struct NodeApi {
    pub new_rules: ApiRoute<Vec<Rule>, Vec<bool>>,
    pub get_rules: ApiRoute<(), Vec<ReactiveRule>>,
}

/// Core node that maintains variable state, aliases, and event handlers.
#[derive(Debug)]
pub struct Node {
    pub vars: Arc<Mutex<HashMap<AtomicCondition, VarValue>>>,
    pub aliases: Arc<Mutex<HashMap<String, Vec<Rule>>>>,
    pub event_handlers: Arc<Mutex<HashMap<String, Arc<EventHandler>>>>,
    pub api: NodeApi,
}

impl Node {
    /// Async constructor that builds the node and applies initial rules if provided.
    ///
    /// This avoids races where rules would be "not reliably" applied if fired without awaiting.
    pub async fn new_with_rules(rules: Option<Vec<Rule>>) -> Arc<Self> {
        // Use `Arc::new_cyclic` to get a self-referential structure safely.
        let node = Arc::new_cyclic(|weak_node: &Weak<Node>| {
            // Shared internal state
            let vars = Arc::new(Mutex::new(HashMap::new()));
            let aliases = Arc::new(Mutex::new(HashMap::new()));
            let event_handlers = Arc::new(Mutex::new(HashMap::new()));

            // Cloneable handles for closure capture
            let weak_node = weak_node.clone();
            let handlers_for_new = event_handlers.clone();
            let handlers_for_get = event_handlers.clone();

            // Route to add new rules; handles both reactive rules and immediate cases.
            let new_rules = ApiRoute::new(move |rules: Vec<Rule>| {
                let weak_node = weak_node.clone();
                let handlers = handlers_for_new.clone();
                async move {
                    let node = weak_node
                        .upgrade()
                        .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Node dropped"))?;

                    let mut results = Vec::with_capacity(rules.len());
                    let mut guard = handlers.lock().await;

                    for rule in rules.into_iter() {
                        match &rule {
                            Rule::Reactive(rr) => {
                                let handler_id = rr.get_identifier();
                                let mut result = true;

                                match guard.entry(handler_id.clone()) {
                                    Entry::Vacant(entry) => {
                                        debug!("Creating new handler: {}", handler_id);
                                        let new_handler =
                                            Arc::new(EventHandler::new(node.clone(), rr.clone()));
                                        entry.insert(new_handler);
                                    }
                                    Entry::Occupied(entry) => {
                                        // Clone out the handler so we can drop the lock before awaiting.
                                        let handler = entry.get().clone();
                                        // Drop guard to avoid holding the node-wide lock across await.
                                        drop(guard);
                                        result = handler.add_rule(rr.clone()).await?;
                                        // Reacquire for the next iteration.
                                        guard = handlers.lock().await;
                                    }
                                }
                                results.push(result);
                            }
                            Rule::Case { action } => {
                                let res = node.clone().process_action(action.clone()).await;
                                match res {
                                    Ok(r) => results.push(r),
                                    Err(e) => {
                                        error!("Failed to process action: {}", e);
                                        results.push(false);
                                    }
                                }
                            }
                            _ => {
                                results.push(false);
                            }
                        }
                    }

                    Ok(results)
                }
            });

            // Route to gather all reactive rules by querying each handler.
            let get_rules = ApiRoute::new(move |(): ()| {
                let handlers = handlers_for_get.clone();
                async move {
                    let mut rules_accum: Vec<ReactiveRule> = Vec::new();
                    let guard = handlers.lock().await;
                    for handler in guard.values() {
                        debug!("Collecting rules from handler: {}", handler.id);
                        let mut handler_rules = handler.api.get_rules.call(()).await?;
                        rules_accum.append(&mut handler_rules);
                    }
                    Ok(rules_accum)
                }
            });

            Node {
                vars,
                aliases,
                event_handlers,
                api: NodeApi {
                    new_rules,
                    get_rules,
                },
            }
        });

        // Apply initial rules in a controlled (awaited) fashion.
        if let Some(initial_rules) = rules {
            debug!("Initializing Node with rules: {:?}", initial_rules);
            if let Err(e) = node.api.new_rules.call(initial_rules).await {
                error!("Failed to apply initial rules: {:?}", e);
            }
        } else {
            debug!("Initializing Node without initial rules");
        }

        node
    }

    /// Recursively evaluates complex conditions. Instrumented for tracing.
    #[instrument(skip(self, condition))]
    #[async_recursion]
    pub async fn process_condition(
        &self,
        condition: &Condition,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match condition {
            Condition::Atomic(val) => {
                let mut vars = self.vars.lock().await;
                match vars.entry(val.clone()) {
                    Entry::Vacant(_) => Err(Box::<dyn Error + Send + Sync>::from(format!(
                        "Condition variable does not exist: {:?}",
                        val
                    ))),
                    Entry::Occupied(entry) => {
                        let value = entry.get();
                        value.to_bool().map_err(|e| {
                            Box::<dyn Error + Send + Sync>::from(format!(
                                "Failed to evaluate condition {:?}: {}",
                                val, e
                            ))
                        })
                    }
                }
            }
            Condition::Not(cond) => {
                let result = self.process_condition(cond).await?;
                Ok(!result)
            }
            Condition::Parentheses(cond) => self.process_condition(cond).await,
            Condition::Conjunction(conds) => {
                for cond in conds {
                    if !self.process_condition(cond).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Disjunction(conds) => {
                for cond in conds {
                    if self.process_condition(cond).await? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    /// Entry point for processing an action. Handles triggers, productions, and consumptions.
    #[instrument(skip(self, action))]
    #[async_recursion]
    pub async fn process_action(
        self: Arc<Self>,
        action: Action,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        match action {
            Action::Primitive(prim_event) => match prim_event {
                PrimitiveEvent::Trigger(desc) => {
                    let mut handlers_guard = self.event_handlers.lock().await;
                    match handlers_guard.entry(desc.clone()) {
                        Entry::Vacant(_) => {
                            error!("Invalid action cannot be executed: {}", desc);
                            Err(Box::<dyn Error + Send + Sync>::from(format!(
                                "Invalid action: {}",
                                desc
                            )))
                        }
                        Entry::Occupied(eh_entry) => {
                            let handler = eh_entry.get().clone();
                            drop(handlers_guard); // minimize lock scope

                            match handler.state().await {
                                VarValue::True => {
                                    debug!("Processing action for event handler: {}", desc);
                                    let action_res = handler.api.process_action.call(()).await;
                                    action_res.map_err(|e| e)
                                }
                                VarValue::False => {
                                    info!("Inactive variable was silently not executed: {}", desc);
                                    Ok(true)
                                }
                                VarValue::Unknown => Ok(true),
                            }
                        }
                    }
                }
                PrimitiveEvent::Production(ac) => self.update_var(ac, VarValue::True).await,
                PrimitiveEvent::Consumption(ac) => self.update_var(ac, VarValue::False).await,
            },
            Action::List(list) => {
                match list {
                    ActionList::Sequence(actions) => {
                        // Sequential-start execution: fire each sub-action one after another without waiting for completion, but still collect their results.
                        let mut handles = Vec::with_capacity(actions.len());
                        for sub in actions {
                            let node_clone = Arc::clone(&self);
                            let handle = tokio::spawn(async move {
                                node_clone.process_action(sub.clone()).await
                            });
                            handles.push(handle);
                        }
                        collect_conjunction(handles).await
                    }
                    ActionList::Parallel(actions) => {
                        // Parallel execution: launch all sub-actions concurrently and await all their results.
                        let start_notify = Arc::new(Notify::new());
                        let mut handles = Vec::with_capacity(actions.len());

                        for sub in actions {
                            let node_clone = Arc::clone(&self);
                            let gate = start_notify.clone();
                            let handle = tokio::spawn(async move {
                                gate.notified().await; // wait until everyone is ready
                                node_clone.process_action(sub.clone()).await
                            });
                            handles.push(handle);
                        }
                        // release actions all simultaneously
                        start_notify.notify_waiters();
                        collect_conjunction(handles).await
                    }
                    ActionList::Alternative(actions) => {
                        // Alternative execution: launch one random action from the list.
                        if actions.is_empty() {
                            return Err(Box::<dyn std::error::Error + Send + Sync>::from(
                                "Cannot execute empty alternative action",
                            ));
                        }

                        // Get a random action.
                        let selected_action = {
                            let mut rng = rand::rng(); // current thread-local RNG
                            actions
                                .choose(&mut rng)
                                .expect("non-empty; just checked")
                                .clone()
                        };

                        debug!("Executing alternative action: {:?}", selected_action);

                        self.clone().process_action(selected_action).await
                    }
                }
            }
        }
    }

    /// Updates an atomic condition's value atomically.
    #[instrument(skip(self, var, value))]
    pub async fn update_var(
        &self,
        var: AtomicCondition,
        value: VarValue,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if value == VarValue::Unknown {
            return Err(Box::<dyn Error + Send + Sync>::from(
                "Cannot update variable to Unknown",
            ));
        }

        let mut vars = self.vars.lock().await;
        vars.insert(var, value);
        Ok(true)
    }
}
