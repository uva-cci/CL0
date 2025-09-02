use cl0_parser::ast::Rule;
use dashmap::{DashMap, DashSet};
use futures::future::join_all;
use std::{error::Error, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};
use async_recursion::async_recursion;

use crate::types::ActivationStatus;

/// From a set of status values, return what the overall status is:
/// - If any are `Unknown`, return an error.
/// - If all are `True`, return `True`.
/// - If any are `False`, return `False`.
/// If no valid status is found, return an error.
pub fn overall_status_from_set(
    statuses: &DashSet<ActivationStatus>,
) -> Result<ActivationStatus, Box<dyn Error + Send + Sync>> {
    if statuses.contains(&ActivationStatus::Conflict) {
        return Err(Box::<dyn Error + Send + Sync>::from(
            "Overall status is unknown due to at least one Unknown value",
        ));
    }
    if statuses.contains(&ActivationStatus::False) {
        return Ok(ActivationStatus::False);
    }
    if statuses.contains(&ActivationStatus::True) {
        return Ok(ActivationStatus::True);
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