use crate::ast::*;
use crate::token::Token;
use chumsky::input::ValueInput;
use chumsky::{Parser, error::Rich, prelude::*};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

/// A Parser for actions in the CL0 language.
/// This parser currently handles:
/// - Primitive actions: `#event`, `+event`, `-event`
/// - Action sequences: `a; b; c`, `a, b, c`, `a par b par c`, `a seq b seq c`, `a alt b alt c`
pub fn action_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Action>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parser().5
}

/// A Parser for primitive events in the CL0 language.
/// This parser currently handles:
/// - Trigger events: `#event`
/// - Production events: `+event`
/// - Consumption events: `-event`
pub fn primitive_event_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<PrimitiveEvent>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parser().4
}

/// A Parser for conditions in the CL0 language.
/// This parser currently handles:
/// - Atomic conditions: `condition`
/// - Parentheses: `(condition)`
/// - Negation: `not condition`
/// - Conjunction: `condition and condition`, `condition, condition`
/// - Disjunction: `condition or condition`, `condition; condition`
pub fn condition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Condition>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parser().3
}

/// A Parser for atomic conditions in the CL0 language.
/// This parser currently handles:
/// - Primitive conditions: `foo`, `bar`, etc.
/// - Compound conditions: `{ rule1. rule2. }` or `{ rule1. rule2. } as alias`
pub fn atomic_condition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<AtomicCondition>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
+ Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parser().1
}

/// A Parser for compounds in the CL0 language.
/// This parser currently handles:
/// - compounds: `{ rule1. rule2. }` or `{ rule1. rule2. } as alias`
pub fn compound_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Compound>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parser().2
}

/// A Parser for primitive conditions in the CL0 language.
/// This parser currently handles:
/// - Primitive conditions: just an identifier (e.g., `foo`, `bar`)
pub fn primitive_condition_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<PrimitiveCondition>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
+ Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let ident = select! { Token::Descriptor(name) => name }.labelled("descriptor");

    // Primitive condition: just an identifier
    ident
        .map_with(|name, span| (PrimitiveCondition::Var(name.to_string()), span.span()))
        .labelled("primitive condition")
}

/// A parser for rules in the CL0 language.
/// This parser currently handles:
/// - Reactive rules: ECA (Event-Condition-Action) or CA (Condition-Action)
/// - Declarative rules: CC (premise -> condition) or CT (premise -o condition)
/// - Case-based rules: => action
/// - Fact-based rules: condition
pub fn rule_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Rule>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    parser().0
}

fn parser<'tokens, 'src: 'tokens, I>() -> (
    impl Parser<'tokens, I, Spanned<Rule>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
    impl Parser<'tokens, I, Spanned<AtomicCondition>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
    + Clone,
    impl Parser<'tokens, I, Spanned<Compound>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
    impl Parser<'tokens, I, Spanned<Condition>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
    impl Parser<'tokens, I, Spanned<PrimitiveEvent>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
    + Clone,
    impl Parser<'tokens, I, Spanned<Action>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone,
)
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let mut rule_parser = Recursive::declare();
    let mut atomic_condition_parser = Recursive::declare();
    let mut compound_parser = Recursive::declare();
    let mut condition_parser = Recursive::declare();
    let mut primitive_event_parser = Recursive::declare();
    let mut action_parser = Recursive::declare();

    compound_parser.define(
        just(Token::LeftCBracket)
            .ignore_then(rule_parser.clone().repeated().collect::<Vec<_>>())
            .then_ignore(just(Token::RightCBracket))
            .then(
                just(Token::As)
                    .ignore_then(select! { Token::Descriptor(alias) => alias })
                    .or_not(),
            )
            .map_with(|(rules, alias), span| {
                (
                    Compound {
                        rules: (rules.into_iter().map(|(r, _)| r).collect()),
                        alias: alias.map(|a| a.to_string()),
                    },
                    span.span(),
                )
            }),
    );

    // Recursive condition parser for complex conditions
    let c_parser = recursive(|condition| {
        // Atomic condition parser (base case)
        let atomic_condition = atomic_condition_parser
            .clone()
            .map_with(|(cond, _), span| (Condition::Atomic(cond), span.span()))
            .labelled("atomic condition");

        // Parentheses: (cond)
        let parentheses = condition
            .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis))
            .map_with(|(cond, _), span| (Condition::Parentheses(Box::new(cond)), span.span()))
            .labelled("parenthesized condition");

        let primary = atomic_condition.or(parentheses);

        // Not operator: not <condition>
        let not = just(Token::Not)
            .repeated()
            .collect::<Vec<_>>()
            .then(primary.clone())
            .map_with(|(conds, inner), _| {
                conds.into_iter().rev().fold(inner, |acc, _| {
                    let (c, span) = acc;
                    (Condition::Not(Box::new(c)), span)
                })
            })
            .labelled("negate condition");

        let and = not
            .clone()
            .separated_by(just(Token::And).or(just(Token::Comma)))
            .at_least(1)
            .collect::<Vec<_>>()
            .map_with(|mut conds, span| {
                if conds.len() == 1 {
                    conds.pop().unwrap()
                } else {
                    (
                        Condition::Conjunction(conds.into_iter().map(|(a, _)| a).collect()),
                        span.span(),
                    )
                }
            })
            .labelled("conjunction condition");

        let or = and
            .separated_by(just(Token::Or).or(just(Token::Semicolon)))
            .at_least(1)
            .collect::<Vec<_>>()
            .map_with(|mut conds, span| {
                if conds.len() == 1 {
                    conds.pop().unwrap()
                } else {
                    (
                        Condition::Disjunction(conds.into_iter().map(|(a, _)| a).collect()),
                        span.span(),
                    )
                }
            })
            .labelled("disjunction condition");

        or.labelled("condition")
    });

    condition_parser.define(c_parser);

    let ac_parser = recursive(|atomic_condition| {
        // Primitive condition: just an identifier
        let primitive_condition = primitive_condition_parser::<I>()
            .map_with(|(cond, span), _| (AtomicCondition::Primitive(cond), span))
            .labelled("primitive condition");

        // Compound condition: { rule1. rule2. } or { rule1. rule2. } as alias
        let compound_condition = compound_parser
            .clone()
            .map_with(|(compound, span), _| (AtomicCondition::Compound(compound), span))
            .labelled("compound condition");

        // Sub-compound condition: namespace.condition
        let descriptor = select! { Token::Descriptor(name) => name }.labelled("descriptor");

        let sub_compound_condition = descriptor
            .clone()
            .then_ignore(just(Token::Dot))
            .then(atomic_condition.clone())
            .map_with(|(namespace, (ac, span)), _| {
                (
                    AtomicCondition::SubCompound {
                        namespace: namespace.to_string(),
                        condition: Box::new(ac),
                    },
                    span,
                )
            })
            .labelled("sub-compound condition");

        sub_compound_condition
            .or(primitive_condition)
            .or(compound_condition)
            .labelled("atomic condition")
    });

    atomic_condition_parser.define(ac_parser);

    // Reactive rules: ECA or CA
    // ECA rule:    #event : condition => action        #event => action
    let eca_rule = primitive_event_parser
        .clone()
        .then(
            just(Token::Colon)
                .ignore_then(condition_parser.clone())
                .or_not(),
        )
        .then_ignore(just(Token::FatArrow))
        .then(action_parser.clone())
        .then_ignore(just(Token::EndRule))
        .map_with(|(((event, _), cond), (action, _)), span| {
            (
                Rule::Reactive(ReactiveRule::ECA {
                    event,
                    condition: cond.map(|(c, _)| c),
                    action,
                }),
                span.span(),
            )
        })
        .labelled("reactive rule");

    // CA rule:   : condition => action
    let ca_rule = just(Token::Colon)
        .ignore_then(condition_parser.clone())
        .then_ignore(just(Token::FatArrow))
        .then(action_parser.clone())
        .then_ignore(just(Token::EndRule))
        .map_with(|((condition, _), (action, _)), span| {
            (
                Rule::Reactive(ReactiveRule::CA { condition, action }),
                span.span(),
            )
        })
        .labelled("ca rule");

    let reactive_rule = eca_rule.or(ca_rule).labelled("reactive rule");

    // Declarative rules: CC or CT
    // CC rule:    premise -> condition
    let cc_rule = condition_parser
        .clone()
        .or_not()
        .then_ignore(just(Token::ThinArrow))
        .then(atomic_condition_parser.clone())
        .then_ignore(just(Token::EndRule))
        .map_with(|(premise, (condition, _)), span| {
            (
                Rule::Declarative(DeclarativeRule::CC {
                    premise: premise.map(|(c, _)| c),
                    condition,
                }),
                span.span(),
            )
        })
        .labelled("cc rule");

    // CT rule:    premise -o condition
    let ct_rule = condition_parser
        .clone()
        .or_not()
        .then_ignore(just(Token::DashO))
        .then(condition_parser.clone())
        .then_ignore(just(Token::EndRule))
        .map_with(|(premise, (condition, _)), span| {
            (
                Rule::Declarative(DeclarativeRule::CT {
                    premise: premise.map(|(c, _)| c),
                    condition,
                }),
                span.span(),
            )
        })
        .labelled("ct rule");

    let declarative_rule = cc_rule.or(ct_rule).labelled("declarative rule");

    // Case-based rule:     => action
    let case_rule = just(Token::FatArrow)
        .ignore_then(action_parser.clone())
        .then_ignore(just(Token::EndRule))
        .map_with(|(action, _), span| (Rule::Case(CaseRule { action }), span.span()))
        .labelled("case");

    // Fact-based rule:     condition
    let fact_rule = atomic_condition_parser
        .clone()
        .then_ignore(just(Token::EndRule))
        .map_with(|(condition, _), span| (Rule::Fact(FactRule { condition }), span.span()))
        .labelled("fact");

    rule_parser.define(
        reactive_rule
            .or(declarative_rule)
            .or(case_rule)
            .or(fact_rule)
            .labelled("rule"),
    );

    // Primitive Event Parser
    let pe_parser = recursive(|_| {
        let descriptor = select! { Token::Descriptor(name) => name }.labelled("descriptor");

        // Trigger Event:
        let trigger = just(Token::Hash)
            .ignore_then(descriptor.clone())
            .map_with(|name, span| (PrimitiveEvent::Trigger(name.to_string()), span.span()))
            .labelled("trigger action");

        // Production Event:
        let production = just(Token::Plus)
            .ignore_then(atomic_condition_parser.clone())
            .map_with(|(ac, _), span| (PrimitiveEvent::Production(ac), span.span()))
            .labelled("production action");

        // Consumption Event:
        let consumption = just(Token::Minus)
            .ignore_then(atomic_condition_parser.clone())
            .map_with(|(ac, _), span| (PrimitiveEvent::Consumption(ac), span.span()))
            .labelled("consumption action");

        trigger
            .or(production)
            .or(consumption)
            .labelled("primitive event")
    });

    primitive_event_parser.define(pe_parser);

    let a_parser = recursive(|_| {
        // Primitive Event:
        let primitive_event_action = primitive_event_parser
            .clone()
            .map_with(|(pe, _), span| (Action::Primitive(pe), span.span()))
            .labelled("primitive action");

        // Action Sequence:
        // Parallel: a, b, c    or    a par b par c
        let parallel = primitive_event_action
            .clone()
            .separated_by(just(Token::Comma).or(just(Token::Par)))
            .at_least(1)
            .allow_trailing()
            .collect::<Vec<_>>()
            .map_with(|mut actions, span| {
                if actions.len() == 1 {
                    return actions.pop().unwrap();
                }
                (
                    Action::List(ActionList::Parallel(
                        actions.into_iter().map(|(a, _)| a).collect(),
                    )),
                    span.span(),
                )
            })
            .labelled("parallel action");

        // Alternative: a alt b alt c
        let alternate = parallel
            .separated_by(just(Token::Alt))
            .at_least(1)
            .allow_trailing()
            .collect::<Vec<_>>()
            .map_with(|mut actions, span| {
                if actions.len() == 1 {
                    return actions.pop().unwrap();
                }
                (
                    Action::List(ActionList::Alternative(
                        actions.into_iter().map(|(a, _)| a).collect(),
                    )),
                    span.span(),
                )
            })
            .labelled("alternative action");

        // Sequence: a; b; c    or    a seq b seq c
        let sequence = alternate
            .separated_by(just(Token::Semicolon).or(just(Token::Seq)))
            .at_least(1)
            .allow_trailing()
            .collect::<Vec<_>>()
            .map_with(|mut actions, span| {
                if actions.len() == 1 {
                    return actions.pop().unwrap();
                }
                (
                    Action::List(ActionList::Sequence(
                        actions.into_iter().map(|(a, _)| a).collect(),
                    )),
                    span.span(),
                )
            })
            .labelled("sequence action");

        sequence.labelled("action")
    });

    action_parser.define(a_parser);

    (
        rule_parser,
        atomic_condition_parser,
        compound_parser,
        condition_parser,
        primitive_event_parser,
        action_parser,
    )
}

/// A Parser for the entire CL0 language.
pub fn program_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Vec<Spanned<Rule>>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    // Parse a sequence of rules, separated by newlines or semicolons
    rule_parser::<I>()
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(end())
        .labelled("program")
}

/// A Parser for directives in the CL0 language.
pub fn directive_parser<'tokens, 'src: 'tokens, I>()
-> impl Parser<'tokens, I, Spanned<Directive>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
where
    I: ValueInput<'tokens, Token = Token<'src>, Span = Span>,
{
    let descriptor = select! { Token::Descriptor(var) => var };

    let inside_params = descriptor
        .clone()
        .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis));

    let scale = just(Token::At)
        .ignore_then(just(Token::Descriptor("scale")))
        .then(
            (select! { Token::Number(num) => num })
                .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis)),
        )
        .then(compound_parser().clone())
        .map_with(|((_, num), (comp, _)), s| {
            (
                Directive::Scale {
                    number: num,
                    policy: comp,
                },
                s.span(),
            )
        });

    let include = just(Token::At)
        .ignore_then(just(Token::Descriptor("include")))
        .then(inside_params.clone())
        .map_with(|(_, desc), s| (Directive::Include(desc.to_string()), s.span()));

    let exclude = just(Token::At)
        .ignore_then(just(Token::Descriptor("exclude")))
        .then(inside_params.clone())
        .map_with(|(_, desc), s| (Directive::Exclude(desc.to_string()), s.span()));

    let interleaving = just(Token::At)
        .ignore_then(just(Token::Descriptor("interleaving")))
        .map_with(|_, s| (Directive::Interleaving, s.span()));

    let external_var = just(Token::At)
        .ignore_then(just(Token::Descriptor("external")))
        .then(inside_params.clone())
        .map_with(|(_, desc), s| (Directive::ExternalVar(desc.to_string()), s.span()));

    let external_event = just(Token::At)
        .ignore_then(just(Token::Descriptor("external")))
        .then(
            primitive_event_parser()
                .clone()
                .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis)),
        )
        .map_with(|(_, (pe, _)), s| (Directive::ExternalEvent(pe), s.span()));

    scale
        .or(include)
        .or(exclude)
        .or(interleaving)
        .or(external_event)
        .or(external_var)
        .labelled("directive")
}
