use cl0_parser::ast::{ReactiveRule, Rule};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

use crate::{api::ApiRoute, node::Node, utils::VarValue};

/// Public-facing API for an event handler. Allows adding new rules, triggering processing,
/// and querying the active rule set.
#[derive(Debug)]
pub struct EventHandlerApi {
    /// Add or update a reactive rule in this handler.
    pub new_rule: ApiRoute<ReactiveRule, bool>,
    /// Trigger evaluation of all currently held rules and perform their side effects.
    pub process_action: ApiRoute<(), bool>,
    /// Enumerate the current reactive rules this handler is tracking.
    pub get_rules: ApiRoute<(), Vec<ReactiveRule>>,
}

/// A single handler responsible for a group of reactive rules bound by identifier.
#[derive(Debug)]
pub struct EventHandler {
    pub id: String,
    rules: Arc<Mutex<Vec<(ReactiveRule, VarValue)>>>,
    pub api: EventHandlerApi,
}

impl EventHandler {
    /// Constructs a new handler seeded with one initial reactive rule.
    pub fn new(node: Arc<Node>, rule: ReactiveRule) -> Self {
        // Each rule carries a status (unknown/true/false) that can be aggregated.
        let rules = Arc::new(Mutex::new(vec![(rule.clone(), VarValue::Unknown)]));
        let id = rule.get_identifier().clone();

        // Route for inserting/updating a rule.
        let nr_rules = rules.clone();
        let nr_node = node.clone();
        let new_rule_route = ApiRoute::new(move |rule: ReactiveRule| {
            let rules = nr_rules.clone();
            let node = nr_node.clone();
            async move {
                {
                    let mut guard = rules.lock().await;
                    guard.push((rule.clone(), VarValue::Unknown));
                }
                let ok = Self::process_rule_internal(node, rule.clone()).await;
                Ok(ok)
            }
        });

        // Route to evaluate all rules and apply their effects.
        let pa_rules = rules.clone();
        let pa_node = node.clone();
        let process_action_route = ApiRoute::new(move |()| {
            let rules = pa_rules.clone();
            let node = pa_node.clone();
            async move {
                let mut valid = true;
                let guard = rules.lock().await;
                for (rule, _) in guard.iter() {
                    let result = Self::process_rule_internal(node.clone(), rule.clone()).await;
                    valid &= result;
                }
                Ok(valid)
            }
        });

        // Route to expose what rules are present.
        let gr_rules = rules.clone();
        let get_rules_route = ApiRoute::new(move |(): ()| {
            let rules = gr_rules.clone();
            async move {
                let guard = rules.lock().await;
                Ok(guard
                    .clone()
                    .into_iter()
                    .map(|(rule, _)| rule)
                    .collect::<Vec<ReactiveRule>>())
            }
        });

        EventHandler {
            id,
            rules,
            api: EventHandlerApi {
                new_rule: new_rule_route,
                process_action: process_action_route,
                get_rules: get_rules_route,
            },
        }
    }

    /// Convenience wrapper so callers do not have to know to `.call(...)` on the internal route.
    #[instrument(skip(self, rule))]
    pub async fn add_rule(
        &self,
        rule: ReactiveRule,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let ok = self.api.new_rule.call(rule).await?;
        Ok(ok)
    }

    /// Core rule evaluation logic: checks condition, and if true, emits the corresponding action.
    #[instrument(skip(node, rule))]
    async fn process_rule_internal(node: Arc<Node>, rule: ReactiveRule) -> bool {
        // Decompose the rule into optional condition and action.
        let (condition, action) = match rule {
            ReactiveRule::CA { condition, action } => (Some(condition), action),
            ReactiveRule::ECA {
                event: _,
                condition,
                action,
            } => (condition, action),
        };

        // Evaluate condition if provided.
        let condition_result = match condition {
            Some(c) => node.process_condition(&c).await,
            None => Ok(true),
        };

        match condition_result {
            Err(e) => {
                error!("Failed to process condition: {}", e);
                false
            }
            Ok(result) => {
                if result {
                    // Condition satisfied: fire the action as a new case rule.
                    debug!("Condition is true, processing action: {:?}", action);
                    let case: Rule = Rule::Case {
                        action: action.clone(),
                    };
                    match node.api.new_rules.call(vec![case]).await {
                        Ok(r) => {
                            let r_val = r.get(0).cloned().unwrap_or(false);
                            if r_val {
                                info!(
                                    "Successfully created new rule (case) for action: {:?}",
                                    action
                                );
                            } else {
                                warn!("Failed to create new rule (case) for action: {:?}", action);
                            }
                            r_val
                        }
                        Err(e) => {
                            error!(
                                "Failed to create new rule (case) for action: {:?}, error: {:?}",
                                action, e
                            );
                            false
                        }
                    }
                } else {
                    debug!("Condition is false, no action processed for: {:?}", action);
                    true
                }
            }
        }
    }

    /// Aggregate the statuses of all contained rules into a single effective state.
    pub async fn state(&self) -> VarValue {
        let mut statuses = HashSet::new();
        for rule in self.rules.lock().await.iter() {
            statuses.insert(rule.1.clone());
        }
        if statuses.len() == 1 {
            statuses.into_iter().next().unwrap_or(VarValue::Unknown)
        } else if statuses.contains(&VarValue::True) {
            VarValue::True
        } else if statuses.contains(&VarValue::False) {
            VarValue::False
        } else {
            VarValue::Unknown
        }
    }
}
