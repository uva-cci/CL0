
//! Generic AST visitor pattern for traversing the CL0 AST.
//!
//! This module defines the [`AstVisitor`] trait, which enables recursive traversal of the CL0 AST types
//! (such as `Rule`, `Condition`, `Action`, etc.) using a generic function or closure. The visitor pattern
//! allows you to apply custom logic to any node in the AST by passing a function that receives a `&dyn Any` reference.
//!
//! # Example
//!
//! ```rust
//! use cl0_parser::ast::{Rule, AtomicCondition};
//! use your_crate::visitor::AstVisitor;
//!
//! let rule: Rule = /* ... */;
//! let mut found = vec![];
//! rule.visit(&mut |node| {
//!     if let Some(ac) = node.downcast_ref::<AtomicCondition>() {
//!         found.push(ac.clone());
//!     }
//! });
//! ```
//!
//! This pattern is flexible and allows you to collect, analyze, or transform AST nodes of any type.

use std::any::Any;
use cl0_parser::ast::{Action, ActionList, AtomicCondition, CaseRule, Compound, Condition, DeclarativeRule, FactRule, PrimitiveCondition, PrimitiveEvent, ReactiveRule, Rule};


/// Trait for recursively visiting all components in the CL0 AST.
///
/// The `visit` method takes a mutable function or closure, which is called on every node (including itself)
/// as the AST is traversed. The function receives a `&dyn Any` reference, allowing for type checks and downcasting.
///
/// Implementations are provided for all major AST types.
pub trait AstVisitor {
    /// Recursively visit this node and all children, calling the provided function on each.
    ///
    /// # Arguments
    /// * `f` - A mutable function or closure that takes a `&dyn Any` reference to each node.
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F);
}

/// Visitor implementation for `Condition`.
impl AstVisitor for Condition {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            Condition::Atomic(ac) => {
                ac.visit(f);
            },
            Condition::Not(condition) => {
                condition.visit(f);
            }
            Condition::Conjunction(conditions) => {
                for condition in conditions {
                    condition.visit(f);
                }
            }
            Condition::Disjunction(conditions) => {
                for condition in conditions {
                    condition.visit(f);
                }
            }
            Condition::Parentheses(condition) => {
                condition.visit(f);
            }
        }
    }
}

/// Visitor implementation for `PrimitiveCondition`.
impl AstVisitor for PrimitiveCondition {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
    }
}

/// Visitor implementation for `AtomicCondition`.
impl AstVisitor for AtomicCondition {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            AtomicCondition::Primitive(pc) => {
                pc.visit(f);
            },
            AtomicCondition::Compound(compound) => {
                compound.visit(f);
            }
            AtomicCondition::SubCompound { condition, .. } => {
                condition.visit(f);
            }
        }
    }
}

/// Visitor implementation for `ActionList`.

impl AstVisitor for ActionList {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            ActionList::Sequence(actions) => {
                for action in actions {
                    action.visit(f);
                }
            }
            ActionList::Parallel(actions) => {
                for action in actions {
                    action.visit(f);
                }
            }
            ActionList::Alternative(actions) => {
                for action in actions {
                    action.visit(f);
                }
            }
        }
    }
}

/// Visitor implementation for `PrimitiveEvent`.
impl AstVisitor for PrimitiveEvent {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            PrimitiveEvent::Production(condition) => {
                condition.visit(f);
            }
            PrimitiveEvent::Consumption(condition) => {
                condition.visit(f);
            }
            _ => {}
        }
    }
}

/// Visitor implementation for `Action`.
impl AstVisitor for Action {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            Action::Primitive(event) => {
                event.visit(f);
            }
            Action::List(action_list) => {
                action_list.visit(f);
            }
        }
    }
}

/// Visitor implementation for `ReactiveRule`.
impl AstVisitor for ReactiveRule {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            ReactiveRule::ECA { event, condition, action } => {
                event.visit(f);
                if let Some(cond) = condition {
                    cond.visit(f);
                }
                action.visit(f);
            }
            ReactiveRule::CA { condition, action } => {
                condition.visit(f);
                action.visit(f);
            }
        }
    }
}

/// Visitor implementation for `DeclarativeRule`.
impl AstVisitor for DeclarativeRule {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            DeclarativeRule::CC { premise, condition } => {
                if let Some(p) = premise {
                    p.visit(f);
                }
                condition.visit(f);
            }
            DeclarativeRule::CT { premise, condition } => {
                if let Some(p) = premise {
                    p.visit(f);
                }
                condition.visit(f);
            }
        }
    }
}

/// Visitor implementation for `CaseRule`.
impl AstVisitor for CaseRule {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        self.action.visit(f);
    }
}

/// Visitor implementation for `FactRule`.
impl AstVisitor for FactRule {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        self.condition.visit(f);
    }
}

/// Visitor implementation for `Rule`.
impl AstVisitor for Rule {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        match self {
            Rule::Reactive(rule) => {
                rule.visit(f);
            }
            Rule::Declarative(rule) => {
                rule.visit(f);
            }
            Rule::Case(rule) => {
                rule.visit(f);
            }
            Rule::Fact(rule) => {
                rule.visit(f);
            }
        }
    }
}

/// Visitor implementation for `Compound`.
impl AstVisitor for Compound {
    fn visit<F: FnMut(&dyn Any)>(&self, f: &mut F) {
        f(self);
        for rule in &self.rules {
            rule.visit(f);
        }
    }
}