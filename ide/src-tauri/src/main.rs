#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gag::BufferRedirect;
use std::io::Read;
use talea::runtime::interpreter::Interpreter;

#[tauri::command]
fn run_talea(code: String) -> Result<String, String> {
    use talea::lexer::Lexer;
    use talea::parser::Parser;
    let mut output = String::new();
    {
        let mut redirect = BufferRedirect::stdout().map_err(|e| e.to_string())?;
        let mut lexer = Lexer::new(&code);
        let tokens = lexer.all_tokens();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().map_err(|e| format!("Parse Error: {}", e))?;
        let mut interpreter = Interpreter::new();
        interpreter
            .execute(ast)
            .map_err(|e| format!("Runtime Error: {}", e))?;
        redirect
            .read_to_string(&mut output)
            .map_err(|e| e.to_string())?;
    }
    Ok(output)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![run_talea])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
