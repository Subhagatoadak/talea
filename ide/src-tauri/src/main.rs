#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::{fs, path::PathBuf};

#[tauri::command]
fn run_talea(code: String) -> Result<String, String> {
    let mut file: PathBuf = std::env::temp_dir();
    file.push("ide_tmp.tl");
    fs::write(&file, code).map_err(|e| e.to_string())?;

    let output = Command::new("talea")
        .arg(&file)
        .output()
        .map_err(|e| e.to_string())?;

    let _ = fs::remove_file(&file);

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![run_talea])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
