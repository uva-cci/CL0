use crate::token::Token;
use chumsky::prelude::*;

pub type Spanned<T> = (T, SimpleSpan);

/// Constructs the lexer that transforms raw input characters into a vector of `Token`s.
///
/// This lexer handles:
/// - Multi-character symbols: `=>`, `->`, `-o`
/// - Single-character symbols: `#`, `:`, `;`, `+`, `-`, `.`, `(`, `)`, `,`
/// - Keywords: `seq`, `par`, `alt`, `and`, `or`, `not`
/// - Identifiers: any other alphanumeric word
/// - Line comments starting with `%`, which are ignored
///
pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, SimpleSpan>>>
{
    // Multi-character symbols must be matched before single-character and identifiers
    let multi_symbol = choice((
        just("=>").to(Token::FatArrow),
        just("->").to(Token::ThinArrow),
        just("-o").to(Token::DashO),
    ));

    // Distinction between `.` as a dot and `.` as an end of rule
    let dot_or_endrule = just('.')
        .then(any().rewind().or_not())
        .map(|(_, next): (char, Option<char>)| match next {
            Some(c) if c.is_alphanumeric() || c == '{' => Token::Dot,
            _ => Token::EndRule,
        });

    // Single-character symbols
    let symbol = choice((
        just("#").to(Token::Hash),
        just(":").to(Token::Colon),
        just(";").to(Token::Semicolon),
        just("+").to(Token::Plus),
        just("-").to(Token::Minus),
        just("(").to(Token::LeftParenthesis),
        just(")").to(Token::RightParenthesis),
        just("{").to(Token::LeftCBracket),
        just("}").to(Token::RightCBracket),
        just(",").to(Token::Comma),
        just("@").to(Token::At),
    ));

    // Reserved words and identifiers
    let ident = text::ascii::ident().map(|identifier: &str| match identifier {
        "seq" => Token::Seq,
        "par" => Token::Par,
        "alt" => Token::Alt,
        "and" => Token::And,
        "or" => Token::Or,
        "not" => Token::Not,
        "as" => Token::As,
        _ => Token::Descriptor(identifier),
    });

    let number = text::int(10)
        .try_map(|s: &str, span| {
            s.parse::<u8>()
                .map(Token::Number)
                .map_err(|_| Rich::custom(span, "number out of range for u8"))
        });

    let token = multi_symbol.or(dot_or_endrule).or(symbol).or(number).or(ident);

    // Comments: skip lines beginning with `%`
    let comment = just("%")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    return token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        // If we encounter an error, skip and attempt to lex the next character as a token instead
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect();
}
