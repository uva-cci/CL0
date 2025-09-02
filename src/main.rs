use std::{
    io::{self, Write}, sync::Arc
};

use cl0_node::{node::Node, types::{RuleWithArgs, ActivationStatus}};
use cl0_parser::{ast::Compound, lex_and_parse_compound, lex_and_parse_safe};

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const MAGENTA: &str = "\x1b[35m";
const RED: &str = "\x1b[31m";

#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    fn validate_policy(policy: &str) -> Option<Compound> {
        // Try to catch panics from lex_and_parse_compound
        let result = std::panic::catch_unwind(|| lex_and_parse_compound(policy));
        match result {
            Ok(compound) => Some(compound),
            Err(_err) => None,
        }
    }

    // Welcome banner
    println!("{}{}Welcome to \"CL0 Node REPL\"!{}", BOLD, CYAN, RESET);
    println!();
    println!(
        "{}This REPL resembles the engine behind a single node in the system.{}",
        BLUE, RESET
    );
    println!();
    println!();
    println!(
        "{}Enter initial rules (optionally) by wrapping {{ ... }}{}",
        BLUE, RESET
    );
    println!();
    println!("{}For example:{}", GREEN, RESET);
    println!("{}{{{}", YELLOW, RESET);
    println!("  {}#e: c => +a.{}", MAGENTA, RESET);
    println!("  {}=> #e.{}", MAGENTA, RESET);
    println!("  {}{{ #f => +v }} as alias.{}", MAGENTA, RESET);
    println!("{}}}{}", YELLOW, RESET);
    println!();
    println!(
        "{}Use {}{{}}{} for empty policy initialization.{}",
        BLUE, YELLOW, BLUE, RESET
    );
    println!();
    println!();
    println!(
        "{}After initialization, run rules one by one.{}",
        BLUE, RESET
    );
    println!();
    println!(
        "{}Use the 'observe' command to view state.{}\n",
        BLUE, RESET
    );
    println!();

    // Basic syntax guide
    println!("{}{}Basic Syntax:{}\n", BOLD, CYAN, RESET);
    println!("{}Events:{}", BOLD, RESET);
    println!("  {}#event{}        Trigger event", GREEN, RESET);
    println!("  {}+condition{}    Production event", GREEN, RESET);
    println!("  {}-condition{}    Consumption event\n", GREEN, RESET);
    println!("{}Conditions:{}", BOLD, RESET);
    println!("  {}foo{}           Atomic condition", YELLOW, RESET);
    println!("  {}not foo{}       Negation", YELLOW, RESET);
    println!("  {}( foo ){}       Priority", YELLOW, RESET);
    println!("  {}a and b{}       Conjunction", YELLOW, RESET);
    println!("  {}a or b{}        Disjunction", YELLOW, RESET);
    println!("  {}{{ ... }} as r{}  Compound alias", YELLOW, RESET);
    println!("\n{}Actions:{}", BOLD, RESET);
    println!("  {}#e{}            Trigger event", GREEN, RESET);
    println!("  {}+v{}            Production action", GREEN, RESET);
    println!("  {}-v{}            Consumption action", GREEN, RESET);
    println!("  {}a, b{}          Parallel actions", GREEN, RESET);
    println!("  {}a; b{}          Sequential actions", GREEN, RESET);
    println!("  {}a alt b{}       Alternative actions\n", GREEN, RESET);

    // Get initial policy
    println!(
        "{}{}Enter initial policy (or leave empty):{}.",
        BOLD, CYAN, RESET
    );

    // The node data
    let mut node: Option<Arc<Node>> = None;

    let initial_policy = {
        let mut policy = String::new();
        let mut bracket_depth = 0;
        loop {
            // Prompt for policy lines
            print!("{}{}> {}", BOLD, MAGENTA, RESET);
            stdout.flush().expect("Failed to flush stdout");

            let mut line = String::new();
            match stdin.read_line(&mut line) {
                Ok(0) => break policy.clone(), // EOF
                Ok(_) => {
                    // Reset policy if user types "reset"
                    if line.trim().eq_ignore_ascii_case("reset") {
                        println!(
                            "{}{}Enter initial policy (or leave empty):{}.",
                            BOLD, CYAN, RESET
                        );
                        policy.clear();
                        bracket_depth = 0;
                        continue;
                    }

                    // Count the brackets to determine if we are done
                    bracket_depth += line.chars().filter(|&c| c == '{').count();
                    bracket_depth -= line.chars().filter(|&c| c == '}').count();
                    // If user pressed empty line before any input, accept empty policy
                    if line.trim().is_empty() && policy.is_empty() {
                        break policy.clone();
                    }
                    policy.push_str(&line);

                    // Capture the policy text

                    // Check validity on full policy text
                    if bracket_depth == 0 {
                        if let Some(compound) = validate_policy(&policy) {
                            println!("{}Valid policy detected, proceeding...{}", GREEN, RESET);
                            println!("{}Parsed policy:\n{}{}{}", GREEN, RESET, compound, RESET);

                            node = Some(Node::new_with_rules(Some(compound.clone().rules)).await);
                            break policy.clone();
                        } else {
                            println!(
                                "{}Policy incomplete or invalid, continue typing... Type \"reset\" to start over{}",
                                YELLOW, RESET
                            );
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{}Error reading policy: {}{}", MAGENTA, err, RESET);
                    break policy.clone();
                }
            }
        }
    };
    println!(
        "{}Initial policy accepted:\n{}{}{}",
        GREEN, RESET, initial_policy, RESET
    );
    
    
    // Unwrap the node (By this point, it should be created)
    if node.is_none() {
        node = Some(Node::new_with_rules(None).await);
    }
    let node = node.unwrap();

    loop {
        // Prompt
        print!(">> ");
        stdout.flush().expect("Failed to flush stdout");

        // Read a line of input
        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl-D)
                println!();
                break;
            }
            Ok(_) => {
                let trimmed = input.trim_end();
                // Exit commands
                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    break;
                }
                if trimmed.eq_ignore_ascii_case("observe") {
                    // Show the current state of the node
                    println!("{}Current state:{}", BLUE, RESET);
                    println!("{}Rules:{}", YELLOW, RESET);
                    println!("==========================");
                    // Fetch and print the rules
                    let rule_res = node.api.get_rules.call(true).await;
                    match rule_res {
                        Ok(rules) => {
                            if rules.is_empty() {
                                println!("{}    No rules defined.{}", YELLOW, RESET);
                            } else {
                                for rule in rules {
                                    let namespace_string = match rule.alias {
                                        Some(ns) => ns.join(".") + ".",
                                        None => "".to_string(),
                                    };
                                    println!("{}    {}{}{}{}: {}{}{}", BLUE, namespace_string, RESET,rule.rule.to_string().trim_end_matches("."), YELLOW, (if rule.value == ActivationStatus::True { GREEN } else { RED }), rule.value, RESET);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{}Failed to fetch rules: {}{}", MAGENTA, e, RESET);
                        }
                    }
                    println!("==========================");
                    println!("{}Variables:{}", YELLOW, RESET);
                    println!("==========================");

                    // Fetch and print the variables
                    let vars_res = node.vars.iter().collect::<Vec<_>>();

                    for var in vars_res {
                        let guard_key = var.key().clone();
                        let guard_value = var.value().clone();
                        println!("{}    {}: {}{}{}", BLUE, guard_key, (if guard_value == ActivationStatus::True { GREEN } else { RED }), guard_value, RESET);
                    }
                    println!("==========================");

                    // let state = node.observe().await;
                    // println!("{:#?}", state);
                    continue;
                }
                // Parse the input as a rule
                let rules = match lex_and_parse_safe(trimmed) {
                    Ok(rules) => rules,
                    Err(_) => {
                        // Should automatically print the error
                        continue;
                    }
                };

                // Add the rules to the node
                let result = node
                    .api
                    .new_rules
                    .call(rules.clone().into_iter().map(|r| RuleWithArgs::from(r)).collect())
                    .await;
                match result {
                    Ok(_) => {
                        if rules.clone().len() == 0 {
                            println!("{}No rules were added.{}", YELLOW, RESET);
                        } else {
                            println!("{}Added {} rule(s) successfully.{}", GREEN, rules.len(), RESET);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}Failed to add rules: {}{}", MAGENTA, e, RESET);
                    }
                }

                // Echo back the input
                println!("{}", trimmed);
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
                break;
            }
        }
    }
}
