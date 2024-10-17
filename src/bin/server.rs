use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{channel, Sender};
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};
use std::sync::{Arc, Mutex};

type Rooms = Arc<Mutex<HashMap<String, Sender<String>>>>; // Room name to broadcast sender

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    rooms: Rooms,
    user_map: Arc<Mutex<HashMap<SocketAddr, String>>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Receive username from client
    if let Some(Ok(msg)) = ws_stream.next().await {
        if let Some(username) = msg.as_text() {
            user_map.lock().unwrap().insert(addr, username.to_string());
            
            // Main loop to handle room joining and message sending
            let mut current_room: Option<String> = None;

            loop {
                // Send available rooms
                let room_list: Vec<String> = rooms.lock().unwrap().keys().cloned().collect();
                let room_list_json = serde_json::to_string(&room_list)?; // Convert room list to JSON
                ws_stream.send(Message::text(format!("rooms:{}", room_list_json))).await?;

                // Notify user about the application
                ws_stream.send(Message::text(format!("Welcome to LFG, {}! Create or join a room and send messages!", username))).await?;

                // Default to a room if none is selected (could be improved later)
                let room_name = "General".to_string();

                // Create a new room if it doesn't exist
                let bcast_tx = rooms.lock().unwrap().entry(room_name.clone()).or_insert_with(|| {
                    let (tx, _) = channel(16); // Create the channel
                    tx // Insert only the Sender into the HashMap
                }).clone();

                let mut bcast_rx = bcast_tx.subscribe();
                current_room = Some(room_name.clone()); // Set current room for this user

                ws_stream.send(Message::text(format!("Welcome to the room: {}", room_name))).await?;

                loop {
                    tokio::select! {
                        incoming = ws_stream.next() => {
                            match incoming {
                                Some(Ok(msg)) => {
                                    if let Some(text) = msg.as_text() {
                                        println!("From client {addr:?} ({username}): {text:?}");

                                        // Check if the message is a command or a text message
                                        if text.starts_with("/create ") {
                                            let new_room = text.strip_prefix("/create ").unwrap_or(text);
                                            let _ = rooms.lock().unwrap().entry(new_room.to_string()).or_insert_with(|| {
                                                let (tx, _) = channel(16);
                                                tx
                                            });
                                            ws_stream.send(Message::text(format!("Room created: {}", new_room))).await?;
                                        } else if text.starts_with("/join ") {
                                            let room_to_join = text.strip_prefix("/join ").unwrap_or(text).to_string();
                                            if current_room.as_deref() != Some(&room_to_join) {
                                                println!("Switching from room {} to room {}", current_room.as_deref().unwrap_or("None"), room_to_join);
                                                
                                                let previous_room = current_room.take().unwrap_or_else(|| "General".to_string());
                                        
                                                // Send message that the user has left the previous room
                                                if let Some(previous_tx) = rooms.lock().unwrap().get(&previous_room) {
                                                    let _ = previous_tx.send(format!("{} has left the room.", username));
                                                }
                                        
                                                // Join the new room
                                                let bcast_tx_new = rooms.lock().unwrap().entry(room_to_join.clone()).or_insert_with(|| {
                                                    let (tx, _) = channel(32);
                                                    tx
                                                }).clone();
                                        
                                                println!("Subscribing to room: {}", room_to_join);
                                                
                                                bcast_rx = bcast_tx_new.subscribe();
                                                current_room = Some(room_to_join.clone());
                                        
                                                // Send a message announcing that the user has joined the room
                                                let _ = bcast_tx_new.send(format!("{} has joined the room.", username));
                                        
                                                ws_stream.send(Message::text(format!("Changing to room: {}", room_to_join))).await?;
                                            } else {
                                                ws_stream.send(Message::text(format!("You are already in the room: {}", room_to_join))).await?;
                                            }
                                        } else {
                                            // Broadcast from the correct room
                                            if let Some(ref room_name) = current_room {
                                                if let Some(tx) = rooms.lock().unwrap().get(room_name) {
                                                    tx.send(format!("{}: {}", username, text))?;
                                                }
                                            }
                                        }
                                    }
                                }
                                Some(Err(err)) => return Err(err.into()),
                                None => return Ok(()),
                            }
                        }
                        msg = bcast_rx.recv() => {
                            ws_stream.send(Message::text(msg?)).await?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let user_map: Arc<Mutex<HashMap<SocketAddr, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("ec2-3-21-236-80.us-east-2.compute.amazonaws.com:2000").await?;
    println!("listening on port 2000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let rooms = Arc::clone(&rooms);
        let user_map = Arc::clone(&user_map);
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            handle_connection(addr, ws_stream, rooms, user_map).await
        });
    }
}