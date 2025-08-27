use crossbeam_channel::{select, unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, warn};
use tungstenite::{accept, Message, Utf8Bytes};
use tungstenite::error::Error as WsError;
use crate::general_use::{ClientId, DroneId, FileRef, MediaRef, ServerId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

// Helper module for handling u64 as strings in JSON
mod stringified_u8 {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error;
    pub fn serialize<S: Serializer>(value: &u8, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsCommand {
    WsUpdateData,

    WsAskListRegisteredClientsToServer {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        #[serde(with = "stringified_u8")]
        server_id: ServerId,
    },

    WsSendMessage {
        #[serde(with = "stringified_u8")]
        source_client_id: ClientId,
        #[serde(with = "stringified_u8")]
        dest_client_id: ClientId,
        message: String,
    },

    WsAskFileList {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        #[serde(with = "stringified_u8")]
        server_id: ServerId,
    },

    WsAskFileContent {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        #[serde(with = "stringified_u8")]
        server_id: ServerId,
        file_ref: FileRef,
    },

    WsAskMedia {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        media_ref: MediaRef,
    },

    WsCrashDrone{
        #[serde(with = "stringified_u8")]
        drone_id: DroneId,
    }
}

// A type alias for the global list of client inboxes.
// Each client (internet WebSocket connection) gets its own Sender<String> for receiving broadcast updates.
type ClientList = Arc<Mutex<HashMap<u64, Sender<String>>>>;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

fn next_client_id() -> u64 {
    NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let clients: ClientList = Arc::new(Mutex::new(HashMap::new()));

    // Spawn broadcaster thread
    let broadcaster_clients = Arc::clone(&clients);
    thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            let client_senders: Vec<Sender<String>> = {
                let client_map = broadcaster_clients.lock().unwrap();
                client_map.values().cloned().collect()
            };
            for sender in client_senders {
                let _ = sender.send(data.clone());
            }
        }
    });

    // Spawn listener thread
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let cmd_tx = cmd_tx.clone();
                    let clients = Arc::clone(&clients);
                    let client_id = next_client_id();

                    thread::spawn(move || {
                        handle_client(stream, addr, client_id, clients, cmd_tx);   //from the stream we get the clients
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}

fn handle_client(
    stream: std::net::TcpStream,
    addr: std::net::SocketAddr,
    client_id: u64,
    clients: ClientList,
    cmd_tx: Sender<WsCommand>,
) {
    let mut websocket = match accept(stream) {   //here is the stream
        Ok(ws) => ws,
        Err(e) => {
            warn!("Failed to accept WebSocket handshake from {}: {}", addr, e);
            return;
        }
    };
    println!("Client connected: {} (id={})", addr, client_id);


    // Create a channel that will serve as this client's inbox for broadcast messages.
    let (client_tx, client_rx) = unbounded::<String>();

    // Add client to the map
    {
        let mut client_map = clients.lock().unwrap();
        client_map.insert(client_id, client_tx.clone());
    }

    websocket.get_mut().set_nonblocking(true).unwrap();

    // Main loop
    loop {
        select! {
            recv(client_rx) -> msg => {
                if let Ok(data) = msg {
                    if let Err(e) = websocket.send(Message::Text(Utf8Bytes::from(data))) {
                        warn!("Failed to send data to WebSocket (id={}): {}", client_id, e);
                        break;
                    }
                }
            },
            default => {
                match websocket.read() {
                    Ok(Message::Text(text)) => {
                        info!("WebSocket Message Received from id={}: {}", client_id, text);
                        if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                            info!("Parsed command from id={}: {:?}", client_id, cmd);
                            
                            // Send the command from the client in the frontend to the backend
                            if let Err(e) = cmd_tx.send(cmd.clone()) {
                                warn!("Failed to send command to backend from id={}: {}", client_id, e);
                                break;
                            }

                        } else {
                            info!("Failed to parse message from id={}: {}", client_id, text);
                        }
                    }
                    Err(WsError::Io(ref err)) if err.kind() == std::io::ErrorKind::WouldBlock => {},
                    Err(e) => {
                        warn!("WebSocket error (id={}): {}", client_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    println!("Client disconnected: {} (id={})", addr, client_id);

    { // Scoped mutex lock for removing client
        let mut client_map = clients.lock().unwrap();
        client_map.remove(&client_id);
    }
    
}
