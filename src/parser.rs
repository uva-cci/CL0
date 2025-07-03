use crate::ast::*;
use crate::token::Token;
use chumsky::input::ValueInput;
use chumsky::{Parser, error::Rich, prelude::*};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

/// A Parser for actions in the CL0 language.
/// This parser currently handles:
/// - Primitive actions: `#event`, `+event`, `-event`
/// (STILL DEBUGGING) - Action sequences: `a; b; c`, `a, b, c`, `a par b par c`, `a seq b seq c`, `a alt b alt c`
pub fn action_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Action<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    // Primitive Event:
    let primitive_event_action = primitive_event_parser::<I>()
        .map_with(|(pe, _), span| (Action::Primitive(pe), span.span()))
        .labelled("primitive action");

    // Action Sequence:
    recursive(|action| {        // Causing an infinite loop here, need to fix
        // Parallel: a, b, c    or    a par b par c
        let parallel = action
            .clone()
            .separated_by(just(Token::Comma).or(just(Token::Par)))
            .at_least(2)
            .collect::<Vec<_>>()
            .map_with(|actions, span| {
                (
                    Action::List(ActionList::Parallel(
                        actions.into_iter().map(|(a, _)| a).collect(),
                    )),
                    span.span(),
                )
            })
            .labelled("parallel action");

        // Sequence: a; b; c    or    a seq b seq c
        let sequence = parallel
            .clone()
            .separated_by(just(Token::Semicolon).or(just(Token::Seq)))
            .at_least(2)
            .collect::<Vec<_>>()
            .map_with(|actions, span| {
                (
                    Action::List(ActionList::Sequence(
                        actions.into_iter().map(|(a, _)| a).collect(),
                    )),
                    span.span(),
                )
            })
            .labelled("sequence action");

        // Alternative: a alt b alt c
        let alternate = sequence
            .clone()
            .separated_by(just(Token::Alt))
            .at_least(2)
            .collect::<Vec<_>>()
            .map_with(|actions, span| {
                
                (
                    Action::List(ActionList::Alternative(
                        actions.into_iter().map(|(a, _)| a).collect(),
                    )),
                    span.span(),
                )
            })
            .labelled("sequence action");

        alternate.or(sequence).or(parallel).or(primitive_event_action).labelled("action")
    })
}

/// A Parser for primitive events in the CL0 language.
/// This parser currently handles:
///
pub fn primitive_event_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<PrimitiveEvent<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
+ Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let descriptor = select! { Token::Descriptor(name) => name }.labelled("descriptor");

    // Trigger Event:
    let trigger = just(Token::Hash)
        .ignore_then(descriptor.clone())
        .map_with(|name, span| (PrimitiveEvent::Trigger(name), span.span()))
        .labelled("trigger action");

    // Production Event:
    let production = just(Token::Plus)
        .ignore_then(descriptor.clone())
        .map_with(|name, span| {
            (
                PrimitiveEvent::Production(PrimitiveCondition::Var(name)),
                span.span(),
            )
        })
        .labelled("production action");

    // Consumption Event:
    let consumption = just(Token::Minus)
        .ignore_then(descriptor.clone())
        .map_with(|name, span| {
            (
                PrimitiveEvent::Consumption(PrimitiveCondition::Var(name)),
                span.span(),
            )
        })
        .labelled("consumption action");

    trigger
        .or(production)
        .or(consumption)
        .labelled("primitive event")
}

/// A Parser for primitive conditions in the CL0 language.
/// This parser currently handles:
/// - Primitive conditions: just an identifier (e.g., `foo`, `bar`)
pub fn primitive_condition_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<
    'tokens,
    I,
    Spanned<PrimitiveCondition<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let ident = select! { Token::Descriptor(name) => name }.labelled("descriptor");

    // Primitive condition: just an identifier
    ident
        .map_with(|name, span| (PrimitiveCondition::Var(name), span.span()))
        .labelled("primitive condition")
}



// / A parser for rules in the CL0 language.
// / This parser currently handles:
// /
// pub fn rule_parser<'tokens, 'src: 'tokens, I>()
// -> impl Parser<'tokens, I, Vec<Spanned<Rule<'src>>>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
// + Clone
// where
//     I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
// {
//     let ident = select! { Token::Identifier(name) => name }.labelled("identifier");

// // Event declaration: #name : condition => action
// let event_op = just(Token::Hash)
//     .ignore_then(ident.clone())
//     .map(|n| EventOp::Hash(n))
//     .or(just(Token::Plus)
//         .ignore_then(ident.clone())
//         .map(|n| EventOp::Plus(n)))
//     .or(just(Token::Minus)
//         .ignore_then(ident.clone())
//         .map(|n| EventOp::Minus(n)))
//     .or_not();

// let cond = just(Token::Colon)
//     .ignore_then(condition_parser::<I>())
//     .or_not();

// let event_decl = event_op
//     .then(cond)
//     .then_ignore(just(Token::FatArrow))
//     .then(action_parser::<I>())
//     .then_ignore(just(Token::Dot))
//     .map_with(|((event_op, cond), (action, _)), span| {
//         (
//             Declaration::Event {
//                 event_type: event_op,
//                 condition: cond.map(|(c, _)| c),
//                 action: action,
//             },
//             span.span(),
//         )
//     })
//     .labelled("event declaration");

// // Unary declaration: -o name . or -> name .
// let unary_decl = just(Token::DashO)
//     .ignore_then(ident.clone())
//     .map(|n| UnaryOp::DashO(n))
//     .or(just(Token::ThinArrow)
//         .ignore_then(ident.clone())
//         .map(|n| UnaryOp::ThinArrow(n)))
//     .then_ignore(just(Token::Dot))
//     .map_with(|op, span| (Declaration::Unary { op: op }, span.span()))
//     .labelled("unary declaration");

// // Declarative declaration: premise -> action .
// let declarative_decl = ident
//     .then_ignore(just(Token::ThinArrow))
//     .then(action_parser::<I>())
//     .then_ignore(just(Token::Dot))
//     .map_with(|(premise, (action, _)), span| {
//         (
//             Declaration::Declarative {
//                 premise: premise,
//                 action: action,
//             },
//             span.span(),
//         )
//     })
//     .labelled("declarative declaration");

// // One declaration that needs to be repeated
// let single_decl = event_decl.or(unary_decl).or(declarative_decl);

// single_decl
//     .repeated()
//     .collect::<Vec<_>>()
//     .map_with(|decls, span| {
//         let mut result = Vec::new();
//         for (decl, decl_span) in decls {
//             result.push((decl, decl_span));
//         }
//         result
//     })
// }

// / A Parser for conditions in the CL0 language.
// / This parser currently handles:
// / - Variable conditions: `x`, `y`, etc.
// / - Conjunction: `x and y`, `x, y`
// / - Disjunction: `x or y`, `x, y`
// / - Parentheses for grouping: `(x and y)`, `(x or y)`,
// / - Negation: `not x`


// fn condition_parser<'tokens, 'src: 'tokens, I>()
// -> impl Parser<'tokens, I, Spanned<Condition<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
// + Clone
// where
//     I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
// {
//     recursive(|condition| {
//         let ident = select! { Token::Identifier(name) => name }.labelled("identifier");

//         // Parentheses: (cond)
//         let paren = condition
//             .clone()
//             .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis))
//             .labelled("parenthesized condition");

//         // Atom: identifier or parenthesized
//         let atom = ident
//             .map_with(|name, span| (Condition::Var(name), span.span()))
//             .or(paren);

//         // Not: not <atom or not>
//         let not = just(Token::Not)
//             .ignore_then(condition.clone())
//             .map_with(|(cond, span), not_span| (Condition::Not(Box::new(cond)), not_span.span()))
//             .labelled("negation");

//         // Highest precedence: not or atom
//         let primary = not.or(atom);

//         // Conjunction: a and b or a, b
//         let and = primary
//             .clone()
//             .separated_by(just(Token::And).or(just(Token::Comma)))
//             .collect::<Vec<_>>()
//             .map_with(|conds, span| {
//                 (
//                     Condition::And(conds.into_iter().map(|(a, _)| a).collect()),
//                     span.span(),
//                 )
//             })
//             .labelled("conjunction condition");

//         // Disjunction: a or b
//         let or = and
//             .clone()
//             .separated_by(just(Token::Or))
//             .collect::<Vec<_>>()
//             .map_with(|conds, span| {
//                 if conds.len() == 1 {
//                     conds.into_iter().next().unwrap()
//                 } else {
//                     (
//                         Condition::Or(conds.into_iter().map(|(a, _)| a).collect()),
//                         span.span(),
//                     )
//                 }
//             })
//             .labelled("disjunction condition");

//         or
//     })
// }


// // Handle #ident
// let trigger = just(Token::Hash)
//     .ignore_then(ident.clone())
//     .map_with(|name, span| (Action::Trigger(name), span.span()))
//     .labelled("trigger event action");

// // Handle +ident
// let production = just(Token::Plus)
//     .ignore_then(ident.clone())
//     .map_with(|name, span| (Action::Production(name), span.span()))
//     .labelled("production action");

// // Handle -ident
// let consumption = just(Token::Minus)
//     .ignore_then(ident.clone())
//     .map_with(|name, span| (Action::Consumption(name), span.span()))
//     .labelled("consumption action");

// let atomic = trigger.or(production).or(consumption);

// // Bracketed event declaration block: { ... }
// let block = just(Token::LeftCBracket)
//     .ignore_then(declaration_parser::<I>())
//     .then_ignore(just(Token::RightCBracket))
//     .map_with(|decls, span| {
//         (
//             Action::CBrackets(decls.into_iter().map(|(decl, _)| decl).collect()),
//             span.span(),
//         )
//     })
//     .labelled("new event declaration block");

// // Recursive parser for delimited lists
// recursive(|action| {
//     // Parallel: a, b, c
//     let parallel = action
//         .clone()
//         .separated_by(just(Token::Comma))
//         .at_least(1)
//         .allow_trailing()
//         .collect::<Vec<_>>()
//         .map_with(|actions, span| {
//             (
//                 Action::Parallel(actions.into_iter().map(|(a, _)| a).collect()),
//                 span.span(),
//             )
//         })
//         .labelled("parallel action");

//     // Sequence: a; b; c
//     let sequence = action
//         .clone()
//         .separated_by(just(Token::Semicolon))
//         .at_least(1)
//         .allow_trailing()
//         .collect::<Vec<_>>()
//         .map_with(|actions, span| {
//             (
//                 Action::Sequence(actions.into_iter().map(|(a, _)| a).collect()),
//                 span.span(),
//             )
//         })
//         .labelled("sequence action");

//     // seq(a b c)
//     let seq_func = just(Token::Seq)
//         .ignore_then(
//             action
//                 .clone()
//                 .repeated()
//                 .at_least(1)
//                 .collect::<Vec<_>>()
//                 .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis))
//         )
//         .map_with(|actions, span| {
//             (
//                 Action::Sequence(actions.into_iter().map(|(a, _)| a).collect()),
//                 span.span(),
//             )
//         })
//         .labelled("sequence function action");

//     // par(a b c)
//     let par_func = just(Token::Par)
//         .ignore_then(
//             action
//                 .clone()
//                 .repeated()
//                 .at_least(1)
//                 .collect::<Vec<_>>()
//                 .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis))
//         )
//         .map_with(|actions, span| {
//             (
//                 Action::Parallel(actions.into_iter().map(|(a, _)| a).collect()),
//                 span.span(),
//             )
//         })
//         .labelled("parallel function action");

//     // alt(a b c)
//     let alt_func = just(Token::Alt)
//         .ignore_then(
//             action
//                 .clone()
//                 .repeated()
//                 .at_least(1)
//                 .collect::<Vec<_>>()
//                 .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis))
//         )
//         .map_with(|actions, span| {
//             (
//                 Action::Alternative(actions.into_iter().map(|(a, _)| a).collect()),
//                 span.span(),
//             )
//         })
//         .labelled("alternate function action");

//     // Try to parse parallel or sequence first, then fallback to atomic
//     block
//         .or(parallel)
//         .or(sequence)
//         .or(atomic)
//         .or(seq_func)
//         .or(par_func)
//         .or(alt_func)
//         .labelled("action")
// })
