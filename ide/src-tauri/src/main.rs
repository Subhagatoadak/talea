#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use talea::runtime::interpreter::Interpreter;

#[tauri::command]
fn run_talea(code: String) -> Result<String, String> {
    use talea::lexer::Lexer;
    use talea::parser::Parser;
    let output = Arc::new(Mutex::new(String::new()));
    let captured = output.clone();
    std::io::set_print(move |s| {
        captured.lock().unwrap().push_str(s);
    });
    let mut lexer = Lexer::new(&code);
    let tokens = lexer.all_tokens();
    let mut parser = Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|e| format!("Parse Error: {}", e))?;
    let mut interpreter = Interpreter::new();
    interpreter
        .execute(ast)
        .map_err(|e| format!("Runtime Error: {}", e))?;
    std::io::set_print(|s| {
        use std::io::Write;
        let _ = std::io::stdout().write_all(s.as_bytes());
    });
    Ok(output.lock().unwrap().clone())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_talea])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
