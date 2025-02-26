mod app_state;
mod frontend_commands;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use app_state::AppState;
use kwaak::{
    commands::{self, CommandEvent},
    config::Config,
    repository::Repository,
};
use tauri::Manager as _;
use tokio::sync::mpsc;
use tokio_util::task::AbortOnDropHandle;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let repository = load_repository();

    // Start tracing
    let _guard = kwaak::kwaak_tracing::init(&repository, false);

    // Start the backend
    let mut handler = commands::CommandHandler::from_repository(repository.clone());

    let command_tx = handler.command_tx().clone();

    let command_handle = handler.start();

    let app_state = Arc::new(Mutex::new(AppState {
        repository,
        command_tx,
        command_handle,
        chats: Vec::new(),
    }));

    // Load the kwaak config / repository
    // Start a command handler
    // Copy the command tx to the appstate
    // poc: print the config via the command handler
    tauri::Builder::default()
        .setup(|app| {
            app.manage(app_state);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_repository() -> Repository {
    let config =
        Config::load(Path::new("kwaak.toml")).expect("failed to load config, is it present?");
    Repository::from_config(config)
}
