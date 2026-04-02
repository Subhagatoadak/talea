//! Entry point for the Talea language runtime.
//!
//! Supports multiple execution modes:
//!
//! - **File mode** (`talea script.talea`): auto-detect English prose vs Talea code.
//! - **Compile mode** (`talea --compile script.talea`): compile to `script.taleac`.
//! - **Run mode** (`talea --run script.taleac`): execute pre-compiled bytecode.
//! - **English mode** (`talea --english script.talea`): translate prose → Talea, print, then run.
//! - **To-Talea mode** (`talea --to-talea script.talea`): translate prose → Talea, print only.
//! - **REPL mode**: interactive prompt with auto-detection per line.

// src/main.rs

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;
use talea::runtime::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // ── Compile mode ────────────────────────────────────────────────────────
    if args.len() >= 3 && args[1] == "--compile" {
        let path = &args[2];
        let contents = read_file_or_exit(path);
        let stmts = parse_talea_or_exit(&contents, path);
        let chunk = match talea::compiler::compile(stmts) {
            Ok(c) => c,
            Err(e) => { eprintln!("Compile Error: {}", e); return; }
        };
        let bytes = match chunk.to_bytes() {
            Ok(b) => b,
            Err(e) => { eprintln!("Serialisation Error: {}", e); return; }
        };
        let out = path.replace(".talea", ".taleac").replace(".tea", ".taleac");
        fs::write(&out, bytes).unwrap_or_else(|e| eprintln!("Write Error: {}", e));
        println!("Compiled → {}", out);
        return;
    }

    // ── Run bytecode mode ────────────────────────────────────────────────────
    if args.len() >= 3 && args[1] == "--run" {
        let path = &args[2];
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => { eprintln!("Cannot read '{}': {}", path, e); return; }
        };
        let chunk = match talea::bytecode::Chunk::from_bytes(&bytes) {
            Ok(c) => c,
            Err(e) => { eprintln!("Deserialisation Error: {}", e); return; }
        };
        let mut vm = talea::vm::VM::new();
        if let Err(e) = vm.execute(&chunk) {
            eprintln!("VM Error: {}", e);
        }
        return;
    }

    // ── English → Talea → Run mode ───────────────────────────────────────────
    if args.len() >= 3 && args[1] == "--english" {
        let path = &args[2];
        let contents = read_file_or_exit(path);
        let stmts = english_to_statements_or_exit(&contents);
        print_generated_talea(&stmts);
        let mut interp = Interpreter::new();
        if let Err(e) = interp.execute(stmts) {
            eprintln!("Runtime Error: {}", e);
        }
        return;
    }

    // ── English → Talea (print only) ─────────────────────────────────────────
    if args.len() >= 3 && args[1] == "--to-talea" {
        let path = &args[2];
        let contents = read_file_or_exit(path);
        let stmts = english_to_statements_or_exit(&contents);
        print_generated_talea(&stmts);
        return;
    }

    // ── File mode (auto-detect) ───────────────────────────────────────────────
    if args.len() >= 2 && !args[1].starts_with('-') {
        let path = &args[1];
        let contents = read_file_or_exit(path);

        if path.ends_with(".taleac") {
            // Pre-compiled bytecode
            let bytes = match fs::read(path) {
                Ok(b) => b,
                Err(e) => { eprintln!("Cannot read '{}': {}", path, e); return; }
            };
            let chunk = match talea::bytecode::Chunk::from_bytes(&bytes) {
                Ok(c) => c,
                Err(e) => { eprintln!("Deserialisation Error: {}", e); return; }
            };
            let mut vm = talea::vm::VM::new();
            if let Err(e) = vm.execute(&chunk) {
                eprintln!("VM Error: {}", e);
            }
        } else if is_english_prose(&contents) {
            eprintln!("[talea] Detected English prose — translating…");
            let stmts = english_to_statements_or_exit(&contents);
            print_generated_talea(&stmts);
            let mut interp = Interpreter::new();
            if let Err(e) = interp.execute(stmts) {
                eprintln!("Runtime Error: {}", e);
            }
        } else {
            let mut interp = Interpreter::new();
            let stmts = parse_talea_or_exit(&contents, path);
            if let Err(e) = interp.execute(stmts) {
                eprintln!("Runtime Error: {}", e);
            }
        }
        return;
    }

    // ── REPL mode ────────────────────────────────────────────────────────────
    let mut interpreter = Interpreter::new();
    println!("talea REPL v0.2.0  (type 'exit' to quit)");
    let mut rl = DefaultEditor::new().expect("failed to create editor");

    loop {
        match rl.readline("> ") {
            Ok(line) => {
                let input = line.trim().to_string();
                if input == "exit" { break; }
                if input.is_empty() { continue; }
                rl.add_history_entry(&input).ok();

                // Per-line auto-detection
                let stmts = if is_english_prose(&input) {
                    match talea::nl_parser::parse_english(&input) {
                        Ok(s) => {
                            if !s.is_empty() {
                                print_generated_talea(&s);
                            }
                            s
                        }
                        Err(e) => {
                            eprintln!("NL Parse Error: {}", e);
                            continue;
                        }
                    }
                } else {
                    let mut lexer = talea::lexer::Lexer::new(&input);
                    let tokens = lexer.all_tokens();
                    let mut parser = talea::parser::Parser::new(tokens);
                    match parser.parse() {
                        Ok(ast) => ast,
                        Err(e) => {
                            eprintln!("Parse Error: {}", e);
                            continue;
                        }
                    }
                };

                if let Err(e) = interpreter.execute(stmts) {
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

// ──────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────

fn read_file_or_exit(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Could not read file '{}': {}", path, e);
        std::process::exit(1);
    })
}

fn parse_talea_or_exit(contents: &str, path: &str) -> Vec<talea::ast::Statement> {
    let mut lexer = talea::lexer::Lexer::new(contents);
    let tokens = lexer.all_tokens();
    let mut parser = talea::parser::Parser::new(tokens);
    parser.parse().unwrap_or_else(|e| {
        eprintln!("Parse Error in '{}': {}", path, e);
        std::process::exit(1);
    })
}

fn english_to_statements_or_exit(contents: &str) -> Vec<talea::ast::Statement> {
    talea::nl_parser::parse_english(contents).unwrap_or_else(|e| {
        eprintln!("NL Parse Error: {}", e);
        std::process::exit(1);
    })
}

/// Print a rough Talea representation of a statement list.
fn print_generated_talea(stmts: &[talea::ast::Statement]) {
    println!("--- Generated Talea ---");
    for stmt in stmts {
        println!("  {:?}", stmt);
    }
    println!("-----------------------");
}

/// Heuristic: if fewer than 30% of non-empty, non-comment lines start with a
/// known Talea keyword, treat the content as English prose.
fn is_english_prose(content: &str) -> bool {
    let talea_keywords = [
        "load", "save", "print", "define", "let", "tokenize", "sentiment", "count",
        "split", "regress", "use", "if", "for", "openai", "mean", "clean", "lowercase",
        "uppercase", "stem", "normalize", "lemmatize", "tag", "concordance", "wordcloud",
        "association", "semantic", "keyword", "ttest", "anova", "shapiro", "kolmogorov",
        "median", "mode", "skewness", "kurtosis", "summarize", "insight", "translate",
        "filter", "import", "call", "set", "replace", "join", "doc2bow",
    ];
    let lines: Vec<&str> = content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .collect();

    if lines.is_empty() {
        return false;
    }

    let keyword_lines = lines.iter().filter(|l| {
        let first = l.trim().split_whitespace().next().unwrap_or("").to_lowercase();
        talea_keywords.contains(&first.as_str())
    }).count();

    (keyword_lines as f32 / lines.len() as f32) < 0.3
}
