/// Logical condition type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition<'src> {
    /// A plain variable (e.g., `loaded`)
    Atomic(AtomicCondition<'src>),
    /// Logical negation (e.g., `not loaded`)
    Not(Box<Self>),
    /// A conjunction (AND) of two conditions (e.g., `loaded AND ready`)
    Conjunction(Vec<Self>),
    /// A disjunction (OR) of two conditions (e.g., `loaded OR ready`)
    Disjunction(Vec<Self>),
    /// Parenthesized condition (e.g., `(loaded AND ready)`)
    Parentheses(Box<Self>),
}

/// A primitive condition is a basic variable or identifier used in conditions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveCondition<'src> {
    Var(&'src str),
}

/// An atomic condition can be either a compound condition or a primitive condition.
#[derive(Debug, Clone, PartialEq, Eq)]

pub enum AtomicCondition<'src> {
    Compound(Compound<'src>),
    Primitive(PrimitiveCondition<'src>),
}

/// A sequence of actions can be a sequence, parallel, or alternative execution.    
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionList<'src> {
    /// A sequence of actions (e.g., `seq a b` or `a; b`)
    Sequence(Vec<Action<'src>>),
    /// A parallel execution of actions (e.g., `par a b` or `a, b`)
    Parallel(Vec<Action<'src>>),
    /// An alternative choice of actions (e.g., `alt a b`)
    Alternative(Vec<Action<'src>>),
}

/// Represents a primitive event, which can be a trigger, production, or consumption event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveEvent<'src> {
    Trigger(&'src str),                     // #Identifier
    Production(PrimitiveCondition<'src>),   // +Identifier
    Consumption(PrimitiveCondition<'src>),  // -Identifier
}

/// Represents an action, which can be a primitive event or a sequence of actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<'src> {
    /// A single event action, like `#click`, `+load`, or `-submit`
    Primitive(PrimitiveEvent<'src>),
    /// A sequence of actions, like `seq a seq b seq c` seq(test, test2, par(test3, test4), etc)
    List(ActionList<'src>),
}

/// Represents a reactive rule, which can be either an ECA (Event-Condition-Action) or CA (Condition-Action).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReactiveRule<'src> {
    ECA {
        event: PrimitiveEvent<'src>,
        condition: Option<Condition<'src>>,
        action: Action<'src>,
    },
    CA {
        condition: Condition<'src>,
        action: Action<'src>,
    }
}

/// Represents a declarative rule, which can be either a CC (->) or CT (-o).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarativeRule<'src> {
    CC {
        premise: Option<Condition<'src>>,
        condition: AtomicCondition<'src>,
    },
    CT {
        premise: Option<Condition<'src>>,
        condition: Condition<'src>,
    }
}

/// Represents a rule in the system, which can be reactive, declarative, case-based, or fact-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rule<'src> {
    Reactive(ReactiveRule<'src>),
    Declarative(DeclarativeRule<'src>),
    Case {
        action: Action<'src>,
    },
    Fact {
        condition: AtomicCondition<'src>,
    }
}

// Represents a compound rule, which can contain multiple rules and an optional alias to refer to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compound<'src> {
    pub rules: Vec<Rule<'src>>,
    pub alias: Option<&'src str>,
}
