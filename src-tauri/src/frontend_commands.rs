//! Commands are triggered from the frontend and invoke a command on the kwaak backend
//!
//! The Channel is used to send responses back to the frontend. These responses are duplicated
//! so that the internal tauri state is also updated with the response.
use std::sync::{Arc, Mutex};

use kwaak::commands::{Command, CommandEvent, Responder};
use tauri::ipc::Channel;

use crate::app_state::{AppState, CommandResponseWithUuid};

#[tauri::command]
/// Tell the backend to quit
pub fn quit(state: tauri::State<AppState>) {
    state.dispatch(CommandEvent::quit());
}

#[tauri::command]
/// Get the current config used in a chat
pub fn show_config(
    state: tauri::State<AppState>,
    current_chat: uuid::Uuid,
    on_event: Channel<CommandResponseWithUuid>,
) {
    let responder = state.responder_from_channel(current_chat, on_event);

    let event = CommandEvent::builder()
        .command(Command::ShowConfig)
        .uuid(current_chat)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}

#[tauri::command]
/// Force an index of the repository
pub fn index_repository(state: tauri::State<AppState>, on_event: Channel<CommandResponseWithUuid>) {
    // Indexing is a global operation
    // So no chat id to resolve to
    let some_uuid = uuid::Uuid::new_v4();

    let responder = state.responder_from_channel(some_uuid, on_event);

    let event = CommandEvent::builder()
        .command(Command::IndexRepository)
        .uuid(some_uuid)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}

#[tauri::command]
/// Stop an agent so a user can chat again
pub fn stop_agent(
    state: tauri::State<AppState>,
    current_chat: uuid::Uuid,
    on_event: Channel<CommandResponseWithUuid>,
) {
    let responder = state.responder_from_channel(current_chat, on_event);

    let event = CommandEvent::builder()
        .command(Command::StopAgent)
        .uuid(current_chat)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}

#[tauri::command]
/// Send a chat message
pub fn chat(
    state: tauri::State<AppState>,
    current_chat: uuid::Uuid,
    message: &str,
    on_event: Channel<CommandResponseWithUuid>,
) {
    let responder = state.responder_from_channel(current_chat, on_event);

    let event = CommandEvent::builder()
        .command(Command::Chat {
            message: message.into(),
        })
        .uuid(current_chat)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}

#[tauri::command]
/// Get the current changes made by the agent
pub fn diff(
    state: tauri::State<AppState>,
    current_chat: uuid::Uuid,
    on_event: Channel<CommandResponseWithUuid>,
) {
    let responder = state.responder_from_channel(current_chat, on_event);

    let event = CommandEvent::builder()
        .command(Command::Diff)
        .uuid(current_chat)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}

#[tauri::command]
/// Execute a shell command in the context of an agent
pub fn exec(
    state: tauri::State<AppState>,
    current_chat: uuid::Uuid,
    cmd: &str,
    on_event: Channel<CommandResponseWithUuid>,
) {
    let responder = state.responder_from_channel(current_chat, on_event);

    let event = CommandEvent::builder()
        .command(Command::Exec {
            cmd: swiftide::traits::Command::shell(cmd),
        })
        .uuid(current_chat)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}

#[tauri::command]
/// Retries the last chat completion
pub fn retry_chat(
    state: tauri::State<AppState>,
    current_chat: uuid::Uuid,
    on_event: Channel<CommandResponseWithUuid>,
) {
    let responder = state.responder_from_channel(current_chat, on_event);

    let event = CommandEvent::builder()
        .command(Command::RetryChat)
        .uuid(current_chat)
        .responder(responder)
        .build()
        .unwrap();

    state.dispatch(event);
}
