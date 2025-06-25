// src/main.rs

use std::io::{self, Write};
// Bring the new Interpreter into scope
use talea::runtime::interpreter::Interpreter;

fn main() {
    println!("talea REPL v0.1.0");

    // Create the interpreter. It holds the state (like variables).
    // It lives for the entire REPL session.
    let mut interpreter = Interpreter::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input == "exit" {
            break;
        }
        
        if input.is_empty() {
            continue;
        }

        // 1. Lexing
        let mut lexer = talea::lexer::Lexer::new(input);
        let tokens = lexer.all_tokens();

        // 2. Parsing
        let mut parser = talea::parser::Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                eprintln!("Parse Error: {}", e);
                continue; // Skip to the next loop iteration
            }
        };

        // 3. INTERPRETING!
        // The interpreter executes the AST.
        if let Err(e) = interpreter.execute(ast) {
            eprintln!("Runtime Error: {}", e);
        }
    }
}