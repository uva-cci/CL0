/// A token in the source language.
/// This is used after lexing to distinguish keywords, operators, identifiers, and literals.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token<'src> {
    /// Symbol for an event, like `#click`
    Hash,
    /// Symbol for defining an event
    Colon,

    /// Symbol for sequencing events, eg. `<event>; <event>`
    Semicolon,
    /// `seq` literal for sequencing events, eg. `seq <event> <event> ...`
    Seq,

    /// `par` literal for parallel branching, eg. `par <event> <event> ...`
    Par,

    /// `alt` literal for alternative branching, eg. `alt <event> <event> ...`
    Alt,

    /// Symbol for starting a condition grouping, eg. `(<condition>)`
    LeftParenthesis,
    /// Symbol for ending a condition grouping, eg. `(<condition>)`
    RightParenthesis,
    
    /// Symbol for starting a event grouping, eg. `{<event>}`
    LeftCBracket,
    /// Symbol for ending a event grouping, eg. `{<event>}`
    RightCBracket,

    /// Symbol for conjunction or parallel branching, eg. `<condition>, <condition>`
    Comma,
    /// `and` literal conjunction, eg. `<condition> and <condition>`
    And,

    /// `or` literal disjunction, eg. `<condition> or <condition>`
    Or,

    /// `not` literal negation, eg. `not <condition>`
    Not,

    /// Symbol for production events
    Plus,
    /// Symbol for consumption events
    Minus,

    /// Symbol for a dot, used to end a line
    Dot,

    /// Represents `-o` for negating a condition
    DashO,

    /// Represents a fat arrow `=>`
    FatArrow,

    /// Represents a thin arrow `->`
    ThinArrow,

    /// A variable/function name, like `foo` or `my_var`
    Descriptor(&'src str),
}