use std::collections::HashMap;

use crate::ast::{Action, Condition, Declaration, EventOp, UnaryOp};
use crate::token::Token;
use ariadne::{Color, Label, Report, ReportKind, sources};
use chumsky::container::Seq;
use chumsky::input::{BorrowInput, ValueInput};
use chumsky::{
    Parser,
    error::Rich,
    input::Input, // for the Input bound
    prelude::*,
};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

/// The parser signature:
/// - Input: any type `I` over spanned tokens
/// - Output: one `Declaration` with its span
/// - Error: `Rich` errors for Ariadne
// pub fn declaration_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<
//     'tokens,
//     I,
//     Vec<Declaration<'src>>,
//     extra::Err<Rich<'tokens, Token<'src>, Span>>,
// > + Clone
// where
//     I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
// {
//     // Helper: match identifiers and capture their span
//     let ident = select! { Token::Identifier(name) => name }.labelled("identifier");

//     // Recursive parser for conditions.
//     let condition = recursive(|cond| {
//         let var = ident.clone().map(|(n, _)| Condition::Var(n));
//         // let paren = cond
//         //     .clone()
//         //     .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis))
//         //     .map(Condition::Parentheses);
//         // let not_p = just(Token::Not)
//         //     .ignore_then(cond.clone())
//         //     .map(Condition::Not);
//         // let atom = var.or(paren).or(not_p);
//         // atom.clone()
//         //     .then(
//         //         just(Token::And)
//         //             .to(true)
//         //             .or(just(Token::Or).to(false))
//         //             .then(cond.clone())
//         //             .repeated(),
//         //     )
//         //     .foldl(|lhs, (is_and, rhs)| {
//         //         if is_and {
//         //             Condition::And(vec![lhs, rhs])
//         //         } else {
//         //             Condition::Or(vec![lhs, rhs])
//         //         }
//         //     })
//         var
//     });

//     // Recursive parser for actions.
//     let action = recursive(|action| {
//         // Primary actions: +x, -x, #x
//         let prod = just(Token::Plus)
//             .ignore_then(ident.clone())
//             .map(|(n, _)| Action::Production(n));
//         let cons = just(Token::Negative)
//             .ignore_then(ident.clone())
//             .map(|(n, _)| Action::Consumption(n));
//         let trig = just(Token::Hash)
//             .ignore_then(ident.clone())
//             .map(|(n, _)| Action::Trigger(n));
//         // Block: { Declaration }
//         let block = just(Token::LeftCBracket)
//             .ignore_then(recursive(|decl| decl.clone()))
//             .then_ignore(just(Token::RightCBracket))
//             .map(|decl| Action::CBrackets(Box::new(decl)));
//         // seq, par, alt keywords
//         // let seq_kw = just(Token::Seq)
//         //     .ignore_then(action.clone())
//         //     .repeated()
//         //     .until(just(Token::Dot))
//         //     .at_least(1)
//         //     .map(Action::Sequence);
//         // let par_kw = just(Token::Par)
//         //     .ignore_then(action.clone().repeated().at_least(1))
//         //     .map(|v: Vec<_>| Action::Parallel(v));
//         // let alt_kw = just(Token::Alt)
//         //     .ignore_then(action.clone().repeated().at_least(1))
//         //     .map(|v: Vec<_>| Action::Alternative(v));
//         let primary = prod
//             .or(cons)
//             .or(trig)
//             .or(block);
//             // .or(seq_kw)
//             // .or(par_kw)
//             // .or(alt_kw);
//         // Delimited lists: comma = parallel, semicolon = sequence
//         primary
//             .clone()
//             .then(
//                 just(Token::Comma)
//                     .to(true)
//                     .or(just(Token::Semicolon).to(false))
//                     .then(primary.clone())
//                     .repeated(),
//             )
//             .map(|(first, rest)| {
//                 if rest.is_empty() {
//                     first
//                 } else {
//                     let is_parallel = rest[0].0;
//                     let mut items = Vec::with_capacity(1 + rest.len());
//                     items.push(first);
//                     for (_, act) in rest {
//                         items.push(act);
//                     }
//                     if is_parallel {
//                         Action::Parallel(items)
//                     } else {
//                         Action::Sequence(items)
//                     }
//                 }
//             })
//     });

//     // Prefix operators for declarations: #x, +x, -x
//     let event_op = just(Token::Hash)
//         .ignore_then(ident.clone())
//         .map(|(n, _)| EventOp::Hash(n))
//         .or(just(Token::Plus)
//             .ignore_then(ident.clone())
//             .map(|(n, _)| EventOp::Plus(n)))
//         .or(just(Token::Negative)
//             .ignore_then(ident.clone())
//             .map(|(n, _)| EventOp::Minus(n)));

//     // Event branch: (prefix? name) (: cond)? => action .
//     let event_decl = event_op
//         .or_not()
//         .then(ident.clone().map(|(n, _)| n))
//         .then_ignore(just(Token::Colon).ignore_then(condition.clone()).or_not())
//         .then_ignore(just(Token::FatArrow))
//         .then(action.clone())
//         .then_ignore(just(Token::Dot))
//         .map_with(|(opt_op, cond), span| {
//             (
//                 Declaration::Event {
//                     event_type: opt_op,
//                     condition: cond,
//                     action: ((opt_op, name), cond),
//                 },
//                 span,
//             )
//         });

//     // Unary branch: -o name . or -> name .
//     let unary_decl = just(Token::DashO)
//         .ignore_then(ident.clone())
//         .map(|(n, _)| UnaryOp::DashO(n))
//         .or(just(Token::ThinArrow)
//             .ignore_then(ident.clone())
//             .map(|(n, _)| UnaryOp::ThinArrow(n)))
//         .then_ignore(just(Token::Dot))
//         .map_with(|op, span| (Declaration::Unary { op }, span));

//     // Declarative branch: name -> action .
//     let declarative = ident
//         .clone()
//         .map(|(n, _)| n)
//         .then_ignore(just(Token::ThinArrow))
//         .then(action.clone())
//         .then_ignore(just(Token::Dot))
//         .map_with(|(premise, act), span| {
//             (
//                 Declaration::Declarative {
//                     premise: premise,
//                     action: act,
//                 },
//                 span.span(),
//             )
//         });

//     // Combine all branches
//     event_decl.or(unary_decl).or(declarative)
// }

// /// A Parser for conditions in the CL0 language.
// /// This parser currently handles:
// /// - Variable conditions: `x`, `y`, etc.
// /// - Conjunction: `x and y`, `x, y`
// /// - Disjunction: `x or y`, `x, y`
// /// - Parentheses for grouping: `(x and y)`, `(x or y)`,
// /// - Negation: `not x`
// /// 
// fn condition_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<
//     'tokens,
//     I,
//     Spanned<Condition<'src>>,
//     extra::Err<Rich<'tokens, Token<'src>, Span>>,
// > + Clone
// where
//     I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
// {
//     let ident = select! { Token::Identifier(name) => name }.labelled("identifier");

//     recursive(|condition| {
//         // Negation: not x
//         let not = just(Token::Not).ignore_then(condition.clone())
//             .ignore_then(ident.clone())
//             .map_with(|name, span| (Condition::Not(Box::new(Condition::Var(name))), span.span()))
//             .labelled("negation condition");

    

//         ident.map_with(|name, span| {
//             (Condition::Var(name), span.span())
//         })
//     })
// }

/// A Parser for actions in the CL0 language.
/// This parser currently handles:
/// - Atomic actions: `#event`, `+event`, `-event`
/// - Block event declaration actions: `{ ... }`
/// - Parallel actions: `a, b, c`
/// - Sequence actions: `a; b; c`
pub fn action_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<
    'tokens,
    I,
    Spanned<Action<'src>>,
    extra::Err<Rich<'tokens, Token<'src>, Span>>,
> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
{
    let ident = select! { Token::Identifier(name) => name }.labelled("identifier");

    // Handle #ident
    let trigger = just(Token::Hash)
        .ignore_then(ident.clone())
        .map_with(|name, span| (Action::Trigger(name), span.span())).labelled("trigger event action");

    // Handle +ident
    let production = just(Token::Plus)
        .ignore_then(ident.clone())
        .map_with(|name, span| (Action::Production(name), span.span())).labelled("production action");

    // Handle -ident
    let consumption = just(Token::Minus)
        .ignore_then(ident.clone())
        .map_with(|name, span| (Action::Consumption(name), span.span())).labelled("consumption action");

    let atomic = trigger.or(production).or(consumption);

    // // Bracketed event declaration block: { ... }
    // let block = just(Token::LeftCBracket)
    //     .ignore_then(declaration_parser::<I>())
    //     .then_ignore(just(Token::RightCBracket))
    //     .map_with(|decls, span| (Action::CBrackets(decls), span.span())).labelled("new event declaration block");

    // Recursive parser for delimited lists
    recursive(|action| {
        // Parallel: a, b, c
        let parallel = action
            .clone()
            .separated_by(just(Token::Comma))
            .at_least(2)
            .collect::<Vec<_>>().map_with(|actions, span| {
                (Action::Parallel(actions.into_iter().map(|(a, _)| a).collect()), span.span())
            }).labelled("parallel action");

        // Sequence: a; b; c
        let sequence = action
            .clone()
            .separated_by(just(Token::Comma))
            .at_least(2)
            .collect::<Vec<_>>().map_with(|actions, span| {
                (Action::Sequence(actions.into_iter().map(|(a, _)| a).collect()), span.span())
            }).labelled("sequence action");

        // Try to parse parallel or sequence first, then fallback to atomic
        // block.or(parallel).or(sequence).or(atomic)
        parallel.or(sequence).or(atomic)
    })
}
