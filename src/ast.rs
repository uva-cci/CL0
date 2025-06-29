/// Logical condition type used in event declarations.
/// EBNF Grammar::
/// ```
/// Condition   ::= "not" Condition
///              | Condition ( ("and" | ',' | "or" ) Condition )*
///              | "(" Condition ")"
///              | Identifier
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Condition<'src> {
    /// A plain variable (e.g., `loaded`)
    Var(&'src str),
    /// Logical negation (e.g., `not loaded`)
    Not(Box<Self>),
    /// A conjunction (AND) of two conditions (e.g., `loaded AND ready`)
    And(Vec<Self>),
    /// A disjunction (OR) of two conditions (e.g., `loaded OR ready`)
    Or(Vec<Self>),
    /// Parenthesized condition (e.g., `(loaded AND ready)`)
    Parentheses(Box<Self>),
}

/// Actions that can be triggered when an event fires.
/// EBNF Grammar::
/// ```
/// Action   ::= ( '+' | '-' | '#' ) Identifier
///           | ( 'seq' | 'par' | 'alt' ) Action+
///           | '{' EventDecl '}'
///           | Action ( ("," | ";" ) Action )*
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action<'src> {
    /// Enables a condition variable (e.g., `+loaded`)
    Production(&'src str),
    /// Disables a condition variable (e.g., `-loaded`)
    Consumption(&'src str),
    /// Triggers another event (e.g., `#fail`)
    Trigger(&'src str),
    /// A sequence of actions (e.g., `seq a b` or `a; b`)
    Sequence(Vec<Self>),
    /// A parallel execution of actions (e.g., `par a b` or `a, b`)
    Parallel(Vec<Self>),
    /// An alternative choice of actions (e.g., `alt a b`)
    Alternative(Vec<Self>),
    /// A block of event declarations (e.g., `{ #click => +clicked }`)
    CBrackets(Vec<Declaration<'src>>), // { EventDecl }
}

/// An event declaration parsed from source code.
/// EBNF Grammar::
/// ```
/// EventDecl   ::= ( ( '#' | '+' | '-' ) Identifier )? ( ':' Condition )? '=>' Action '.'
///              | ( '-o' | '->' ) Identifier '.'
///              | Identifier '->' Action '.'
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventOp<'src> {
    Hash(&'src str),    // #Identifier
    Plus(&'src str),    // +Identifier
    Minus(&'src str),   // -Identifier
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp<'src> {
    DashO(&'src str),     // -o
    ThinArrow(&'src str), // ->
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration<'src> {
    Event {
        event_type: Option<EventOp<'src>>,
        condition: Option<Condition<'src>>,
        action: Action<'src>,
    },
    Unary {
        op: UnaryOp<'src>,
    },
    Declarative {
        premise: &'src str,
        action: Action<'src>,
    },
}
