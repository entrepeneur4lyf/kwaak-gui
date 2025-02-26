use std::sync::{Arc, Mutex};

use kwaak::{
    chat::Chat,
    commands::{CommandEvent, CommandResponse, Responder},
    repository::Repository,
};
use tokio::sync::mpsc;
use tokio_util::task::AbortOnDropHandle;

pub struct AppState {
    pub repository: Repository,
    pub command_tx: mpsc::UnboundedSender<CommandEvent>, // backend_handle: AbortOnDropHandle<()>,
    pub command_handle: AbortOnDropHandle<()>,
    pub command_responder: Option<Arc<AppCommandResponder>>,
    pub chats: Vec<Chat>, // chats: Chat,
}

impl AppState {
    pub fn dispatch(&self, event: CommandEvent) {
        self.command_tx.send(event).unwrap();
    }

    pub fn handle_response(&self, response: CommandResponse) {
        todo!()
    }

    pub fn responder(&self) -> Arc<dyn Responder> {
        self.command_responder
            .as_ref()
            .expect("app responder not started")
            .tx
            .clone() as Arc<dyn Responder>
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
        app.lock().unwrap().command_responder = Some(responder.into());
    }
}

pub struct AppCommandResponder {
    tx: Arc<mpsc::UnboundedSender<CommandResponse>>,
    _handle: AbortOnDropHandle<()>,
}
