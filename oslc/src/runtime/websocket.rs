use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};

pub struct WebSocketServer {
    path: String,
    config: WsConfig,
    handlers: WsHandlers,
}

#[derive(Debug, Clone)]
pub struct WsConfig {
    pub max_message_size: usize,
    pub ping_interval: u64,
    pub ping_timeout: u64,
    pub compression: bool,
}

#[derive(Clone)]
pub struct WsHandlers {
    pub on_connect: Option<Arc<dyn Fn(WsConnection) + Send + Sync>>,
    pub on_message: Option<Arc<dyn Fn(WsConnection, WsMessage) + Send + Sync>>,
    pub on_disconnect: Option<Arc<dyn Fn(WsConnection, u16) + Send + Sync>>,
    pub on_error: Option<Arc<dyn Fn(WsConnection, WsError) + Send + Sync>>,
}

pub struct WsConnection {
    id: String,
    remote_addr: String,
    is_open: bool,
    data: HashMap<String, String>,
}

impl WsConnection {
    pub fn new(id: &str, remote_addr: &str) -> Self {
        WsConnection {
            id: id.to_string(),
            remote_addr: remote_addr.to_string(),
            is_open: true,
            data: HashMap::new(),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn remote_addr(&self) -> &str {
        &self.remote_addr
    }

    pub fn send(&mut self, msg: WsMessage) -> Result<(), WsError> {
        if !self.is_open {
            return Err(WsError::ConnectionClosed);
        }
        Ok(())
    }

    pub fn send_text(&mut self, text: &str) -> Result<(), WsError> {
        self.send(WsMessage::Text(text.to_string()))
    }

    pub fn send_json(&mut self, value: serde_json::Value) -> Result<(), WsError> {
        let text = serde_json::to_string(&value).map_err(|e| WsError::Serialization(e.to_string()))?;
        self.send(WsMessage::Text(text))
    }

    pub fn close(&mut self, code: u16) -> Result<(), WsError> {
        self.is_open = false;
        Ok(())
    }

    pub fn set_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
}

#[derive(Debug, Clone)]
pub enum WsMessage {
    Text(String),
    Binary(Vec<u8>),
    Close(Option<u16>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
}

impl WsMessage {
    pub fn text(&self) -> Option<&str> {
        match self {
            WsMessage::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn json(&self) -> serde_json::Value {
        match self {
            WsMessage::Text(s) => serde_json::from_str(s).unwrap_or(serde_json::Value::Null),
            WsMessage::Binary(b) => serde_json::from_slice(b).unwrap_or(serde_json::Value::Null),
            _ => serde_json::Value::Null,
        }
    }
}

#[derive(Debug)]
pub enum WsError {
    ConnectionClosed,
    SendFailed(String),
    Serialization(String),
    Protocol(String),
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::ConnectionClosed => write!(f, "Connection closed"),
            WsError::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            WsError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            WsError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

pub struct Room {
    name: String,
    members: Vec<String>,
}

impl Room {
    pub fn new(name: &str) -> Self {
        Room {
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add(&mut self, conn_id: &str) {
        if !self.members.contains(&conn_id.to_string()) {
            self.members.push(conn_id.to_string());
        }
    }

    pub fn remove(&mut self, conn_id: &str) {
        self.members.retain(|id| id != conn_id);
    }

    pub fn broadcast(&self, msg: WsMessage) {
        for member in &self.members {
            println!("Broadcasting to {}", member);
        }
    }

    pub fn members(&self) -> &[String] {
        &self.members
    }
}

pub struct RoomManager {
    rooms: HashMap<String, Room>,
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: HashMap::new(),
        }
    }

    pub fn join(&mut self, conn_id: &str, room_name: &str) {
        self.rooms
            .entry(room_name.to_string())
            .or_insert_with(|| Room::new(room_name))
            .add(conn_id);
    }

    pub fn leave(&mut self, conn_id: &str, room_name: &str) {
        if let Some(room) = self.rooms.get_mut(room_name) {
            room.remove(conn_id);
        }
    }

    pub fn leave_all(&mut self, conn_id: &str) {
        for room in self.rooms.values_mut() {
            room.remove(conn_id);
        }
    }

    pub fn broadcast(&self, room_name: &str, msg: WsMessage) {
        if let Some(room) = self.rooms.get(room_name) {
            room.broadcast(msg);
        }
    }
}