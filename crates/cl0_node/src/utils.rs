use cl0_parser::ast::{CaseRule, DeclarativeRule, FactRule, ReactiveRule, Rule};
use dashmap::{DashMap, DashSet};
use futures::future::join_all;
use std::{error::Error, fmt, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};
use async_recursion::async_recursion;

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
    Err(Box::<dyn Error + Send + Sync>::from(
        "No valid status found",
    ))
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
#[derive(Debug)]
pub struct AliasNamespace {
    sub_namespaces: DashMap<String, Arc<AliasNamespace>>,
    rules: RwLock<Vec<Rule>>, // <-- Tokio’s async RwLock
}

impl AliasNamespace {
    pub fn new() -> Self {
        AliasNamespace {
            sub_namespaces: DashMap::new(),
            rules: RwLock::new(Vec::new()),
        }
    }

    /// Retrieves rules from this namespace or one of its descendants.
    #[async_recursion]
    pub async fn get_rules(
        &self,
        mut aliases: Vec<String>,
    ) -> Result<Vec<Rule>, Box<dyn Error + Send + Sync>> {
        if aliases.is_empty() {
            let guard = self.rules.read().await;
            return Ok(guard.clone());
        }

        let first = aliases.remove(0);
        if let Some(child) = self.sub_namespaces.get(&first) {
            child.get_rules(aliases).await
        } else {
            Err(format!("No matching namespace for alias `{}`", first).into())
        }
    }

    /// Creates (or replaces) rules in this namespace or a descendant.
    #[async_recursion]
    pub async fn create_rules(
        &self,
        mut aliases: Vec<String>,
        new_rules: Vec<Rule>,
        override_entries: bool,
    ) -> Result<Option<Vec<Rule>>, Box<dyn Error + Send + Sync>> {
        if aliases.is_empty() {
            let mut guard = self.rules.write().await;
            let old = guard.clone();
            if override_entries {
                // Replace all rules
                *guard = new_rules;
            } else {
                // Union: combine old and new, remove duplicates
                let mut combined = old.clone();
                for rule in new_rules {
                    if !combined.contains(&rule) {
                        combined.push(rule);
                    }
                }
                *guard = combined;
            }
            return if old.is_empty() {
                Ok(None)
            } else {
                Ok(Some(old))
            };
        }

        let first = aliases.remove(0);
        let child = self
            .sub_namespaces
            .entry(first.clone())
            .or_insert_with(|| Arc::new(AliasNamespace::new()))
            .clone();
        child.create_rules(aliases, new_rules, override_entries).await
    }
}


/// Get head and tail of a path.
pub fn get_parts<T>(path: Vec<T>) -> (T, Vec<T>, Vec<T>)
where
    T: Clone,
{
    if path.is_empty() {
        panic!("Path cannot be empty");
    }
    let head = path[0].clone();
    let tail = path[1..].to_vec();
    let full_path = path.clone();
    (head, tail, full_path)
}