use async_recursion::async_recursion;
use cl0_parser::ast::{
    Action, ActionList, AtomicCondition, CaseRule, Compound, Condition, FactRule,
    PrimitiveCondition, PrimitiveEvent, Rule,
};
use dashmap::{DashMap, DashSet};
use rand::seq::IndexedRandom;
use std::error::Error;
use std::sync::{Arc, Weak};
use tokio::sync::Notify;
use tracing::{debug, error, info, instrument, warn};

use crate::api::ApiRoute;
use crate::event_handler::EventHandler;
use crate::utils::{
    AliasNamespace, FactRuleWithArgs, ReactiveRuleWithArgs, RuleWithArgs, VarValue,
    collect_conjunction, overall_status_from_set,
};
use crate::visitor::AstVisitor;

/// Public API surface for a Node: adding/getting rules.
#[derive(Debug)]
pub struct NodeApi {
    pub new_rules: ApiRoute<Vec<RuleWithArgs>, Vec<bool>>,
    pub get_rules: ApiRoute<(), Vec<ReactiveRuleWithArgs>>,
}

/// Core node that maintains variable state, aliases, and event handlers.
#[derive(Debug)]
pub struct Node {
    pub vars: Arc<DashMap<PrimitiveCondition, VarValue>>,
    pub aliases: Arc<DashMap<String, AliasNamespace>>,
    pub event_handlers: Arc<DashMap<String, Arc<EventHandler>>>,
    pub api: NodeApi,
}

impl Node {
    /// Async constructor that builds the node and applies initial rules if provided.
    pub async fn new_with_rules(rules: Option<Vec<Rule>>) -> Arc<Self> {
        // Use `Arc::new_cyclic` to get a self-referential structure safely
        let node = Arc::new_cyclic(|weak_node: &Weak<Node>| {
            // Shared internal state
            let vars: Arc<DashMap<PrimitiveCondition, VarValue>> = Arc::new(DashMap::new());
            let aliases: Arc<DashMap<String, AliasNamespace>> = Arc::new(DashMap::new());
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
            let get_rules = ApiRoute::new(move |(): ()| {
                let handlers = handlers_for_get.clone();
                async move {
                    let mut rules_accum: Vec<ReactiveRuleWithArgs> = Vec::new();
                    for handler_ref in handlers.iter() {
                        let id = handler_ref.key();
                        debug!("Collecting rules from handler: {}", id);
                        let handler = handler_ref.value().clone(); // Arc<EventHandler>
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
                let _ = Self::store_atomic_condition(node.clone(), ac, VarValue::False, None).await;
            }

            // Execute all case rules at the end of initialization
            debug!("Initializing Node with rules: {:?}", initial_rules.clone());
            let other_rules: Vec<RuleWithArgs> = initial_rules
                .clone()
                .into_iter()
                .filter_map(|rule| match rule {
                    Rule::Reactive(r) => Some(RuleWithArgs::Reactive(ReactiveRuleWithArgs {
                        rule: r,
                        value: VarValue::True,
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
                },
                PrimitiveEvent::Production(ac) => {
                    self.store_atomic_condition(ac, VarValue::True, None).await
                }
                PrimitiveEvent::Consumption(ac) => {
                    self.store_atomic_condition(ac, VarValue::False, None).await
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

    #[instrument(skip(self, condition, value, var_namespace))]
    #[async_recursion]
    pub async fn store_atomic_condition(
        self: Arc<Self>,
        condition: AtomicCondition,
        value: VarValue,
        var_namespace: Option<Vec<String>>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
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
                    let mut alias_namespace = self
                        .aliases
                        .entry(first_alias.clone())
                        .or_insert_with(AliasNamespace::new);

                    // Create or update the sub-namespace for the remaining aliases
                    let prev_rules = alias_namespace.create_rules(n, rules.clone());

                    // Log what happened
                    match prev_rules {
                        Ok(Some(existing_rules)) => {
                            warn!("Overriding existing rules: {:?}", existing_rules);
                        }
                        Ok(None) => {
                            info!("No existing rules to override, creating new ones");
                        }
                        Err(e) => {
                            error!("Failed to create or update rules: {}", e);
                            return Err(e);
                        }
                    }
                }

                // Convert the rules into RuleWithArgs format
                let rules_with_args: Vec<RuleWithArgs> = rules
                    .into_iter()
                    .map(|r| match r {
                        Rule::Reactive(rr) => RuleWithArgs::Reactive(ReactiveRuleWithArgs::new(
                            rr.clone(),
                            value.clone(),
                            var_namespace_copy.clone(),
                        )),
                        Rule::Fact(fact_rule) => RuleWithArgs::Fact(FactRuleWithArgs {
                            rule: fact_rule.clone(),
                            value: Some(value.clone()),
                        }),
                        Rule::Case(case_rule) => RuleWithArgs::Case(case_rule.clone()),
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
                self.store_atomic_condition(*condition, VarValue::False, Some(n))
                    .await
            }
        }
    }

    /// Retrieves the value of an atomic condition.
    #[instrument(skip(self, condition, var_namespace))]
    #[async_recursion]
    pub async fn get_atomic_condition(
        self: Arc<Self>,
        condition: AtomicCondition,
        var_namespace: Option<Vec<String>>,
    ) -> Result<VarValue, Box<dyn std::error::Error + Send + Sync>> {
        match condition {
            AtomicCondition::Primitive(prim_cond) => {
                let v_ref = self.vars.get(&prim_cond);
                match v_ref {
                    None => Ok(VarValue::Unknown),
                    Some(entry) => Ok(entry.clone()),
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
                    if let Some(alias_namespace) = self.clone().aliases.get(&first_alias) {
                        // Get the previous rules
                        let prev_rules = alias_namespace.get_rules(n.clone())?;

                        // Get the intersection of the rules with the previous rules
                        let matching_rules = rules
                            .iter()
                            .filter(|rule| prev_rules.contains(rule))
                            .cloned()
                            .collect::<Vec<Rule>>();

                        // Need to check every rule in the compound to determine the overall value (Only checking for reactive rules currently)
                        let statuses: DashSet<VarValue> = DashSet::new();
                        for rule in matching_rules {
                            match rule {
                                Rule::Reactive(rr) => {
                                    // Get the status of the reactive rule
                                    let s = self
                                        .clone()
                                        .get_rule_status(&ReactiveRuleWithArgs::new(
                                            rr,
                                            VarValue::True,
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
                    } else {
                        return Err(Box::<dyn Error + Send + Sync>::from(format!(
                            "No matching namespace found for alias: {}",
                            first_alias
                        )));
                    }
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
    ) -> Result<VarValue, Box<dyn std::error::Error + Send + Sync>> {
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
                let rules = handler.api.get_rules.call(()).await?;

                // Go through the rules and find the one that matches the rule's identifier
                for r in rules {
                    if r.rule == rule.rule && r.alias == rule.alias {
                        // If the rule matches, return its value
                        return Ok(rule.value.clone());
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
        value: VarValue,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if value == VarValue::Unknown {
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
                        let real_val = value.clone().map_or(VarValue::True, |v| v.clone());
                        self.store_atomic_condition(rule.condition.clone(), real_val, None)
                            .await
                    }
                    _ => {
                        // By default, compound and sub-compound rules are set to false
                        let real_val = value.clone().map_or(VarValue::False, |v| v.clone());
                        self.store_atomic_condition(rule.condition.clone(), real_val, None)
                            .await
                    }
                }
            }
            _ => Ok(false),
        };
        result
    }
}
