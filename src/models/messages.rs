use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, broadcast};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Message {
    pub avatar_url: String,
    pub content: String,
    pub page_id: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserEvent {
    pub page_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TypingEvent {
    pub event: String,
    pub is_typing: bool,
    pub user_id: String,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
pub enum ChatEvent {
    #[serde(rename = "get_room_users")]
    GetRoomUsers,
    #[serde(rename = "join_room")]
    JoinRoom,
    #[serde(rename = "leave_room")]
    LeaveRoom,
    #[serde(rename = "message_received")]
    MessageReceived,
    #[serde(rename = "message_sent")]
    MessageSent,
    #[serde(rename = "typing")]
    Typing,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ChatUser {
    pub avatar_url: String,
    pub email: String,
    pub page_id: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Clone)]
pub struct WsState {
    pub connected_users: Arc<Mutex<HashMap<String, HashMap<String, ChatUser>>>>,
    pub rooms: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl WsState {
    pub fn new() -> Self {
        WsState {
            connected_users: Arc::new(Mutex::new(HashMap::new())),
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
