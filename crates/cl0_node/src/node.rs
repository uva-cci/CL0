use async_recursion::async_recursion;
use cl0_parser::ast::{
    Action, ActionList, AtomicCondition, CaseRule, Compound, Condition, FactRule,
    PrimitiveCondition, PrimitiveEvent, Rule,
};
use dashmap::{DashMap, DashSet};
use rand::seq::IndexedRandom;
use std::error::Error;
use std::sync::{Arc, Weak};
use std::vec;
use tokio::sync::Barrier;
use tracing::{debug, error, info, instrument, warn};
// use tracing_subscriber::field::debug;

use crate::api::ApiRoute;
use crate::event_handler::EventHandler;
use crate::types::{ActivationStatus, FactRuleWithArgs, ReactiveRuleWithArgs, RuleWithArgs};
use crate::utils::{
    AliasNamespace,
    collect_conjunction, get_parts, overall_status_from_set,
};
use crate::visitor::AstVisitor;

/// Public API surface for a Node: adding/getting rules.
#[derive(Debug)]
pub struct NodeApi {
    pub new_rules: ApiRoute<Vec<RuleWithArgs>, Vec<bool>>,
    pub get_rules: ApiRoute<bool, Vec<ReactiveRuleWithArgs>>,
}

/// Core node that maintains variable state, aliases, and event handlers.
#[derive(Debug)]
pub struct Node {
    pub vars: Arc<DashMap<PrimitiveCondition, ActivationStatus>>,
    pub aliases: Arc<DashMap<String, Arc<AliasNamespace>>>,
    pub event_handlers: Arc<DashMap<String, Arc<EventHandler>>>,
    pub api: NodeApi,
}

impl Node {
    /// Async constructor that builds the node and applies initial rules if provided.
    pub async fn new_with_rules(rules: Option<Vec<Rule>>) -> Arc<Self> {
        // Use `Arc::new_cyclic` to get a self-referential structure safely
        let node = Arc::new_cyclic(|weak_node: &Weak<Node>| {
            // Shared internal state
            let vars: Arc<DashMap<PrimitiveCondition, ActivationStatus>> = Arc::new(DashMap::new());
            let aliases: Arc<DashMap<String, Arc<AliasNamespace>>> = Arc::new(DashMap::new());
            let event_handlers: Arc<DashMap<String, Arc<EventHandler>>> = Arc::new(DashMap::new());

            // Cloneable handles for closure capture
            let weak_node = weak_node.clone();
            let handlers_for_get = event_handlers.clone();

            // Route to add new rules; handles both reactive rules and immediate cases
            let new_rules = ApiRoute::new(move |rules: Vec<RuleWithArgs>| {
                let weak_node = weak_node.clone();
                async move {
                    let node = weak_node
                        .upgrade()
                        .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Node dropped"))?;

                    let results = Vec::with_capacity(rules.len());

                    for rule in rules.into_iter() {
                        Self::process_rule(node.clone(), rule).await?;
                    }

                    Ok(results)
                }
            });

            // Route to gather all reactive rules by querying each handler
            let get_rules = ApiRoute::new(move |all: bool| {
                let handlers = handlers_for_get.clone();
                async move {
                    let mut rules_accum: Vec<ReactiveRuleWithArgs> = Vec::new();
                    for handler_ref in handlers.iter() {
                        let id = handler_ref.key();
                        debug!("Collecting rules from handler: {}", id);
                        let handler = handler_ref.value().clone(); // Arc<EventHandler>
                        let mut handler_rules = handler.api.get_rules.call(all).await?;
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

        // Apply initial rules in a controlled (awaited) fashion
        if let Some(initial_rules) = rules {
            // Initialize all potential atomic conditions
            let mut atomic_conditions = Vec::new();
            for rule in initial_rules.clone() {
                rule.visit(&mut |c| {
                    if let Some(ac) = c.downcast_ref::<AtomicCondition>() {
                        atomic_conditions.push(ac.clone());
                    }
                });
            }
            for ac in atomic_conditions {
                // Store each atomic condition with an initial value of False
                let _ = Self::store_atomic_condition(node.clone(), ac, ActivationStatus::False, None, true)
                    .await;
            }

            // Execute all case rules at the end of initialization
            debug!("Initializing Node with rules: {:?}", initial_rules.clone());
            let other_rules: Vec<RuleWithArgs> = initial_rules
                .clone()
                .into_iter()
                .filter_map(|rule| match rule {
                    Rule::Reactive(r) => Some(RuleWithArgs::Reactive(ReactiveRuleWithArgs {
                        rule: r,
                        value: ActivationStatus::True,
                        alias: None,
                    })),
                    Rule::Fact(FactRule { condition }) => {
                        Some(RuleWithArgs::Fact(FactRuleWithArgs {
                            rule: FactRule { condition },
                            value: None,
                        }))
                    }
                    Rule::Declarative(d) => Some(RuleWithArgs::Declarative(d)),
                    _ => None,
                })
                .collect();
            let case_rules: Vec<RuleWithArgs> = initial_rules
                .clone()
                .into_iter()
                .filter(|rule| matches!(rule, Rule::Case { .. }))
                .filter_map(|rule| match rule {
                    Rule::Case(CaseRule { action }) => {
                        Some(RuleWithArgs::Case(CaseRule { action }))
                    }
                    _ => None,
                })
                .collect();

            // Apply non-case rules first
            debug!("Applying initial rules: {:?}", other_rules);
            let res = node.api.new_rules.call(other_rules).await;
            if let Err(e) = res {
                error!("Failed to apply initial rules: {:?}", e);
            } else {
                debug!("Initial rules applied successfully");
                // Apply case rules next
                debug!("Applying case rules: {:?}", case_rules);
                if let Err(e) = node.api.new_rules.call(case_rules).await {
                    error!("Failed to apply case rules: {:?}", e);
                } else {
                    debug!("Case rules applied successfully");
                }
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
        self: Arc<Self>,
        condition: &Condition,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let cond = condition.clone().to_string();
        debug!("Processing condition: {}", cond);
        match condition {
            Condition::Atomic(val) => {
                let ac = self.get_atomic_condition(val.clone(), None).await;
                match ac {
                    Ok(value) => value.to_bool().map_err(|e| {
                        Box::<dyn Error + Send + Sync>::from(format!(
                            "Failed to evaluate condition {:?}: {}",
                            val, e
                        ))
                    }),
                    Err(e) => {
                        error!("Failed to get atomic condition: {}", e);
                        Err(Box::<dyn Error + Send + Sync>::from(format!(
                            "Failed to get atomic condition {:?}: {}",
                            val, e
                        )))
                    }
                }
            }
            Condition::Not(cond) => {
                let result = self.process_condition(cond).await?;
                Ok(!result)
            }
            Condition::Parentheses(cond) => self.process_condition(cond).await,
            Condition::Conjunction(conds) => {
                let node_clone = Arc::clone(&self);
                for cond in conds {
                    let node_clone = node_clone.clone();
                    if !node_clone.process_condition(cond).await? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            Condition::Disjunction(conds) => {
                let node_clone = Arc::clone(&self);
                for cond in conds {
                    let node_clone = node_clone.clone();
                    if node_clone.process_condition(cond).await? {
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
        // Log the action being processed
        let a = action.clone().to_string();
        debug!("Processing action: {}", a);

        // Match on the action type to determine how to process it
        match action {
            Action::Primitive(prim_event) => match prim_event {
                PrimitiveEvent::Trigger(desc) => match self.event_handlers.get(&desc) {
                    None => {
                        error!("Invalid action cannot be executed: {}", desc);
                        Err(Box::<dyn Error + Send + Sync>::from(format!(
                            "Invalid action: {}",
                            desc
                        )))
                    }
                    Some(eh_entry) => {
                        let handler = eh_entry.value();
                        match handler.state().await {
                            ActivationStatus::True => {
                                debug!("Processing action for event handler: {}", desc);
                                let action_res = handler.api.process_action.call(()).await;
                                action_res.map_err(|e| e)
                            }
                            ActivationStatus::False => {
                                info!("Inactive variable was silently not executed: {}", desc);
                                Ok(true)
                            }
                            ActivationStatus::Conflict => Ok(true),
                        }
                    }
                },
                PrimitiveEvent::Production(ac) => {
                    let alias_rules = self.get_alias_rules(ac.clone(), None).await;
                    match alias_rules {
                        Err(_) => {
                            debug!("No alias found for atomic condition: {:?}", ac);
                            self.store_atomic_condition(ac, ActivationStatus::True, None, true)
                                .await
                        }
                        Ok((rules, ns)) => {
                            debug!("Found alias rules for atomic condition: {:?}", ac);
                            // store_atomic_condition should handle everything except for case rules
                            // Store the atomic condition as True after processing all rules
                            let mut r = self
                                .clone()
                                .store_atomic_condition(
                                    AtomicCondition::Compound(Compound {
                                        rules: rules.clone(),
                                        alias: None,
                                    }),
                                    ActivationStatus::True,
                                    Some(ns),
                                    true,
                                )
                                .await?;

                            // Extract case rules from the alias rules
                            let case_rules: Vec<RuleWithArgs> = rules
                                .iter()
                                .filter_map(|rule| match rule {
                                    Rule::Case(cr) => Some(RuleWithArgs::Case(cr.clone())),
                                    _ => None,
                                })
                                .collect();

                            // Process each case rule
                            for case_rule in case_rules {
                                debug!("Processing case rule: {:?}", case_rule);
                                match self.clone().process_rule(case_rule).await {
                                    Ok(res) => r &= res,
                                    Err(e) => {
                                        error!("Failed to process case rule: {}", e);
                                        return Err(e);
                                    }
                                }
                            }
                            Ok(r)
                        }
                    }
                }

                PrimitiveEvent::Consumption(ac) => {
                    let alias_rules = self.get_alias_rules(ac.clone(), None).await;
                    match alias_rules {
                        Err(_) => {
                            debug!("No alias found for atomic condition: {:?}", ac);
                            self.store_atomic_condition(ac, ActivationStatus::False, None, true)
                                .await
                        }
                        Ok((rules, ns)) => {
                            debug!("Found alias rules for atomic condition: {:?}", ac);
                            // store_atomic_condition should handle everything except for case rules
                            // Store the atomic condition as True after processing all rules
                            let mut r = self
                                .clone()
                                .store_atomic_condition(
                                    AtomicCondition::Compound(Compound {
                                        rules: rules.clone(),
                                        alias: None,
                                    }),
                                    ActivationStatus::False,
                                    Some(ns),
                                    false,
                                )
                                .await?;

                            // Extract case rules from the alias rules
                            let case_rules: Vec<RuleWithArgs> = rules
                                .iter()
                                .filter_map(|rule| match rule {
                                    Rule::Case(cr) => Some(RuleWithArgs::Case(cr.clone())),
                                    _ => None,
                                })
                                .collect();

                            // Process each case rule
                            for case_rule in case_rules {
                                debug!("Processing case rule: {:?}", case_rule);
                                match self.clone().process_rule(case_rule).await {
                                    Ok(res) => r &= res,
                                    Err(e) => {
                                        error!("Failed to process case rule: {}", e);
                                        return Err(e);
                                    }
                                }
                            }
                            Ok(r)
                        }
                    }
                }
            },
            Action::List(list) => {
                match list {
                    ActionList::Sequence(actions) => {
                        // Sequential-start execution: fire each sub-action one after another without waiting for completion, but still collect their results
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
                        // Parallel execution: launch all sub-actions concurrently and await all their results
                        let barrier = Arc::new(Barrier::new(actions.len() + 1));
                        let mut handles = Vec::with_capacity(actions.len());

                        for sub in actions {
                            let node_clone = Arc::clone(&self);
                            let b = barrier.clone();
                            let handle = tokio::spawn(async move {
                                b.wait().await;
                                node_clone.process_action(sub.clone()).await
                            });
                            handles.push(handle);
                        }
                        // release actions all simultaneously
                        barrier.wait().await;
                        collect_conjunction(handles).await
                    }
                    ActionList::Alternative(actions) => {
                        // Alternative execution: launch one random action from the list
                        if actions.is_empty() {
                            return Err(Box::<dyn std::error::Error + Send + Sync>::from(
                                "Cannot execute empty alternative action",
                            ));
                        }

                        // Get a random action
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

    /// Retrieves rules from the alias namespace based on the atomic condition and optional namespace.
    /// Will return an error if the alias is not found or if the condition is not a primitive variable.
    #[async_recursion]
    async fn get_alias_rules(
        &self,
        atomic_condition: AtomicCondition,
        alias_namespace: Option<Vec<String>>,
    ) -> Result<(Vec<Rule>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
        // Log the atomic condition being processed
        let ac_str = atomic_condition.clone().to_string();
        debug!(
            "Retrieving alias rules for atomic condition {} using namespace {:?}",
            ac_str, alias_namespace
        );

        // Meant to be used for retrieving rules from an alias namespace defined by an atomic condition.
        match atomic_condition {
            // The tail of the alias namespace
            AtomicCondition::Primitive(PrimitiveCondition::Var(var)) => {
                // Check if there is any alias namespace provided
                match alias_namespace {
                    // No alias namespace provided, therefore the variable is in the main alias
                    None => {
                        let an = match self.aliases.get(&var) {
                            // If the alias cannot be found, return an error
                            None => {
                                return Err(Box::<dyn std::error::Error + Send + Sync>::from(
                                    "No alias found",
                                ));
                            }
                            // Alias found, retrieve the rules
                            Some(alias_namespace_ref) => {
                                // Clone the AliasNamespace to avoid holding a lock
                                alias_namespace_ref.value().clone()
                            }
                        };
                        // Get the rules from the alias namespace
                        let rules = an.get_rules(vec![]).await?;
                        // Return the rules and the namespace as a single variable
                        let ns = vec![var.clone().to_string()];
                        Ok((rules, ns))
                    }
                    // An alias namespace is provided, therefore the variable is in a sub-alias
                    Some(alias_namespace_val) => {
                        // Combine the alias namespace with the variable name
                        let mut combined_namespace = alias_namespace_val.clone();
                        combined_namespace.push(var.clone().to_string());
                        let (head, tail, combined_namespace) = get_parts(combined_namespace);

                        // Get the alias namespace from the aliases map
                        let an = match self.aliases.get(&head) {
                            // If the alias cannot be found, return an error
                            None => {
                                return Err(Box::<dyn std::error::Error + Send + Sync>::from(
                                    "No alias found",
                                ));
                            }
                            // Alias found, retrieve the rules
                            Some(alias_namespace_ref) => {
                                // Clone the AliasNamespace to avoid holding a lock
                                alias_namespace_ref.value().clone()
                            }
                        };
                        // Get the rules from the alias namespace
                        let rules = an.get_rules(tail).await?;
                        Ok((rules, combined_namespace))
                    }
                }
            }
            // The atomic condition itself should be ignored for rules retrieval, just return the namespace and rules if an alias is provided from the sub compound.
            AtomicCondition::Compound(Compound { rules, .. }) => {
                match alias_namespace {
                    // No alias namespace provided, return an error telling that there are no alias rules
                    None => Err(Box::<dyn std::error::Error + Send + Sync>::from(
                        "No alias found",
                    )),
                    // Alias namespace provided, return the overlapping rules and the namespace
                    Some(ns) => {
                        let tail_namespace = ns.clone()[ns.len() - 1].clone();
                        let main_namespace = ns[..ns.len() - 1].to_vec();
                        let ns = if main_namespace.is_empty() {
                            None
                        } else {
                            Some(main_namespace)
                        };
                        // Convert to a primitive atomic condition
                        let ac =
                            AtomicCondition::Primitive(PrimitiveCondition::Var(tail_namespace));

                        // Get the rules from the main alias namespace
                        let (full_rules, full_namespace) = self.get_alias_rules(ac, ns).await?;

                        // Get the matching rules
                        let matching_rules: Vec<Rule> = rules
                            .into_iter()
                            .filter(|rule| full_rules.contains(rule))
                            .collect();

                        // Return the matching rules and the namespace
                        Ok((matching_rules, full_namespace))
                    }
                }
            }
            // Sub namespace defined by a sub compound condition
            AtomicCondition::SubCompound {
                namespace,
                condition,
            } => {
                // Append the namespace to the main alias namespace
                let mut next_ns = match alias_namespace {
                    Some(ns) => ns,
                    None => vec![],
                };
                next_ns.push(namespace);

                // Recursively get the rules from the sub namespace
                self.get_alias_rules(*condition, Some(next_ns)).await
            }
        }
    }

    /// Stores an atomic condition with a value.
    /// Condition - the atomic condition to store,
    /// value - the value to associate with the condition,
    /// var_namespace - optional namespace to store the condition in,
    /// override_entries - whether to override existing entries in the namespace.
    #[instrument(skip(self, condition, value, var_namespace))]
    #[async_recursion]
    pub async fn store_atomic_condition(
        self: Arc<Self>,
        condition: AtomicCondition,
        value: ActivationStatus,
        var_namespace: Option<Vec<String>>,
        override_entries: bool,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Log the condition being processed
        let cond = condition.clone().to_string();
        debug!("Storing atomic condition: {} with value: {:?}", cond, value);

        // Match on the type of atomic condition to determine how to handle it
        match condition {
            AtomicCondition::Primitive(prim_cond) => {
                // Handle primitive conditions
                self.update_var(prim_cond, value)
            }
            AtomicCondition::Compound(Compound { rules, alias }) => {
                // Handle compound conditions

                // Check the current namespace
                let mut n = match var_namespace.clone() {
                    Some(ns) => ns,
                    None => vec![],
                };

                // Append the alias if provided
                if let Some(alias_name) = alias.clone() {
                    n.push(alias_name.clone());
                }

                // Copy of the namespace for future use
                let var_namespace_copy = if n.len() > 0 { Some(n.clone()) } else { None };

                // If the namespace is empty, we are in the main namespace
                if n.is_empty() {
                    info!("Storing rules in the main namespace");
                } else {
                    info!("Storing rules in namespace: {:?}", n);
                    // Split the namespace into the first alias and the rest
                    let first_alias = n[0].clone();
                    n = n[1..].to_vec().clone(); // Remove the first alias from the namespace

                    // Get the first layer namespace or create it if it does not exist
                    let child_ns = if let Some(r) = self.aliases.get(&first_alias) {
                        r.value().clone()
                    } else {
                        // Create a new AliasNamespace if it does not exist
                        let new_ns = Arc::new(AliasNamespace::new());
                        self.aliases.insert(first_alias.clone(), new_ns.clone());
                        new_ns
                    };

                    // Create or update the rules in the namespace
                    let prev_rules = child_ns
                        .create_rules(n, rules.clone(), override_entries)
                        .await;

                    match prev_rules {
                        Ok(Some(existing)) => warn!("Overriding existing rules: {:?}", existing),
                        Ok(None) => info!("No existing rules, inserted fresh."),
                        Err(e) => {
                            error!("Failed to create or update rules: {}", e);
                            return Err(e);
                        }
                    }
                }

                // Convert the rules into RuleWithArgs format
                let rules_with_args: Vec<RuleWithArgs> = rules
                    .into_iter()
                    .filter_map(|r| match r {
                        Rule::Reactive(rr) => {
                            Some(RuleWithArgs::Reactive(ReactiveRuleWithArgs::new(
                                rr.clone(),
                                value.clone(),
                                var_namespace_copy.clone(),
                            )))
                        }
                        Rule::Fact(fact_rule) => Some(RuleWithArgs::Fact(FactRuleWithArgs {
                            rule: fact_rule.clone(),
                            value: Some(value.clone()),
                        })),
                        Rule::Case(_) => None, // Case rules are not processed here
                        _ => panic!("Unsupported rule type in compound condition: {:?}", r),
                    })
                    .collect();

                // Process the rules with arguments
                let mut r = true;
                for rule in rules_with_args.iter() {
                    match self.clone().process_rule(rule.clone()).await {
                        Ok(res) => r &= res,
                        Err(e) => {
                            error!("Failed to process rule: {}", e);
                            return Err(e);
                        }
                    }
                }
                Ok(r)
            }
            AtomicCondition::SubCompound {
                namespace,
                condition,
            } => {
                // Calculate the new namespace
                let n = match var_namespace {
                    Some(ns) => {
                        let mut new_ns = vec![namespace];
                        new_ns.append(&mut ns.clone());
                        new_ns
                    }
                    None => vec![namespace],
                };

                // Handle sub-compound conditions
                self.store_atomic_condition(*condition, ActivationStatus::False, Some(n), override_entries)
                    .await
            }
        }
    }

    /// Retrieves the value of an atomic condition.
    /// Condition - the atomic condition to store,
    /// value - the value to associate with the condition,
    /// var_namespace - optional namespace to store the condition in,
    /// override_entries - whether to override existing entries in the namespace.
    #[instrument(skip(self, condition, var_namespace))]
    #[async_recursion]
    pub async fn get_atomic_condition(
        self: Arc<Self>,
        condition: AtomicCondition,
        var_namespace: Option<Vec<String>>,
    ) -> Result<ActivationStatus, Box<dyn std::error::Error + Send + Sync>> {
        // Log the condition being processed
        let cond = condition.clone().to_string();
        debug!("Getting atomic condition: {}", cond);

        // Match on the type of atomic condition to determine how to handle it
        match condition {
            AtomicCondition::Primitive(prim_cond) => {
                let v_ref = self.vars.get(&prim_cond);
                match v_ref {
                    None => Ok(ActivationStatus::Conflict),
                    Some(entry) => Ok(entry.value().clone()),
                }
            }
            AtomicCondition::Compound(Compound { rules, alias }) => {
                // Get the corresponding rules from the alias
                // Check the current namespace
                let mut n = match var_namespace {
                    Some(ns) => ns,
                    None => vec![],
                };

                // Append the alias if provided
                if let Some(alias_name) = alias.clone() {
                    n.push(alias_name.clone());
                }

                // Copy of the namespace for future use
                let var_namespace_copy = if n.len() > 0 { Some(n.clone()) } else { None };

                // If the namespace is empty, we are in the main namespace
                if n.is_empty() {
                    info!("Rules are in the main namespace");
                    Err(Box::<dyn Error + Send + Sync>::from(
                        "Cannot get rules from the main namespace",
                    ))
                } else {
                    info!("Rules are in namespace: {:?}", n);
                    // Split the namespace into the first alias and the rest
                    let first_alias = n[0].clone();
                    n = n[1..].to_vec(); // Remove the first alias from the namespace

                    // Get the first layer namespace or return an error
                    let alias_namespace = if let Some(r) = self.aliases.get(&first_alias) {
                        r.value().clone()
                    } else {
                        return Err(Box::<dyn Error + Send + Sync>::from(format!(
                            "No matching namespace found for alias: {}",
                            first_alias
                        )));
                    };
                    // Get the previous rules
                    let prev_rules = alias_namespace.get_rules(n.clone()).await?;

                    // Get the intersection of the rules with the previous rules
                    let matching_rules = rules
                        .iter()
                        .filter(|rule| prev_rules.contains(rule))
                        .cloned()
                        .collect::<Vec<Rule>>();

                    // Need to check every rule in the compound to determine the overall value (Only checking for reactive rules currently)
                    let statuses: DashSet<ActivationStatus> = DashSet::new();
                    for rule in matching_rules {
                        match rule {
                            Rule::Reactive(rr) => {
                                // Get the status of the reactive rule
                                let s = self
                                    .clone()
                                    .get_rule_status(&ReactiveRuleWithArgs::new(
                                        rr,
                                        ActivationStatus::True,
                                        var_namespace_copy.clone(),
                                    ))
                                    .await?;
                                statuses.insert(s);
                            }
                            _ => {}
                        }
                    }
                    // Determine the overall value based on the statuses
                    let overall = overall_status_from_set(&statuses)?;
                    Ok(overall)
                }
            }
            AtomicCondition::SubCompound {
                namespace,
                condition,
            } => {
                // Handle sub-compound conditions
                // Calculate the new namespace
                let n = match var_namespace {
                    Some(ns) => {
                        let mut new_ns = vec![namespace];
                        new_ns.append(&mut ns.clone());
                        new_ns
                    }
                    None => vec![namespace],
                };

                // Handle sub-compound conditions
                self.get_atomic_condition(*condition, Some(n)).await
            }
        }
    }

    // Get a rule status
    async fn get_rule_status(
        self: Arc<Self>,
        rule: &ReactiveRuleWithArgs,
    ) -> Result<ActivationStatus, Box<dyn std::error::Error + Send + Sync>> {
        // Log the rule being processed
        let rule_desc = rule.rule.to_string();
        debug!("Getting status for rule: {}", rule_desc);

        // Get the rule's identifier
        let handler_id = rule.rule.get_identifier();
        // Check if the handler exists
        match self.event_handlers.get(&handler_id) {
            None => {
                // If the handler does not exist
                Err(Box::<dyn Error + Send + Sync>::from(format!(
                    "Handler for rule {} not found",
                    handler_id
                )))
            }
            Some(handler_entry) => {
                // Get the handler
                let handler = handler_entry.value();

                // Get the rules from the handler
                let rules = handler.api.get_rules.call(true).await?;

                // Go through the rules and find the one that matches the rule's identifier
                for r in rules {
                    // Log the rule being checked
                    let r_desc = r.rule.to_string();
                    debug!("Checking rule: {}", r_desc);
                    if r.rule == rule.rule && r.alias == rule.alias {
                        // If the rule matches, return its value
                        return Ok(r.value.clone());
                    }
                }
                // If no matching rule was found, return an error
                Err(Box::<dyn Error + Send + Sync>::from(format!(
                    "Rule {} not found in handler {}",
                    rule.rule.get_identifier(),
                    handler_id
                )))
            }
        }
    }

    /// Updates an atomic condition's value atomically.
    #[instrument(skip(self, var, value))]
    fn update_var(
        self: Arc<Self>,
        var: PrimitiveCondition,
        value: ActivationStatus,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Log the variable being updated
        let var_desc = var.to_string();
        debug!("Updating variable: {} to value: {:?}", var_desc, value);

        // Check if the value is Unknown, which is not allowed
        if value == ActivationStatus::Conflict {
            return Err(Box::<dyn Error + Send + Sync>::from(
                "Cannot update variable to Unknown",
            ));
        }

        self.vars.insert(var, value);
        Ok(true)
    }

    /// Processes a rule with arguments, handling reactive rules, case rules, and fact rules.
    #[instrument(skip(self, rule_with_args))]
    #[async_recursion]
    async fn process_rule(
        self: Arc<Self>,
        rule_with_args: RuleWithArgs,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Log the rule being processed
        let rule_desc = Rule::from(rule_with_args.clone()).to_string();
        debug!("Processing rule: {}", rule_desc);

        // Match on the type of rule to determine how to handle it
        let result: Result<bool, Box<dyn std::error::Error + Send + Sync>> = match &rule_with_args {
            // Reactive rules: check if the handler already exists, create it if not, or add the rule to the existing handler
            RuleWithArgs::Reactive(reactive_rule) => {
                // Get the rule's identifier
                let handler_id = reactive_rule.rule.get_identifier();

                // Check if the handler already exists
                match self.event_handlers.get(&handler_id) {
                    // If the handler does not exist, create a new one
                    None => {
                        debug!("Creating new handler: {}", handler_id);
                        let new_handler =
                            Arc::new(EventHandler::new(self.clone(), reactive_rule.clone()));
                        self.event_handlers.insert(handler_id.clone(), new_handler);
                        info!("Created new handler for rule: {}", handler_id.clone());
                        debug!("Current handlers size: {:?}", self.event_handlers.len());
                        Ok(true)
                    }
                    // If the handler exists, add the rule to it
                    Some(handler) => {
                        debug!("Adding rule to existing handler: {}", handler_id);
                        let handler = handler.clone();
                        handler.add_rule(reactive_rule.clone()).await
                    }
                }
            }
            // Case rules: process the action immediately
            RuleWithArgs::Case(CaseRule { action }) => {
                let res = self.clone().process_action(action.clone()).await;
                match res {
                    Ok(val) => {
                        debug!("Processed case rule with action: {:?}", action);
                        Ok(val)
                    }
                    Err(e) => {
                        error!("Failed to process action: {}", e);
                        return Err(Box::<dyn std::error::Error + Send + Sync>::from(e));
                    }
                }
            }
            // Fact rules: store the atomic condition with the provided value
            RuleWithArgs::Fact(FactRuleWithArgs { rule, value }) => {
                match &rule.condition {
                    AtomicCondition::Primitive(_) => {
                        // By default, primitive conditions are set to True
                        let real_val = value.clone().map_or(ActivationStatus::True, |v| v.clone());
                        self.store_atomic_condition(rule.condition.clone(), real_val, None, true)
                            .await
                    }
                    _ => {
                        // By default, compound and sub-compound rules are set to false
                        let real_val = value.clone().map_or(ActivationStatus::False, |v| v.clone());
                        self.store_atomic_condition(rule.condition.clone(), real_val, None, true)
                            .await
                    }
                }
            }
            _ => Ok(false),
        };
        result
    }
}
