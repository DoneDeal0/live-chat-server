use super::WsState;
use crate::models::messages::{ChatEvent, ChatUser, TypingEvent};
use axum::{
    extract::{Query, State, WebSocketUpgrade, ws::Message::Text},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, from_str, json};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast;

// #[debug_handler]
pub async fn get_room(
    State(state): State<Arc<WsState>>,
    ws: WebSocketUpgrade,
    Query(params): Query<ChatUser>,
) -> impl IntoResponse {
    let state = state.clone();
    ws.on_upgrade(move |socket| async move {
        let mut rooms = state.rooms.lock().await;

        let room = rooms
            .entry(params.page_id.clone())
            .or_insert_with(|| broadcast::channel(16).0)
            .clone();

        let mut room_listener = room.subscribe();
        let mut users = state.connected_users.lock().await;

        let room_users = users
            .entry(params.page_id.clone())
            .or_insert_with(HashMap::new);

        room_users.insert(
            params.user_id.clone(),
            ChatUser {
                user_id: params.user_id.clone(),
                page_id: params.page_id.clone(),
                username: params.username.clone(),
                avatar_url: params.avatar_url.clone(),
                email: params.email.clone(),
            },
        );
        let existing_users: Vec<ChatUser> = room_users
            .values()
            .filter(|chat_user| chat_user.user_id != params.user_id)
            .map(|chat_user| chat_user.clone())
            .collect();

        let room_users_payload = json!({
            "event": ChatEvent::GetRoomUsers,
            "users": existing_users
        })
        .to_string();

        let join_msg = json!({
            "event": ChatEvent::JoinRoom,
            "user_id": &params.user_id,
            "username": &params.username,
            "email": &params.email,
            "avatar_url": &params.avatar_url,
        })
        .to_string();

        // the socket is only bound to the client, not the whole room
        let (mut ws_sink, mut ws_stream) = socket.split();
        ws_sink.send(Text(room_users_payload.into())).await.ok();
        room.send(join_msg).ok();
        drop(rooms);
        drop(users);

        let state_clone = state.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_stream.next().await {
                if let Ok(text) = msg.into_text() {
                    let value: Value = from_str(&text).unwrap_or_default();
                    match value.get("event").and_then(|v| v.as_str()) {
                        Some("typing") => {
                            if let Ok(data) = from_str::<TypingEvent>(&text) {
                                let typing_msg = json!({
                                    "event": ChatEvent::Typing,
                                    "user_id": data.user_id,
                                    "username": data.username,
                                    "is_typing": data.is_typing
                                })
                                .to_string();
                                let rooms = state_clone.rooms.lock().await;
                                if let Some(room) = rooms.get(&params.page_id) {
                                    room.send(typing_msg).ok();
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            // handle deconnection
            let rooms = state_clone.rooms.lock().await;
            let mut users = state_clone.connected_users.lock().await;
            if let Some(tx) = rooms.get(&params.page_id) {
                let leave_msg = json!({
                    "event": ChatEvent::LeaveRoom,
                    "user_id": params.user_id.clone()
                })
                .to_string();
                tx.send(leave_msg).ok();
            }
            if let Some(user_list) = users.get_mut(&params.page_id) {
                user_list.remove(&params.user_id);
                if user_list.is_empty() {
                    users.remove(&params.page_id);
                }
            }
        });

        while let Ok(msg) = room_listener.recv().await {
            ws_sink.send(Text(msg.into())).await.ok();
        }
    })
}
