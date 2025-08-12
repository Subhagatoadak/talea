// src/main.rs

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
// Bring the new Interpreter into scope
use talea::runtime::interpreter::Interpreter;

fn main() {
    println!("talea REPL v0.1.0");

    let mut interpreter = Interpreter::new();
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
                // Add the trimmed input to the history without moving `line`
                rl.add_history_entry(input).ok();

                // 1. Lexing
                let mut lexer = talea::lexer::Lexer::new(input);
                let tokens = lexer.all_tokens();

                // 2. Parsing
                let mut parser = talea::parser::Parser::new(tokens);
                let ast = match parser.parse() {
                    Ok(ast) => ast,
                    Err(e) => {
                        eprintln!("Parse Error: {}", e);
                        continue;
                    }
                };

                // 3. Interpreting
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