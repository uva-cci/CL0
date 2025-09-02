use cl0_parser::ast::{
    Action, ActionList, AtomicCondition, CaseRule, Compound, Condition, DeclarativeRule, FactRule,
    PrimitiveCondition, PrimitiveEvent, ReactiveRule, Rule,
};
use std::{error::Error, fmt};

use crate::generated;

/// Represents a non-reactive rule, which can be declarative, case-based, or fact-based.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NonReactiveRule {
    Fact(FactRule),
}

/// Represents a reactive rule with an optional alias and a value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReactiveRuleWithArgs {
    pub rule: ReactiveRule,
    pub value: ActivationStatus,
    pub alias: Option<Vec<String>>,
}
impl ReactiveRuleWithArgs {
    pub fn new(rule: ReactiveRule, value: ActivationStatus, alias: Option<Vec<String>>) -> Self {
        ReactiveRuleWithArgs { rule, value, alias }
    }
}

/// Represents a FactRule with an optional value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactRuleWithArgs {
    pub rule: FactRule,
    pub value: Option<ActivationStatus>,
}
impl FactRuleWithArgs {
    pub fn new(rule: FactRule, value: Option<ActivationStatus>) -> Self {
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
impl From<Rule> for RuleWithArgs {
    fn from(rw: Rule) -> RuleWithArgs {
        match rw {
            Rule::Declarative(d) => RuleWithArgs::Declarative(d),
            Rule::Case(c) => RuleWithArgs::Case(c),
            Rule::Fact(fr) => RuleWithArgs::Fact(FactRuleWithArgs {
                rule: fr,
                value: None, // Default value for fact rules
            }),
            Rule::Reactive(rr) => RuleWithArgs::Reactive(ReactiveRuleWithArgs {
                rule: rr,
                value: ActivationStatus::True, // Default value for reactive rules
                alias: None,                   // Default alias
            }),
        }
    }
}

/// The possible values a condition variable can take in the system.
///
/// - `True` and `False` are concrete boolean states.
/// - `Unknown` represents a value that is not yet determined; callers can choose to
///   treat it differently (e.g., continue or fail) depending on context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActivationStatus {
    True,
    False, // Inactive
    Conflict,
}

impl ActivationStatus {
    /// Returns `Some(bool)` for concrete values, or `None` for `Conflict`.
    pub fn as_option_bool(&self) -> Option<bool> {
        match self {
            ActivationStatus::True => Some(true),
            ActivationStatus::False => Some(false),
            ActivationStatus::Conflict => None,
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

impl fmt::Display for ActivationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivationStatus::True => write!(f, "True"),
            ActivationStatus::False => write!(f, "False"),
            ActivationStatus::Conflict => write!(f, "Unknown"),
        }
    }
}

impl TryFrom<generated::common::Rule> for Rule {
    type Error = String;

    fn try_from(rule: generated::common::Rule) -> Result<Self, Self::Error> {
        match rule.kind.ok_or("Missing Rule kind")? {
            generated::common::rule::Kind::Reactive(r) => {
                let r = ReactiveRule::try_from(r)?;
                Ok(Rule::Reactive(r))
            }
            generated::common::rule::Kind::Declarative(d) => {
                let d = DeclarativeRule::try_from(d)?;
                Ok(Rule::Declarative(d))
            }
            generated::common::rule::Kind::CaseRule(c) => {
                let action = Action::try_from(c.action.ok_or("Missing action in CaseRule")?)?;
                Ok(Rule::Case(CaseRule { action }))
            }
            generated::common::rule::Kind::FactRule(f) => {
                let condition =
                    AtomicCondition::try_from(f.condition.ok_or("Missing condition in FactRule")?)?;
                Ok(Rule::Fact(FactRule { condition }))
            }
        }
    }
}

impl From<Rule> for generated::common::Rule {
    fn from(rule: Rule) -> Self {
        use generated::common::rule::Kind;

        let kind = match rule {
            Rule::Reactive(r) => Kind::Reactive(r.into()),
            Rule::Declarative(d) => Kind::Declarative(d.into()),
            Rule::Case(c) => Kind::CaseRule(generated::common::CaseRule {
                action: Some(c.action.into()),
            }),
            Rule::Fact(f) => Kind::FactRule(generated::common::FactRule {
                condition: Some(f.condition.into()),
            }),
        };

        generated::common::Rule { kind: Some(kind) }
    }
}

impl TryFrom<generated::common::ReactiveRule> for ReactiveRule {
    type Error = String;

    fn try_from(r: generated::common::ReactiveRule) -> Result<Self, Self::Error> {
        let event = PrimitiveEvent::try_from(r.event.ok_or("Missing event")?)?;
        let condition = match r.condition {
            Some(c) => Some(Condition::try_from(c)?),
            None => None,
        };
        let action = Action::try_from(r.action.ok_or("Missing action")?)?;
        Ok(ReactiveRule::ECA {
            event,
            condition,
            action,
        })
    }
}

impl From<ReactiveRule> for generated::common::ReactiveRule {
    fn from(r: ReactiveRule) -> Self {
        match r {
            ReactiveRule::ECA {
                event,
                condition,
                action,
            } => generated::common::ReactiveRule {
                event: Some(event.into()),
                condition: condition.map(Into::into),
                action: Some(action.into()),
            },
            ReactiveRule::CA { condition, action } => generated::common::ReactiveRule {
                event: None, // CA maps to missing event
                condition: Some(condition.into()),
                action: Some(action.into()),
            },
        }
    }
}

impl TryFrom<generated::common::DeclarativeRule> for DeclarativeRule {
    type Error = String;

    fn try_from(d: generated::common::DeclarativeRule) -> Result<Self, Self::Error> {
        let premise = match d.premise {
            Some(p) => Some(Condition::try_from(p)?),
            None => None,
        };

        match d.target.ok_or("Missing target")? {
            generated::common::declarative_rule::Target::Cc(cc) => {
                let cond = AtomicCondition::try_from(cc)?;
                Ok(DeclarativeRule::CC {
                    premise,
                    condition: cond,
                })
            }
            generated::common::declarative_rule::Target::Ct(ct) => {
                let cond = Condition::try_from(ct)?;
                Ok(DeclarativeRule::CT {
                    premise,
                    condition: cond,
                })
            }
        }
    }
}

impl From<DeclarativeRule> for generated::common::DeclarativeRule {
    fn from(d: DeclarativeRule) -> Self {
        let (premise, target) = match d {
            DeclarativeRule::CC { premise, condition } => {
                let cc = generated::common::AtomicCondition::from(condition);
                (
                    premise.map(Into::into),
                    Some(generated::common::declarative_rule::Target::Cc(cc)),
                )
            }
            DeclarativeRule::CT { premise, condition } => {
                let ct = generated::common::Condition::from(condition);
                (
                    premise.map(Into::into),
                    Some(generated::common::declarative_rule::Target::Ct(ct)),
                )
            }
        };

        generated::common::DeclarativeRule {
            premise: premise,
            target: target,
        }
    }
}

impl TryFrom<generated::common::Action> for Action {
    type Error = String;

    fn try_from(proto: generated::common::Action) -> Result<Self, Self::Error> {
        match proto.kind.ok_or("Missing Action.kind")? {
            generated::common::action::Kind::Primitive(p) => {
                let primitive = PrimitiveEvent::try_from(p)?;
                Ok(Action::Primitive(primitive))
            }
            generated::common::action::Kind::List(list) => {
                let list = ActionList::try_from(list)?;
                Ok(Action::List(list))
            }
        }
    }
}

impl From<Action> for generated::common::Action {
    fn from(a: Action) -> Self {
        use generated::common::action::Kind;

        let kind = match a {
            Action::Primitive(p) => Kind::Primitive(p.into()),
            Action::List(list) => Kind::List(list.into()),
        };

        generated::common::Action { kind: Some(kind) }
    }
}

impl TryFrom<generated::common::ActionList> for ActionList {
    type Error = String;

    fn try_from(list: generated::common::ActionList) -> Result<Self, Self::Error> {
        use generated::common::action_list::Kind;

        match list.kind.ok_or("Missing ActionList.kind")? {
            Kind::Sequence(s) => {
                let items = s
                    .actions
                    .into_iter()
                    .map(Action::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ActionList::Sequence(items))
            }
            Kind::Parallel(p) => {
                let items = p
                    .actions
                    .into_iter()
                    .map(Action::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ActionList::Parallel(items))
            }
            Kind::Alternative(a) => {
                let items = a
                    .actions
                    .into_iter()
                    .map(Action::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ActionList::Alternative(items))
            }
        }
    }
}

impl From<ActionList> for generated::common::ActionList {
    fn from(list: ActionList) -> Self {
        use generated::common::action_list::Kind;

        let kind = match list {
            ActionList::Sequence(items) => Kind::Sequence(generated::common::SequenceAction {
                actions: items.into_iter().map(Into::into).collect(),
            }),
            ActionList::Parallel(items) => Kind::Parallel(generated::common::ParallelAction {
                actions: items.into_iter().map(Into::into).collect(),
            }),
            ActionList::Alternative(items) => {
                Kind::Alternative(generated::common::AlternativeAction {
                    actions: items.into_iter().map(Into::into).collect(),
                })
            }
        };

        generated::common::ActionList { kind: Some(kind) }
    }
}

impl TryFrom<generated::common::PrimitiveEvent> for PrimitiveEvent {
    type Error = String;

    fn try_from(p: generated::common::PrimitiveEvent) -> Result<Self, Self::Error> {
        match p.kind.ok_or("Missing PrimitiveEvent.kind")? {
            generated::common::primitive_event::Kind::Trigger(id) => {
                Ok(PrimitiveEvent::Trigger(id))
            }
            generated::common::primitive_event::Kind::Production(ac) => {
                Ok(PrimitiveEvent::Production(AtomicCondition::try_from(ac)?))
            }
            generated::common::primitive_event::Kind::Consumption(ac) => {
                Ok(PrimitiveEvent::Consumption(AtomicCondition::try_from(ac)?))
            }
        }
    }
}

impl From<PrimitiveEvent> for generated::common::PrimitiveEvent {
    fn from(e: PrimitiveEvent) -> Self {
        use generated::common::primitive_event::Kind;

        let kind = match e {
            PrimitiveEvent::Trigger(id) => Kind::Trigger(id),
            PrimitiveEvent::Production(ac) => Kind::Production(ac.into()),
            PrimitiveEvent::Consumption(ac) => Kind::Consumption(ac.into()),
        };

        generated::common::PrimitiveEvent { kind: Some(kind) }
    }
}

impl TryFrom<generated::common::AtomicCondition> for AtomicCondition {
    type Error = String;

    fn try_from(p: generated::common::AtomicCondition) -> Result<Self, Self::Error> {
        match p.kind.ok_or("Missing AtomicCondition.kind")? {
            generated::common::atomic_condition::Kind::Primitive(pcond) => Ok(
                AtomicCondition::Primitive(PrimitiveCondition::Var(pcond.var_name)),
            ),
            generated::common::atomic_condition::Kind::Compound(comp) => {
                Ok(AtomicCondition::Compound(Compound::try_from(comp)?))
            }
            generated::common::atomic_condition::Kind::Sub(sub) => {
                let cond =
                    AtomicCondition::try_from(*sub.condition.ok_or("Missing sub.condition")?)?;
                Ok(AtomicCondition::SubCompound {
                    namespace: sub.namespace,
                    condition: Box::new(cond),
                })
            }
        }
    }
}

impl From<AtomicCondition> for generated::common::AtomicCondition {
    fn from(ac: AtomicCondition) -> Self {
        use generated::common::atomic_condition::Kind;

        let kind = match ac {
            AtomicCondition::Primitive(PrimitiveCondition::Var(name)) => {
                Kind::Primitive(generated::common::PrimitiveCondition { var_name: name })
            }
            AtomicCondition::Compound(comp) => Kind::Compound(comp.into()),
            AtomicCondition::SubCompound {
                namespace,
                condition,
            } => Kind::Sub(Box::new(generated::common::SubCompound {
                namespace,
                condition: Some(Box::new((*condition).into())),
            })),
        };

        generated::common::AtomicCondition { kind: Some(kind) }
    }
}

impl TryFrom<generated::common::Condition> for Condition {
    type Error = String;

    fn try_from(p: generated::common::Condition) -> Result<Self, Self::Error> {
        use generated::common::condition::Kind;

        match p.kind.ok_or("Missing Condition.kind")? {
            Kind::Atomic(ac) => Ok(Condition::Atomic(AtomicCondition::try_from(ac)?)),
            Kind::Not(sub) => Ok(Condition::Not(Box::new(Condition::try_from(*sub)?))),
            Kind::Conjunction(c) => {
                let items = c
                    .conditions
                    .into_iter()
                    .map(Condition::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Condition::Conjunction(items))
            }
            Kind::Disjunction(d) => {
                let items = d
                    .conditions
                    .into_iter()
                    .map(Condition::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Condition::Disjunction(items))
            }
            Kind::Parentheses(inner) => Ok(Condition::Parentheses(Box::new(Condition::try_from(
                *inner,
            )?))),
        }
    }
}

impl From<Condition> for generated::common::Condition {
    fn from(c: Condition) -> Self {
        use generated::common::condition::Kind;

        let kind = match c {
            Condition::Atomic(ac) => Kind::Atomic(ac.into()),
            Condition::Not(inner) => Kind::Not(Box::new((*inner).into())),
            Condition::Conjunction(items) => Kind::Conjunction(generated::common::Conjunction {
                conditions: items.into_iter().map(Into::into).collect(),
            }),
            Condition::Disjunction(items) => Kind::Disjunction(generated::common::Disjunction {
                conditions: items.into_iter().map(Into::into).collect(),
            }),
            Condition::Parentheses(inner) => Kind::Parentheses(Box::new((*inner).into())),
        };

        generated::common::Condition { kind: Some(kind) }
    }
}

impl TryFrom<generated::common::PrimitiveCondition> for PrimitiveCondition {
    type Error = String;

    fn try_from(p: generated::common::PrimitiveCondition) -> Result<Self, Self::Error> {
        Ok(PrimitiveCondition::Var(p.var_name))
    }
}

impl From<PrimitiveCondition> for generated::common::PrimitiveCondition {
    fn from(p: PrimitiveCondition) -> Self {
        match p {
            PrimitiveCondition::Var(name) => {
                generated::common::PrimitiveCondition { var_name: name }
            }
        }
    }
}

impl TryFrom<generated::common::SubCompound> for AtomicCondition {
    type Error = String;

    fn try_from(p: generated::common::SubCompound) -> Result<Self, Self::Error> {
        let condition = Box::new(AtomicCondition::try_from(
            *p.condition.ok_or("Missing condition in SubCompound")?,
        )?);

        Ok(AtomicCondition::SubCompound {
            namespace: p.namespace,
            condition,
        })
    }
}

impl From<AtomicCondition> for generated::common::SubCompound {
    fn from(ac: AtomicCondition) -> Self {
        match ac {
            AtomicCondition::SubCompound {
                namespace,
                condition,
            } => generated::common::SubCompound {
                namespace,
                condition: Some(Box::new((*condition).into())),
            },
            _ => panic!("Called generated::common::SubCompound::from on non-SubCompound variant"),
        }
    }
}

impl TryFrom<generated::common::Compound> for Compound {
    type Error = String;

    fn try_from(p: generated::common::Compound) -> Result<Self, Self::Error> {
        let rules = p
            .rules
            .into_iter()
            .map(Rule::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        let alias = if p.alias.is_empty() {
            None
        } else {
            Some(p.alias)
        };

        Ok(Compound { rules, alias })
    }
}

impl From<Compound> for generated::common::Compound {
    fn from(c: Compound) -> Self {
        generated::common::Compound {
            rules: c.rules.into_iter().map(Into::into).collect(),
            alias: c.alias.unwrap_or_default(),
        }
    }
}
