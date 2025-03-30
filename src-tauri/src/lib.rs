mod actions;
mod app_state;
mod frontend_commands;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use app_state::AppState;
use kwaak::{
    commands::{self},
    config::Config,
    repository::Repository,
};
use tauri::Manager as _;

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
    let handler = commands::CommandHandler::from_repository(repository.clone());

    let command_tx = handler.command_tx().clone();

    let command_handle = handler.start();

    let app_state = Arc::new(Mutex::new(AppState {
        repository,
        command_tx,
        command_handle,
        chats: Vec::new(),
        command_responder: None,
    }));

    AppState::spawn_responder(&app_state);

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
        .invoke_handler(tauri::generate_handler![
            // These commands wrap the kwaak commands exactly as is
            frontend_commands::quit,
            frontend_commands::show_config,
            frontend_commands::index_repository,
            frontend_commands::stop_agent,
            frontend_commands::chat,
            frontend_commands::diff,
            frontend_commands::exec,
            frontend_commands::retry_chat,
            // TODO: Can imagine other commands to read (maybe write) on the internal state would
            // go here as well
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_repository() -> Repository {
    let config =
        Config::load(Some(Path::new("kwaak.toml"))).expect("failed to load config, is it present?");
    Repository::from_config(config)
}
