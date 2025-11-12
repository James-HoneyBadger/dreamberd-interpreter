// DreamBerd Interpreter - Main entry point
// Rust port of dreamberd/__init__.py

use clap::Parser;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::fs;
use std::path::Path;

mod base;
mod builtin;
mod interpreter;
mod processor;
mod serialize;

use base::DreamberdError;
use processor::{tokenize, generate_syntax_tree};
use interpreter::Interpreter;

#[derive(Parser, Debug)]
#[command(name = "dreamberd")]
#[command(about = "An interpreter for the perfect programming language, DreamBerd", long_about = None)]
struct Args {
    /// The file containing your DreamBerd code
    file: Option<String>,

    /// Show the full Rust backtrace upon errors
    #[arg(short, long)]
    show_traceback: bool,
}

fn main() {
    let args = Args::parse();

    if let Some(filename) = args.file {
        run_file(&filename);
    } else {
        run_repl();
    }
}

/// Run the REPL (Read-Eval-Print Loop)
fn run_repl() {
    println!("{}", "DreamBerd REPL (Rust Edition)".bright_cyan());
    println!("{}", "Type your code and press Enter. Use Ctrl+C or Ctrl+D to exit.".yellow());
    println!();

    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");
    let mut interpreter = Interpreter::new("__repl__".to_string(), String::new());

    loop {
        let readline = rl.readline(&format!("{} ", ">".yellow()));
        
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                // Add line to history
                let _ = rl.add_history_entry(line.as_str());

                // Tokenize and execute
                match tokenize("__repl__", &line) {
                    Ok(tokens) => {
                        match generate_syntax_tree("__repl__", tokens, &line) {
                            Ok(statements) => {
                                interpreter.code = line.clone();
                                match interpreter.interpret_code_statements(statements) {
                                    Ok(result) => {
                                        if let Some(value) = result {
                                            println!("{:?}", value);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("{}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}", "^C".yellow());
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("{}", "Goodbye!".bright_cyan());
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

/// Run a DreamBerd file
fn run_file(filename: &str) {
    let path = Path::new(filename);
    
    if !path.exists() {
        eprintln!("{}: File not found: {}", "Error".red(), filename);
        std::process::exit(1);
    }

    let code = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("{}: Failed to read file: {}", "Error".red(), e);
            std::process::exit(1);
        }
    };

    // Check for multi-file format (lines starting with ====)
    let files = parse_multi_file_format(&code);

    for (file_name, file_code) in files {
        let display_name = file_name.unwrap_or_else(|| "__unnamed_file__".to_string());
        
        match tokenize(&display_name, &file_code) {
            Ok(tokens) => {
                match generate_syntax_tree(&display_name, tokens, &file_code) {
                    Ok(statements) => {
                        let mut interpreter = Interpreter::new(display_name.clone(), file_code.clone());
                        
                        match interpreter.interpret_code_statements(statements) {
                            Ok(_) => {
                                // Successfully executed
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }

    println!(
        "{}",
        "Code has finished executing. Press Ctrl+C to stop waiting for when-statements and after-statements."
            .yellow()
    );

    // Keep running to handle async operations
    // In a full implementation, this would handle when-statements and after-statements
    std::thread::park();
}

/// Parse multi-file format where files are separated by ===== filename =====
fn parse_multi_file_format(code: &str) -> Vec<(Option<String>, String)> {
    let lines: Vec<&str> = code.lines().collect();
    let mut files = Vec::new();
    let mut current_code = String::new();
    let mut current_name: Option<String> = None;
    let mut first_file = true;

    for line in lines {
        if line.starts_with("=====") && line.ends_with("=====") {
            // Save previous file
            if !first_file {
                files.push((current_name.clone(), current_code.clone()));
                current_code.clear();
            }
            first_file = false;

            // Extract new filename
            let name = line.trim_start_matches('=').trim_end_matches('=').trim();
            current_name = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };
        } else {
            current_code.push_str(line);
            current_code.push('\n');
        }
    }

    // Save last file
    if !current_code.is_empty() || !files.is_empty() {
        files.push((current_name, current_code));
    } else if files.is_empty() {
        // No multi-file format, return entire code
        files.push((None, code.to_string()));
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_multi_file_format() {
        let code = r#"===== file1.db =====
code1
===== file2.db =====
code2"#;
        let files = parse_multi_file_format(code);
        assert_eq!(files.len(), 2);
    }
}
