// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    initial_file: Mutex<Option<String>>,
}

#[tauri::command]
fn get_initial_file(state: tauri::State<AppState>) -> Option<String> {
    state.initial_file.lock().unwrap().take()
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            initial_file: Mutex::new(None),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![get_initial_file])
        .setup(|app| {
            let handle = app.handle().clone();
            let state = handle.state::<AppState>();
            if let Some(arg) = std::env::args().nth(1) {
                *state.initial_file.lock().unwrap() = Some(arg);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
