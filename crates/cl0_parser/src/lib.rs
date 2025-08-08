pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use std::error::Error;

use chumsky::{Parser, span::SimpleSpan};

use crate::ast::{Compound, Rule};
use crate::parser::{compound_parser, program_parser};
use crate::{lexer::lexer, token::Token};

use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

pub fn lex_and_parse_span(src: &str) -> Vec<(Rule, SimpleSpan)> {
    // Placeholder when parsing from files
    let file_id: &'static str = "input";

    // Lex all the tokens with their character spans
    let lex_result = lexer().parse(src);
    if lex_result.has_errors() {
        for err in lex_result.errors() {
            let span = err.span().clone();
            let SimpleSpan { start, end, .. } = span;
            let r1 = start..end;
            let r2 = r1.clone();

            // Create lex error report if lexing fails
            Report::build(ReportKind::Error, (file_id, r1))
                .with_message(format!("{:?}", err))
                .with_label(Label::new((file_id, r2)).with_color(Color::Red))
                .finish()
                .print((file_id, Source::from(src)))
                .unwrap();
        }
        std::process::exit(1);
    }

    // Get the lexer output: vector of tokens with their spans
    let spanned: Vec<(Token, SimpleSpan)> = lex_result.output().cloned().expect("no tokens");
    let tokens: Vec<Token> = spanned.iter().map(|(t, _)| t.clone()).collect();

    // Parse the tokens into an AST
    let parse_result = program_parser().parse(tokens.as_slice());
    if parse_result.has_errors() {
        for err in parse_result.errors() {
            // Convert the token span to character indices
            let tok_span = err.span().clone();
            let tok_start = tok_span.start;
            let tok_end = tok_span.end;
            let char_start = spanned.get(tok_start).map(|(_, sp)| sp.start).unwrap_or(0);
            let char_end = if tok_end == 0 {
                char_start
            } else {
                spanned
                    .get(tok_end - 1)
                    .map(|(_, sp)| sp.end)
                    .unwrap_or(char_start)
            };
            let report_range = char_start..char_end;
            let label_range = report_range.clone();

            // Create a custom error for the parser
            let expected: Vec<String> = err
                .expected()
                .map(|t| format!("{}", t.to_string().fg(Color::Green)))
                .collect();
            let found = err
                .found()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "end of input".into());
            let msg = if expected.is_empty() {
                format!("Unexpected {}", found.fg(Color::Blue))
            } else {
                format!(
                    "found {} at {:?} expected {}",
                    found.fg(Color::Blue),
                    report_range,
                    expected.join(", ")
                )
            };

            // Create a parse error report
            Report::build(ReportKind::Error, (file_id, report_range))
                .with_message(msg.clone())
                .with_label(
                    Label::new((file_id, label_range))
                        .with_color(Color::Red)
                        .with_message(msg),
                )
                .finish()
                .print((file_id, Source::from(src)))
                .unwrap();
        }
        std::process::exit(1);
    }

    // If parsing was successful, print the AST
    let output: &Vec<(Rule, SimpleSpan)> = parse_result.output().expect("No output from parser");
    output.clone()
}

pub fn lex_and_parse(src: &str) -> Vec<Rule> {
    lex_and_parse_span(src)
        .into_iter()
        .map(|(rule, _span)| rule)
        .collect()
}

pub fn parse_and_print(src: &str) {
    let output = lex_and_parse_span(src);

    let rules: Vec<ast::Rule> = output.iter().map(|(rule, _span)| rule.clone()).collect();
    println!("{:#?}", rules);
}

pub fn lex_and_parse_compound(src: &str) -> Compound {
    // Placeholder when parsing from files
    let file_id: &'static str = "input";

    // Lex all the tokens with their character spans
    let lex_result = lexer().parse(src);
    if lex_result.has_errors() {
        for err in lex_result.errors() {
            let span = err.span().clone();
            let SimpleSpan { start, end, .. } = span;
            let r1 = start..end;
            let r2 = r1.clone();

            // Create lex error report if lexing fails
            Report::build(ReportKind::Error, (file_id, r1))
                .with_message(format!("{:?}", err))
                .with_label(Label::new((file_id, r2)).with_color(Color::Red))
                .finish()
                .print((file_id, Source::from(src)))
                .unwrap();
        }
        std::process::exit(1);
    }

    // Get the lexer output: vector of tokens with their spans
    let spanned: Vec<(Token, SimpleSpan)> = lex_result.output().cloned().expect("no tokens");
    let tokens: Vec<Token> = spanned.iter().map(|(t, _)| t.clone()).collect();

    // Parse the tokens into an AST
    let parse_result = compound_parser().parse(tokens.as_slice());
    if parse_result.has_errors() {
        for err in parse_result.errors() {
            // Convert the token span to character indices
            let tok_span = err.span().clone();
            let tok_start = tok_span.start;
            let tok_end = tok_span.end;
            let char_start = spanned.get(tok_start).map(|(_, sp)| sp.start).unwrap_or(0);
            let char_end = if tok_end == 0 {
                char_start
            } else {
                spanned
                    .get(tok_end - 1)
                    .map(|(_, sp)| sp.end)
                    .unwrap_or(char_start)
            };
            let report_range = char_start..char_end;
            let label_range = report_range.clone();

            // Create a custom error for the parser
            let expected: Vec<String> = err
                .expected()
                .map(|t| format!("{}", t.to_string().fg(Color::Green)))
                .collect();
            let found = err
                .found()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "end of input".into());
            let msg = if expected.is_empty() {
                format!("Unexpected {}", found.fg(Color::Blue))
            } else {
                format!(
                    "found {} at {:?} expected {}",
                    found.fg(Color::Blue),
                    report_range,
                    expected.join(", ")
                )
            };

            // Create a parse error report
            Report::build(ReportKind::Error, (file_id, report_range))
                .with_message(msg.clone())
                .with_label(
                    Label::new((file_id, label_range))
                        .with_color(Color::Red)
                        .with_message(msg),
                )
                .finish()
                .print((file_id, Source::from(src)))
                .unwrap();
        }
        std::process::exit(1);
    }

    // If parsing was successful, print the AST
    let output: &(Compound, SimpleSpan) = parse_result.output().expect("No output from parser");
    output.clone().0
}

pub fn lex_and_parse_safe(
    src: &str,
) -> Result<Vec<Rule>, Box<dyn std::error::Error + Send + Sync>> {
    // Placeholder when parsing from files
    let file_id: &'static str = "input";

    // Lex all the tokens with their character spans
    let lex_result = lexer().parse(src);
    if lex_result.has_errors() {
        for err in lex_result.errors() {
            let span = err.span().clone();
            let SimpleSpan { start, end, .. } = span;
            let r1 = start..end;
            let r2 = r1.clone();

            // Create lex error report if lexing fails
            Report::build(ReportKind::Error, (file_id, r1))
                .with_message(format!("{:?}", err))
                .with_label(Label::new((file_id, r2)).with_color(Color::Red))
                .finish()
                .print((file_id, Source::from(src)))
                .unwrap();

            return Err(Box::<dyn Error + Send + Sync>::from(format!(
                "Failed to parse"
            )));
        }
    }

    // Get the lexer output: vector of tokens with their spans
    let spanned: Vec<(Token, SimpleSpan)> = lex_result.output().cloned().expect("no tokens");
    let tokens: Vec<Token> = spanned.iter().map(|(t, _)| t.clone()).collect();

    // Parse the tokens into an AST
    let parse_result = program_parser().parse(tokens.as_slice());
    if parse_result.has_errors() {
        for err in parse_result.errors() {
            // Convert the token span to character indices
            let tok_span = err.span().clone();
            let tok_start = tok_span.start;
            let tok_end = tok_span.end;
            let char_start = spanned.get(tok_start).map(|(_, sp)| sp.start).unwrap_or(0);
            let char_end = if tok_end == 0 {
                char_start
            } else {
                spanned
                    .get(tok_end - 1)
                    .map(|(_, sp)| sp.end)
                    .unwrap_or(char_start)
            };
            let report_range = char_start..char_end;
            let label_range = report_range.clone();

            // Create a custom error for the parser
            let expected: Vec<String> = err
                .expected()
                .map(|t| format!("{}", t.to_string().fg(Color::Green)))
                .collect();
            let found = err
                .found()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "end of input".into());
            let msg = if expected.is_empty() {
                format!("Unexpected {}", found.fg(Color::Blue))
            } else {
                format!(
                    "found {} at {:?} expected {}",
                    found.fg(Color::Blue),
                    report_range,
                    expected.join(", ")
                )
            };

            // Create a parse error report
            Report::build(ReportKind::Error, (file_id, report_range))
                .with_message(msg.clone())
                .with_label(
                    Label::new((file_id, label_range))
                        .with_color(Color::Red)
                        .with_message(msg),
                )
                .finish()
                .print((file_id, Source::from(src)))
                .unwrap();
            return Err(Box::<dyn Error + Send + Sync>::from(format!(
                "Failed to parse"
            )));
        }
    }

    // If parsing was successful, print the AST
    let output: &Vec<(Rule, SimpleSpan)> = parse_result.output().expect("No output from parser");
    Ok(output
        .clone()
        .into_iter()
        .map(|(rule, _span)| rule)
        .collect::<Vec<Rule>>())
}
