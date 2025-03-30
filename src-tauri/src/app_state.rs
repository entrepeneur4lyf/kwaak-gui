/// Application state management
///
///
/// Event flow: ui -> frontend command(uuid, channel(CommandResponseWithUuid)) -> backend command
/// dispatched(responder) -> responder forwards to app commands and mirrors to frontend
///
/// Ensures that the backend state is always correct, frontend can show live updates
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use kwaak::{
    chat::Chat,
    commands::{CommandEvent, CommandResponse, Responder},
    repository::Repository,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::task::AbortOnDropHandle;
use uuid::Uuid;

use crate::actions;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponseWithUuid {
    session_id: Uuid,
    #[serde(with = "CommandResponseDef", flatten)]
    content: CommandResponse,
}

/// Wraps Kwaak command response so we can control how it serializes
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(remote = "CommandResponse", tag = "event")]
pub enum CommandResponseDef {
    /// Messages coming from an agent
    Chat(swiftide::chat_completion::ChatMessage),
    /// Short activity updates
    Activity(String),
    /// A chat has been renamed
    RenameChat(String),
    /// A chat branch has been renamed
    RenameBranch(String),
    /// Backend system messages (kwaak currently just renders these as system chat like messages)
    BackendMessage(String),
    /// A command has been completed
    Completed,
}

pub struct AppState {
    pub repository: Repository,
    pub command_tx: mpsc::UnboundedSender<CommandEvent>, // backend_handle: AbortOnDropHandle<()>,
    pub command_handle: AbortOnDropHandle<()>,
    pub command_responder: Option<AppCommandResponder>,
    pub chats: Vec<Chat>, // chats: Chat,
}

impl AppState {
    pub fn dispatch(&self, event: CommandEvent) {
        self.command_tx.send(event).unwrap();
    }

    pub fn handle_response(&mut self, response: CommandResponseWithUuid) -> Result<()> {
        let uuid = response.session_id;
        match response.content {
            // New message is received in a chat
            CommandResponse::Chat(msg) => actions::chat_message(self, uuid, msg),
            // An update message on a running command (short string messages, like running
            // completions, indexing code, etc)
            CommandResponse::Activity(state) => actions::activity_update(self, uuid, &state),
            // A chat has been renamed
            CommandResponse::RenameChat(name) => actions::rename_chat(self, uuid, name),
            // A git branch has been renamed
            CommandResponse::RenameBranch(name) => actions::rename_branch(self, uuid, name),
            // A running command in a chat session has completed. This means it's ready for user
            // interaction
            CommandResponse::Completed => actions::completed(self, uuid),
            // A message from the backend has been received
            CommandResponse::BackendMessage(msg) => actions::backend_message(self, uuid, msg),
        }
    }

    pub fn responder_from_channel(
        &self,
        uuid: Uuid,
        channel: tauri::ipc::Channel<CommandResponseWithUuid>,
    ) -> Arc<dyn Responder> {
        let app_responder = self
            .command_responder
            .as_ref()
            .expect("app responder not started")
            .tx
            .clone();

        Arc::new(TauriCommandResponder(uuid, channel, app_responder))
    }

    /// Starts the receiving loop to handle responses from the backend
    pub fn spawn_responder(app: &Arc<Mutex<AppState>>) {
        let app_for_loop = Arc::clone(app);
        let (tx, rx) = mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            let mut rx = rx;
            while let Some(response) = rx.recv().await {
                app_for_loop.lock().unwrap().handle_response(response);
            }
        });

        let responder = AppCommandResponder {
            tx: Arc::new(tx),
            _handle: AbortOnDropHandle::new(handle),
        };
        app.lock().unwrap().command_responder = Some(responder);
    }
}

pub struct AppCommandResponder {
    tx: Arc<mpsc::UnboundedSender<CommandResponseWithUuid>>,
    _handle: AbortOnDropHandle<()>,
}

// Wraps a backend and frontend tx so that `CommandResponse` can be mirrored to the frontend for
// live updates
//
// It wraps the response with the UUID of the chat session
#[derive(Clone)]
pub struct TauriCommandResponder(
    uuid::Uuid,
    tauri::ipc::Channel<CommandResponseWithUuid>,
    Arc<mpsc::UnboundedSender<CommandResponseWithUuid>>,
);

#[async_trait]
impl Responder for TauriCommandResponder {
    async fn send(&self, response: CommandResponse) {
        let _ = self.1.send(CommandResponseWithUuid {
            session_id: self.0,
            content: response.clone(),
        });
        let _ = self.2.send(CommandResponseWithUuid {
            session_id: self.0,
            content: response.clone(),
        });
    }
}

impl std::fmt::Debug for TauriCommandResponder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriCommandResponder")
            .field("uuid", &self.0)
            .finish()
    }
}
