// src/main.rs

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;
use talea::runtime::interpreter::Interpreter;

fn main() {
    let mut interpreter = Interpreter::new();
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let path = &args[1];
        let Ok(contents) = fs::read_to_string(path) else {
            eprintln!("Could not read file: {}", path);
            return;
        };
        let mut lexer = talea::lexer::Lexer::new(&contents);
        let tokens = lexer.all_tokens();
        let mut parser = talea::parser::Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse Error: {}", e);
                return;
            }
        };
        if let Err(e) = interpreter.execute(ast) {
            eprintln!("Runtime Error: {}", e);
        }
        return;
    }

    println!("talea REPL v0.1.0");
    let mut rl = DefaultEditor::new().expect("failed to create editor");

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let input = line.trim();
                if input == "exit" {
                    break;
                }
                if input.is_empty() {
                    continue;
                }
                rl.add_history_entry(input).ok();

                let mut lexer = talea::lexer::Lexer::new(input);
                let tokens = lexer.all_tokens();

                let mut parser = talea::parser::Parser::new(tokens);
                let ast = match parser.parse() {
                    Ok(ast) => ast,
                    Err(e) => {
                        eprintln!("Parse Error: {}", e);
                        continue;
                    }
                };

                if let Err(e) = interpreter.execute(ast) {
                    eprintln!("Runtime Error: {}", e);
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}
