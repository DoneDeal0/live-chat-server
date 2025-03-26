# LIVE CHAT SERVER

Live chat server in rust for a hackaton project.

## INSTALLATION

You need to install Rust and Cargo:

```bash
curl https://sh.rustup.rs -sSf | sh
```

## USAGE

There are 3 routes:

### Http

> send a message to the room

```bash
POST "/chat/send-message"
{
    avatar_url: String,
    content: String,
    page_id: String,
    user_id: String,
    username: String,
}
```

> check if the server is up and running

```bash
GET "/check-health"
```

### Websocket

> connect to a room and subscribe to events

```bash
GET "/chat/get-room?avatar_url=url&email=email&page_id=pid&user_id=uid&username=name"
```

The available events are:

```rust
enum ChatEvent {
    GetRoomUsers = "get_room_users"
    JoinRoom = "join_room"
    LeaveRoom = "leave_room"
    MessageReceived = "message_received"
    MessageSent = "message_sent"
    Typing = "typing"
}
```

## TEST

```bash
cargo test
```

## RUN

```bash
cargo run
```

## BUILD

```bash
cargo build
```

<hr/>

### CAVEATS

- Some events can be broadcasted back to the sender. You may need to filter your own data on the frontend based on the `user_id`.

- This project is not secure. It allows requests from all origins and doesn't authenticate them with a cookie. It was just for a hackathon. I may improve it later.
