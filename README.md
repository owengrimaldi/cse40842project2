# LFG - Game Chat Rooms
LFG Chat Rooms Project for CSE 40842, written in Rust using tokio.

![](/demo2.png)

## Dependencies (Cargo.toml)
This project uses the following dependencies:
```
  futures-util = { version = "0.3.30", features = ["sink"] }
  http = "1.1.0"
  tokio = { version = "1.40.0", features = ["full"] }
  tokio-websockets = { version = "0.10.1", features = ["client", "fastrand", "server", "sha1_smol"] }
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  warp = "0.3"
  tokio-tungstenite = "0.18"
```

## Setup
Clone or download the repository and access its root. Run the following command:
```
  cargo build --release
```
Currently, this server is set up to run on a specific AWS EC2 instance. To run it on your own instance, edit the Websocket connections in the server.rs, client.rs, and client.html file.

## Usage
Run the project with the following command:
```
  cargo run --bin server
```


## Controls
```
Use the 'Join' button to join an already-created room.
Use the 'Create' button to create a new room.
Type in the message box and 'Enter' to send messages.
```
