use std::sync::{Arc, Mutex};

use kwaak::commands::{Command, CommandEvent, Responder};

use crate::app_state::AppState;

/// Tell the backend to quit
#[tauri::command]
pub fn quit(state: tauri::State<AppState>) {
    state.dispatch(CommandEvent::quit());
}

pub fn show_config(state: tauri::State<Arc<Mutex<AppState>>>, current_chat: uuid::Uuid) {
    let responder = state.lock().unwrap().command_tx.clone();
    let event = CommandEvent::builder()
        .command(Command::ShowConfig)
        .uuid(current_chat)
        .responder(Arc::new(responder) as Arc<dyn Responder>)
        .build()
        .unwrap();
}
