use super::WsState;
use crate::models::messages::{ChatEvent, Message};
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;

// #[debug_handler]
pub async fn send_message(
    State(state): State<Arc<WsState>>,
    Json(message): Json<Message>,
) -> StatusCode {
    // todo: save to DB
    let rooms = state.rooms.lock().await;
    if let Some(tx) = rooms.get(&message.page_id) {
        let msg = json!({
            "event": ChatEvent::MessageReceived,
            "user_id": message.user_id,
            "username": message.username,
            "avatar_url": message.avatar_url,
            "content": message.content
        })
        .to_string();
        tx.send(msg).ok();
    }
    StatusCode::OK
}
