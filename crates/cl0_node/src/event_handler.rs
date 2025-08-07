use cl0_parser::ast::{CaseRule, ReactiveRule};
use dashmap::DashSet;
use std::{collections::HashSet, sync::Arc};
use tracing::{debug, error, info, instrument, warn};

use crate::{api::ApiRoute, node::Node, utils::{ReactiveRuleWithArgs, RuleWithArgs, VarValue}};

/// Public-facing API for an event handler. Allows adding new rules, triggering processing,
/// and querying the active rule set.
#[derive(Debug)]
pub struct EventHandlerApi {
    /// Add or update a reactive rule in this handler.
    pub new_rule: ApiRoute<ReactiveRuleWithArgs, bool>,
    /// Trigger evaluation of all currently held rules and perform their side effects.
    pub process_action: ApiRoute<(), bool>,
    /// Enumerate the current reactive rules this handler is tracking.
    pub get_rules: ApiRoute<(), Vec<ReactiveRuleWithArgs>>,
}

/// A single handler responsible for a group of reactive rules bound by identifier.
#[derive(Debug)]
pub struct EventHandler {
    pub id: String,
    rules: Arc<DashSet<ReactiveRuleWithArgs>>,
    pub api: EventHandlerApi,
}

impl EventHandler {
    /// Constructs a new handler seeded with one initial reactive rule.
    pub fn new(node: Arc<Node>, rule_with_args: ReactiveRuleWithArgs) -> Self {
        // Each rule carries a status (unknown/true/false) that can be aggregated.
        let rules: Arc<DashSet<ReactiveRuleWithArgs>> = Arc::new(DashSet::new());
        let id = rule_with_args.rule.get_identifier().clone();

        // Insert the initial rule with an unknown status
        rules.insert(rule_with_args);

        // Route for inserting/updating a rule.
        let nr_rules = rules.clone();
        let nr_node = node.clone();
        let new_rule_route = ApiRoute::new(move |rule_with_args: ReactiveRuleWithArgs| {
            let rules = nr_rules.clone();
            let node = nr_node.clone();
            async move {
                {
                    rules.insert(rule_with_args.clone());
                }
                let ok = Self::process_rule_internal(node, rule_with_args.rule.clone()).await;
                Ok(ok)
            }
        });

        // Route to evaluate all rules and apply their effects
        let pa_rules = rules.clone();
        let pa_node = node.clone();
        let process_action_route = ApiRoute::new(move |()| {
            let rules = pa_rules.clone();
            let node = pa_node.clone();
            async move {
                let mut valid = true;
                for rule_ref in rules.iter() {
                    let rule_with_args= rule_ref.key();
                    let result = Self::process_rule_internal(node.clone(), rule_with_args.rule.clone()).await;
                    valid &= result;
                }
                Ok(valid)
            }
        });

        // Route to expose what rules are present
        let gr_rules = rules.clone();
        let get_rules_route = ApiRoute::new(move |(): ()| {
            let rules = gr_rules.clone();
            async move {
                Ok(rules
                    .iter()
                    .map(|entry| entry.key().clone())
                    .collect::<Vec<ReactiveRuleWithArgs>>())
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
    #[instrument(skip(self, rule_with_args))]
    pub async fn add_rule(
        &self,
        rule_with_args: ReactiveRuleWithArgs,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let ok = self.api.new_rule.call(rule_with_args).await?;
        Ok(ok)
    }

    /// Core rule evaluation logic: checks condition, and if true, emits the corresponding action.
    #[instrument(skip(node, rule))]
    async fn process_rule_internal(node: Arc<Node>, rule: ReactiveRule) -> bool {
        // Decompose the rule into optional condition and action
        let (condition, action) = match rule {
            ReactiveRule::CA { condition, action } => (Some(condition), action),
            ReactiveRule::ECA {
                event: _,
                condition,
                action,
            } => (condition, action),
        };

        // Evaluate condition if provided
        let node_clone = node.clone();
        let condition_result = match condition {
            Some(c) => node_clone.process_condition(&c).await,
            None => Ok(true),
        };

        match condition_result {
            Err(e) => {
                error!("Failed to process condition: {}", e);
                false
            }
            Ok(result) => {
                if result {
                    // Condition satisfied: fire the action as a new case rule
                    debug!("Condition is true, processing action: {:?}", action);
                    let case = RuleWithArgs::Case(CaseRule {
                        action: action.clone(),
                    });
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
        let mut statuses: HashSet<VarValue> = HashSet::new();
        for rule in self.rules.iter() {
            statuses.insert(rule.value.clone());
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
