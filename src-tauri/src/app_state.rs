use std::sync::{Arc, Mutex};

use kwaak::{
    chat::Chat,
    commands::{CommandEvent, CommandResponse},
    repository::Repository,
};
use tokio::sync::mpsc;
use tokio_util::task::AbortOnDropHandle;

pub struct AppState {
    pub repository: Repository,
    pub command_tx: mpsc::UnboundedSender<CommandEvent>, // backend_handle: AbortOnDropHandle<()>,
    pub command_handle: AbortOnDropHandle<()>,
    pub command_responder: AppCommandResponder,
    pub chats: Vec<Chat>, // chats: Chat,
}

impl AppState {
    pub fn dispatch(&self, event: CommandEvent) {
        self.command_tx.send(event).unwrap();
    }

    pub fn handle_response(&self, response: CommandResponse) {
        todo!()
    }
}

pub struct AppCommandResponder {
    tx: mpsc::UnboundedSender<CommandResponse>,
    _handle: AbortOnDropHandle<()>,
}
pub fn spawn_app_command_responder(app: &Arc<Mutex<AppState>>) -> AppCommandResponder {
    let app = Arc::clone(app);
    let (tx, rx) = mpsc::unbounded_channel();
    let handle = tokio::spawn(async move {
        let mut rx = rx;
        while let Some(response) = rx.recv().await {
            app.lock().unwrap().handle_response(response);
        }
    });

    AppCommandResponder {
        tx,
        _handle: AbortOnDropHandle::new(handle),
    }
}
