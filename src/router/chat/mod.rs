mod get_room;
mod send_message;
use axum::{
    Router,
    routing::{get, post},
};
use get_room::get_room;
use send_message::send_message;
use std::sync::Arc;

use crate::models::messages::WsState;

pub fn chat_routes() -> Router {
    let ws_state = WsState::new();
    Router::new()
        .route("/get-room", get(get_room))
        .route("/send-message", post(send_message))
        .with_state(Arc::new(ws_state))
}

#[cfg(test)]
mod tests;
