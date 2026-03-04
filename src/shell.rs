use anyhow::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::output::OutputFormat;
use crate::Cli;

pub async fn run_shell(
    output: OutputFormat,
    api_key: Option<String>,
    private_key: Option<String>,
) -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    println!();
    println!("  {}", "Limitless CLI · Interactive Shell".cyan().bold());
    println!("  Type 'help' for commands, 'exit' to quit.");
    println!();

    loop {
        match rl.readline("limitless> ") {
            Ok(line) => {
                let line = line.trim();
                if line == "exit" || line == "quit" {
                    break;
                }
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                let args = split_args(line);
                let cmd = args.first().map(|s| s.as_str()).unwrap_or("");

                if cmd == "shell" {
                    println!("Already in shell mode.");
                    continue;
                }
                if cmd == "setup" {
                    println!("Run 'limitless setup' outside the shell.");
                    continue;
                }

                let mut full_args = vec!["limitless".to_string()];
                full_args.push("--output".into());
                full_args.push(output.to_string());
                if let Some(ref key) = api_key {
                    full_args.push("--api-key".into());
                    full_args.push(key.clone());
                }
                if let Some(ref pk) = private_key {
                    full_args.push("--private-key".into());
                    full_args.push(pk.clone());
                }
                full_args.extend(args);

                match <Cli as clap::Parser>::try_parse_from(&full_args) {
                    Ok(cli) => {
                        if cli.command.is_none() {
                            continue;
                        }
                        if let Err(e) = Box::pin(crate::execute(cli)).await {
                            match &output {
                                OutputFormat::Json => {
                                    println!(
                                        "{}",
                                        serde_json::json!({"error": e.to_string()})
                                    );
                                }
                                OutputFormat::Table => {
                                    eprintln!("{} {:#}", "error:".red().bold(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        e.print().ok();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("{} {}", "error:".red().bold(), e);
                break;
            }
        }
    }

    println!("Goodbye!");
    Ok(())
}

fn split_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quote_char = '"';

    for ch in input.chars() {
        if in_quote {
            if ch == quote_char {
                in_quote = false;
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            in_quote = true;
            quote_char = ch;
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}
