use cl0_parser::ast::{CaseRule, DeclarativeRule, FactRule, ReactiveRule, Rule};
use dashmap::{DashMap, DashSet};
use futures::future::join_all;
use std::{error::Error, fmt};
use tokio::task::JoinHandle;

/// Represents a non-reactive rule, which can be declarative, case-based, or fact-based.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NonReactiveRule {
    Fact(FactRule),
}

/// Represents a reactive rule with an optional alias and a value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactiveRuleWithArgs {
    pub rule: ReactiveRule,
    pub value: VarValue,
    pub alias: Option<Vec<String>>,
}
impl ReactiveRuleWithArgs {
    pub fn new(rule: ReactiveRule, value: VarValue, alias: Option<Vec<String>>) -> Self {
        ReactiveRuleWithArgs { rule, value, alias }
    }
}

/// Represents a FactRule with an optional value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactRuleWithArgs {
    pub rule: FactRule,
    pub value: Option<VarValue>,
}
impl FactRuleWithArgs {
    pub fn new(rule: FactRule, value: Option<VarValue>) -> Self {
        FactRuleWithArgs { rule, value }
    }
}

/// Represents a rule with extra arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RuleWithArgs {
    Declarative(DeclarativeRule),
    Case(CaseRule),
    Fact(FactRuleWithArgs),
    Reactive(ReactiveRuleWithArgs),
}
impl From<RuleWithArgs> for Rule {
    fn from(rwa: RuleWithArgs) -> Rule {
        match rwa {
            RuleWithArgs::Declarative(d) => Rule::Declarative(d),
            RuleWithArgs::Case(c) => Rule::Case(c),
            RuleWithArgs::Fact(FactRuleWithArgs { rule, .. }) => Rule::Fact(rule),
            RuleWithArgs::Reactive(ReactiveRuleWithArgs { rule, .. }) => Rule::Reactive(rule),
        }
    }
}

/// The possible values a condition variable can take in the system.
///
/// - `True` and `False` are concrete boolean states.
/// - `Unknown` represents a value that is not yet determined; callers can choose to
///   treat it differently (e.g., continue or fail) depending on context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarValue {
    True,
    False, // Inactive
    Unknown,
}

impl VarValue {
    /// Returns `Some(bool)` for concrete values, or `None` for `Unknown`.
    pub fn as_option_bool(&self) -> Option<bool> {
        match self {
            VarValue::True => Some(true),
            VarValue::False => Some(false),
            VarValue::Unknown => None,
        }
    }

    /// Converts into a boolean, returning an error for ambiguous states.
    pub fn to_bool(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.as_option_bool()
            .ok_or(Box::<dyn Error + Send + Sync>::from(format!(
                "Not a boolean value: {}",
                self
            )))
    }
}

impl fmt::Display for VarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarValue::True => write!(f, "True"),
            VarValue::False => write!(f, "False"),
            VarValue::Unknown => write!(f, "Unknown"),
        }
    }
}

/// From a set of status values, return what the overall status is:
/// - If any are `Unknown`, return an error.
/// - If all are `True`, return `True`.
/// - If any are `False`, return `False`.
/// If no valid status is found, return an error.
pub fn overall_status_from_set(
    statuses: &DashSet<VarValue>,
) -> Result<VarValue, Box<dyn Error + Send + Sync>> {
    if statuses.contains(&VarValue::Unknown) {
        return Err(Box::<dyn Error + Send + Sync>::from(
            "Overall status is unknown due to at least one Unknown value",
        ));
    }
    if statuses.contains(&VarValue::False) {
        return Ok(VarValue::False);
    }
    if statuses.contains(&VarValue::True) {
        return Ok(VarValue::True);
    }
    Err(Box::<dyn Error + Send + Sync>::from("No valid status found"))
}

/// Awaits a collection of `JoinHandle<Result<bool, E>>`, returns the conjunction
/// of all their successful boolean results, or the first error encountered.
pub async fn collect_conjunction(
    handles: Vec<JoinHandle<Result<bool, Box<dyn std::error::Error + Send + Sync>>>>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut overall = true;
    for join_res in join_all(handles).await {
        match join_res {
            Ok(inner) => match inner {
                Ok(val) => overall &= val,
                Err(e) => return Err(e),
            },
            Err(join_err) => return Err(Box::<dyn Error + Send + Sync>::from(join_err)),
        }
    }
    Ok(overall)
}

/// Represents a namespace for aliases, which can contain sub-namespaces and rules.
#[derive(Debug, Clone)]
pub struct AliasNamespace {
    pub sub_namespaces: DashMap<String, AliasNamespace>,
    pub rules: Vec<Rule>,
}
impl AliasNamespace {
    /// Creates a new empty `AliasNamespace`.
    pub fn new() -> Self {
        AliasNamespace {
            sub_namespaces: DashMap::new(),
            rules: Vec::new(),
        }
    }
    /// Retrieves rules from the current namespace or sub-namespaces based on the provided aliases.
    pub fn get_rules(&self, aliases: Vec<String>) -> Result<Vec<Rule>, Box<dyn Error + Send + Sync>> {
        // Check if there are any aliases to process. If empty, we are in the main namespace
        if aliases.is_empty() {
            return Ok(self.rules.clone());
        }
        
        // Split the aliases list into the first alias and the rest
        let first_alias = aliases[0].clone();
        let rest_aliases = &aliases[1..];

        if let Some(ns) = self.sub_namespaces.get(&first_alias) {
            ns.get_rules(rest_aliases.to_vec())
        } else {
            Err(Box::<dyn Error + Send + Sync>::from(
                format!("No matching namespace found for alias: {}", first_alias),
            ))
        }
    }
    /// Creates rules in the current namespace or sub-namespaces based on the provided aliases.
    pub fn create_rules(
        &mut self,
        aliases: Vec<String>,
        rules: Vec<Rule>,
    ) -> Result<Option<Vec<Rule>>, Box<dyn Error + Send + Sync>> {
        // Check if there are any aliases to process. If empty, we are in the main namespace
        if aliases.is_empty() {
            let existing_rules = self.rules.clone();
            self.rules = rules.clone();
            if existing_rules.is_empty() {
                return Ok(None); // No existing rules, just return None
            }
            return Ok(Some(existing_rules));
        }

        // Split the aliases list into the first alias and the rest
        let first_alias = aliases[0].clone();
        let rest_aliases = &aliases[1..];

        // Get or create the sub-namespace for the first alias
        let mut sub_ns = self.sub_namespaces.entry(first_alias).or_insert_with(AliasNamespace::new);
        
        // Recursively create rules in the sub-namespace
        sub_ns.value_mut().create_rules(rest_aliases.to_vec(), rules)
    }
}