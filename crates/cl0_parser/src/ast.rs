use std::fmt::{self};

/// Logical condition type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Condition {
    /// A plain variable (e.g., `loaded`)
    Atomic(AtomicCondition),
    /// Logical negation (e.g., `not loaded`)
    Not(Box<Self>),
    /// A conjunction (AND) of two conditions (e.g., `loaded AND ready`)
    Conjunction(Vec<Self>),
    /// A disjunction (OR) of two conditions (e.g., `loaded OR ready`)
    Disjunction(Vec<Self>),
    /// Parenthesized condition (e.g., `(loaded AND ready)`)
    Parentheses(Box<Self>),
}
/// Implements the Display trait for Condition, allowing it to be formatted as a string.
impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Condition::Atomic(atomic_condition) => write!(f, "{}", atomic_condition.to_string()),
            Condition::Not(condition) => write!(f, "not {}", condition.to_string()),
            Condition::Conjunction(conditions) => {
                let joined = conditions
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" and ");
                write!(f, "{}", joined)
            }
            Condition::Disjunction(conditions) => {
                let joined = conditions
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(" or ");
                write!(f, "{}", joined)
            }
            Condition::Parentheses(condition) => write!(f, "({})", condition.to_string()),
        }
    }
}

/// A primitive condition is a basic variable or identifier used in conditions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveCondition {
    Var(String),
}
/// Implements the Display trait for PrimitiveCondition, allowing it to be formatted as a string.
impl fmt::Display for PrimitiveCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveCondition::Var(v) => write!(f, "{}", v.to_string()),
        }
    }
}

/// An atomic condition can be either a compound condition or a primitive condition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AtomicCondition {
    Primitive(PrimitiveCondition),
    Compound(Compound),
    SubCompound {
        /// The namespace of the sub-compound condition, which is used to identify it.
        namespace: String,
        /// A sub-compound condition, which is a compound condition that can be nested.
        condition: Box<Self>,
    },
}
/// Implements the Display trait for AtomicCondition, allowing it to be formatted as a string.
impl fmt::Display for AtomicCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomicCondition::Primitive(primitive_condition) => {
                write!(f, "{}", primitive_condition.to_string())
            }
            AtomicCondition::Compound(compound) => write!(f, "{}", compound.to_string()),
            AtomicCondition::SubCompound {
                namespace,
                condition,
            } => {
                write!(f, "{}.{}", namespace.to_string(), condition.to_string())
            }
        }
    }
}

/// A sequence of actions can be a sequence, parallel, or alternative execution.    
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionList {
    /// A sequence of actions (e.g., `a seq b` or `a; b`)
    Sequence(Vec<Action>),
    /// A parallel execution of actions (e.g., `a par b` or `a, b`)
    Parallel(Vec<Action>),
    /// An alternative choice of actions (e.g., `a alt b`)
    Alternative(Vec<Action>),
}
/// Implements the Display trait for ActionList, allowing it to be formatted as a string.
impl fmt::Display for ActionList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionList::Sequence(list) => write!(
                f,
                "{}",
                list.into_iter()
                    .map(|s| { s.to_string() })
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
            ActionList::Parallel(list) => write!(
                f,
                "{}",
                list.into_iter()
                    .map(|s| { s.to_string() })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ActionList::Alternative(list) => write!(
                f,
                "{}",
                list.into_iter()
                    .map(|s| { s.to_string() })
                    .collect::<Vec<_>>()
                    .join("alt ")
            ),
        }
    }
}

/// Represents a primitive event, which can be a trigger, production, or consumption event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveEvent {
    Trigger(String),              // #Identifier
    Production(AtomicCondition),  // +Identifier
    Consumption(AtomicCondition), // -Identifier
}
impl PrimitiveEvent {
    /// Returns the identifier of the primitive event, which is the identifier of what triggers the event.
    pub fn get_identifier(&self) -> String {
        match self {
            PrimitiveEvent::Trigger(id) => id.to_string(),
            PrimitiveEvent::Production(cond) => cond.to_string(),
            PrimitiveEvent::Consumption(cond) => cond.to_string(),
        }
    }
}
/// Implements the Display trait for PrimitiveEvent, allowing it to be formatted as a string.
impl fmt::Display for PrimitiveEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveEvent::Trigger(id) => write!(f, "#{}", id.to_string()),
            PrimitiveEvent::Production(cond) => write!(f, "+{}", cond.to_string()),
            PrimitiveEvent::Consumption(cond) => write!(f, "-{}", cond.to_string()),
        }
    }
}

/// Represents an action, which can be a primitive event or a sequence of actions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    /// A single event action, like `#click`, `+load`, or `-submit`
    Primitive(PrimitiveEvent),
    /// A sequence of actions, like `seq a seq b seq c` seq(test, test2, par(test3, test4), etc)
    List(ActionList),
}
/// Implements the Display trait for Action, allowing it to be formatted as a string.
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Primitive(event) => write!(f, "{}", event.to_string()),
            Action::List(action_list) => write!(f, "{}", action_list.to_string()),
        }
    }
}

/// Represents a reactive rule, which can be either an ECA (Event-Condition-Action) or CA (Condition-Action).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReactiveRule {
    ECA {
        event: PrimitiveEvent,
        condition: Option<Condition>,
        action: Action,
    },
    CA {
        condition: Condition,
        action: Action,
    },
}
impl ReactiveRule {
    /// Returns the identifier of the reactive rule, which is the identifier of what triggers the event.
    pub fn get_identifier(&self) -> String {
        match self {
            ReactiveRule::ECA { event, .. } => event.get_identifier(),
            ReactiveRule::CA { .. } => "".to_string(),
        }
    }
}
/// Implements the Display trait for ReactiveRule, allowing it to be formatted as a string.
impl fmt::Display for ReactiveRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReactiveRule::ECA {
                event,
                condition,
                action,
            } => match condition {
                Some(c) => write!(
                    f,
                    "{}: {} => {}.",
                    event.to_string(),
                    c.to_string(),
                    action.to_string()
                ),
                None => write!(f, "{} => {}.", event.to_string(), action.to_string()),
            },
            ReactiveRule::CA { condition, action } => {
                write!(f, ":{} => {}.", condition.to_string(), action.to_string())
            }
        }
    }
}

/// Represents a declarative rule, which can be either a CC (->) or CT (-o).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeclarativeRule {
    CC {
        premise: Option<Condition>,
        condition: AtomicCondition,
    },
    CT {
        premise: Option<Condition>,
        condition: Condition,
    },
}
impl DeclarativeRule {
    /// Returns the identifier of the declarative rule, which is the premise.
    pub fn get_identifier(&self) -> String {
        match self {
            DeclarativeRule::CC { premise, .. } => match premise {
                Some(c) => c.to_string(),
                None => "".to_string(),
            },
            DeclarativeRule::CT { premise, .. } => match premise {
                Some(c) => c.to_string(),
                None => "".to_string(),
            },
        }
    }
}
/// Implements the Display trait for DeclarativeRule, allowing it to be formatted as a string.
impl fmt::Display for DeclarativeRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclarativeRule::CC { premise, condition } => match premise {
                Some(c) => write!(f, "{} -> {}.", c.to_string(), condition.to_string()),
                None => write!(f, "-> {}.", condition.to_string()),
            },
            DeclarativeRule::CT { premise, condition } => match premise {
                Some(p) => write!(f, "{} -> {}.", p.to_string(), condition.to_string()),
                None => write!(f, "-> {}.", condition.to_string()),
            },
        }
    }
}
/// Represents a case rule, which is a rule that only contains an action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseRule {
    /// The action to be taken when the case is triggered.
    pub action: Action,
}

/// Represents a fact rule, which is a rule that only contains a condition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactRule {
    /// The condition that must be satisfied for the fact to hold.
    pub condition: AtomicCondition,
}

/// Represents a rule in the system, which can be reactive, declarative, case-based, or fact-based.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Rule {
    Reactive(ReactiveRule),
    Declarative(DeclarativeRule),
    Case(CaseRule),
    Fact(FactRule),
}

/// Implements the Display trait for Rule, allowing it to be formatted as a string.
impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rule::Reactive(reactive_rule) => write!(f, "{}", reactive_rule.to_string()),
            Rule::Declarative(declarative_rule) => write!(f, "{}", declarative_rule.to_string()),
            Rule::Case(CaseRule { action }) => write!(f, "=> {}.", action.to_string()),
            Rule::Fact(FactRule { condition }) => write!(f, "{}.", condition.to_string()),
        }
    }
}
impl Rule {
    /// Returns the identifier of the rule, which is the identifier of what triggers the event.
    pub fn get_identifier(&self) -> Option<String> {
        match self {
            Rule::Reactive(reactive_rule) => Some(reactive_rule.get_identifier()),
            Rule::Declarative(declarative_rule) => Some(declarative_rule.get_identifier()),
            Rule::Case { .. } => None,
            Rule::Fact { .. } => None,
        }
    }
}

// Represents a compound rule, which can contain multiple rules and an optional alias to refer to
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Compound {
    pub rules: Vec<Rule>,
    pub alias: Option<String>,
}
/// Implements the Display trait for Compound, allowing it to be formatted as a string.
impl fmt::Display for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Compound { rules, alias } => {
                let rules_string = rules
                    .into_iter()
                    .map(|rule| rule.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                return match alias {
                    Some(alias) => write!(f, "{{ {} }} as {}", rules_string, alias),
                    None => write!(f, "{{ {} }}", rules_string),
                };
            }
        }
    }
}

// Represents a directive meaning different things
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Directive {
    Scale { number: u8, policy: Compound },
    Include(String),
    Exclude(String),
    Interleaving,
    ExternalVar(String),
    ExternalEvent(PrimitiveEvent),
}

/// Implements the Display trait for Compound, allowing it to be formatted as a string.
impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Directive::Scale { number, policy } => {
                write!(f, "@scale({})\n{}", number, policy)
            }
            Directive::Include(s) => {
                write!(f, "@include({})", s)
            }
            Directive::Exclude(s) => {
                write!(f, "@exclude({})", s)
            }
            Directive::Interleaving => {
                write!(f, "@interleaving")
            }
            Directive::ExternalVar(s) => {
                write!(f, "@external({})", s)
            }
            Directive::ExternalEvent(pe) => {
                write!(f, "@external({})", pe)
            }
        }
    }
}
