#[cfg(test)]
mod tests {
    use crate::{
        models::messages::{Message, WsState},
        router::chat::{get_room, send_message},
    };
    use axum::{
        Router,
        http::StatusCode,
        routing::{get, post},
    };
    use axum_test::{TestServer, TestServerConfig};
    use futures_util::{SinkExt, StreamExt};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tokio_tungstenite::{connect_async, tungstenite};

    async fn setup_test_server() -> TestServer {
        let state = Arc::new(WsState::new());
        let app = Router::new()
            .route("/get-room", get(get_room))
            .route("/send-message", post(send_message)) // Add send_message route
            .with_state(state);

        let mut config = TestServerConfig::default();
        config.transport = Some(axum_test::Transport::HttpRandomPort);

        TestServer::new_with_config(app, config).unwrap()
    }

    #[tokio::test]
    async fn test_websocket_full_suite() {
        let server = setup_test_server().await;

        let ws_url1 = format!(
            "ws://{}/get-room?user_id=user1&page_id=room1&email=user1@gmail.com&avatar_url=https://example.com/avatar1.jpg&username=UserOne",
            server.server_url("/get-room").unwrap().authority()
        );

        let ws_url2 = format!(
            "ws://{}/get-room?user_id=user2&page_id=room1&email=user2@gmail.com&avatar_url=https://example.com/avatar2.jpg&username=UserTwo",
            server.server_url("/get-room").unwrap().authority()
        );

        // Connect first user
        println!("will connect user 1");
        let (ws_stream1, _) = connect_async(&ws_url1).await.unwrap();
        let (mut write1, mut read1) = ws_stream1.split();
        println!("will connect user 2");
        let msg1 = read1.next().await.unwrap().unwrap().into_text().unwrap();
        let value1: Value = serde_json::from_str(&msg1).unwrap();
        assert_eq!(value1["event"], "get_room_users");
        assert!(value1["users"].as_array().unwrap().is_empty());

        let msg2 = read1.next().await.unwrap().unwrap().into_text().unwrap();
        let value2: Value = serde_json::from_str(&msg2).unwrap();
        assert_eq!(value2["event"], "join_room");
        assert_eq!(value2["user_id"].as_str().unwrap(), "user1");

        // Connect second user
        let (ws_stream2, _) = connect_async(&ws_url2).await.unwrap();
        let (write2, mut read2) = ws_stream2.split();

        let msg3 = read2.next().await.unwrap().unwrap().into_text().unwrap();
        let value3: Value = serde_json::from_str(&msg3).unwrap();
        assert_eq!(value3["event"], "get_room_users");
        assert_eq!(value3["users"].as_array().unwrap().len(), 1);
        assert_eq!(value3["users"][0]["user_id"].as_str().unwrap(), "user1");

        let msg4 = read2.next().await.unwrap().unwrap().into_text().unwrap();
        let value4: Value = serde_json::from_str(&msg4).unwrap();
        assert_eq!(value4["event"], "join_room");
        assert_eq!(value4["user_id"].as_str().unwrap(), "user2");

        let msg5 = read1.next().await.unwrap().unwrap().into_text().unwrap();
        let value5: Value = serde_json::from_str(&msg5).unwrap();
        assert_eq!(value5["event"], "join_room");
        assert_eq!(value5["user_id"].as_str().unwrap(), "user2");

        // Test Typing event from user1
        let typing_msg = json!({
            "event": "typing",
            "user_id": "user1",
            "username": "UserOne",
            "is_typing": true
        })
        .to_string();
        write1
            .send(tungstenite::Message::Text(typing_msg.into()))
            .await
            .unwrap();

        let msg6 = read2.next().await.unwrap().unwrap().into_text().unwrap();
        let value6: Value = serde_json::from_str(&msg6).unwrap();
        assert_eq!(value6["event"], "typing");
        assert_eq!(value6["user_id"].as_str().unwrap(), "user1");

        let msg7 = read1.next().await.unwrap().unwrap().into_text().unwrap();
        let value7: Value = serde_json::from_str(&msg7).unwrap();
        assert_eq!(value7["event"], "typing");
        assert_eq!(value7["user_id"].as_str().unwrap(), "user1");

        // Test LeaveRoom by dropping user2
        drop(write2);
        drop(read2);

        let msg8 = read1.next().await.unwrap().unwrap().into_text().unwrap();
        let value8: Value = serde_json::from_str(&msg8).unwrap();
        assert_eq!(value8["event"], "leave_room");
        assert_eq!(value8["user_id"].as_str().unwrap(), "user2");

        drop(write1);
        drop(read1);
    }

    #[tokio::test]
    async fn test_send_message() {
        let server = setup_test_server().await;

        let ws_url1 = format!(
            "ws://{}/get-room?user_id=user1&page_id=room1&email=user1@gmail.com&avatar_url=https://example.com/avatar1.jpg&username=UserOne",
            server.server_url("/get-room").unwrap().authority()
        );

        let ws_url2 = format!(
            "ws://{}/get-room?user_id=user2&page_id=room1&email=user2@gmail.com&avatar_url=https://example.com/avatar2.jpg&username=UserTwo",
            server.server_url("/get-room").unwrap().authority()
        );

        // Connect users
        let (ws_stream1, _) = connect_async(&ws_url1).await.unwrap();
        let (write1, mut read1) = ws_stream1.split();

        let (ws_stream2, _) = connect_async(&ws_url2).await.unwrap();
        let (write2, mut read2) = ws_stream2.split();

        // Drain initial messages for user1
        let _ = read1.next().await.unwrap().unwrap(); // get_room_users
        let _ = read1.next().await.unwrap().unwrap(); // user1 join_room
        let _ = read1.next().await.unwrap().unwrap(); // user2 join_room

        // Drain initial messages for user2
        let _ = read2.next().await.unwrap().unwrap(); // get_room_users
        let _ = read2.next().await.unwrap().unwrap(); // user2 join_room

        // Send a message via POST
        let message = Message {
            page_id: "room1".to_string(),
            user_id: "user1".to_string(),
            username: "UserOne".to_string(),
            avatar_url: "https://example.com/avatar1.jpg".to_string(),
            content: "Hello, room!".to_string(),
        };
        let response = server.post("/send-message").json(&message).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        // user1 receives the MessageReceived event
        let msg1 = read1.next().await.unwrap().unwrap().into_text().unwrap();
        let value1: Value = serde_json::from_str(&msg1).unwrap();
        assert_eq!(value1["event"], "message_received");
        assert_eq!(value1["user_id"].as_str().unwrap(), "user1");
        assert_eq!(value1["username"].as_str().unwrap(), "UserOne");
        assert_eq!(
            value1["avatar_url"].as_str().unwrap(),
            "https://example.com/avatar1.jpg"
        );
        assert_eq!(value1["content"].as_str().unwrap(), "Hello, room!");

        // user2 receives the MessageReceived event
        let msg2 = read2.next().await.unwrap().unwrap().into_text().unwrap();
        let value2: Value = serde_json::from_str(&msg2).unwrap();
        assert_eq!(value2["event"], "message_received");
        assert_eq!(value2["user_id"].as_str().unwrap(), "user1");
        assert_eq!(value2["username"].as_str().unwrap(), "UserOne");
        assert_eq!(
            value2["avatar_url"].as_str().unwrap(),
            "https://example.com/avatar1.jpg"
        );
        assert_eq!(value2["content"].as_str().unwrap(), "Hello, room!");

        // Clean up
        drop(write1);
        drop(read1);
        drop(write2);
        drop(read2);
    }
}
