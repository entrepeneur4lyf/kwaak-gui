use anyhow::Result;
use kwaak::chat::ChatState;
use swiftide::chat_completion;
use uuid::Uuid;

use crate::app_state::AppState;

pub fn chat_message(
    state: &mut AppState,
    session_id: Uuid,
    message: chat_completion::ChatMessage,
) -> Result<()> {
    let Some(chat) = state.chats.iter_mut().find(|c| c.uuid == session_id) else {
        anyhow::bail!("chat not found");
    };

    let message = kwaak::chat_message::ChatMessage::from(message);

    chat.add_message(message);

    Ok(())
}

pub fn activity_update(
    state: &mut AppState,
    session_id: Uuid,
    state: chat_completion::ActivityState,
) -> Result<()> {
    //
    Ok(())
}

pub fn rename_chat(state: &mut AppState, session_id: Uuid, name: String) -> Result<()> {
    let Some(chat) = state.chats.iter_mut().find(|c| c.uuid == session_id) else {
        anyhow::bail!("chat not found");
    };

    chat.name = name;

    Ok(())
}

pub fn rename_branch(state: &mut AppState, session_id: Uuid, name: String) -> Result<()> {
    let Some(chat) = state.chats.iter_mut().find(|c| c.uuid == session_id) else {
        anyhow::bail!("chat not found");
    };
    chat.branch_name = Some(name);
    //
    Ok(())
}

pub fn completed(state: &mut AppState, session_id: Uuid) -> Result<()> {
    let Some(chat) = state.chats.iter_mut().find(|c| c.uuid == session_id) else {
        anyhow::bail!("chat not found");
    };

    chat.transition(ChatState::Ready);

    Ok(())
}

pub fn backend_message(session_id: Uuid, message: String) -> Result<()> {
    let Some(chat) = state.chats.iter_mut().find(|c| c.uuid == session_id) else {
        anyhow::bail!("chat not found");
    };

    chat.add_message(kwaak::chat_message::ChatMessage::new_system(message));

    Ok(())
}
