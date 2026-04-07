use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

pub struct WebSocketRoom {
    name: String,
    members: Vec<String>,
}

impl WebSocketRoom {
    pub fn new(name: &str) -> Self {
        WebSocketRoom {
            name: name.to_string(),
            members: Vec::new(),
        }
    }
    
    pub fn join(&mut self, member: &str) {
        if !self.members.contains(&member.to_string()) {
            self.members.push(member.to_string());
        }
    }
    
    pub fn leave(&mut self, member: &str) {
        self.members.retain(|m| m != member);
    }
    
    pub fn broadcast(&self, message: &str) {
        println!("Broadcasting to room '{}': {}", self.name, message);
    }
    
    pub fn members(&self) -> &[String] {
        &self.members
    }
}

pub struct RoomManager {
    rooms: HashMap<String, WebSocketRoom>,
}

impl RoomManager {
    pub fn new() -> Self {
        RoomManager {
            rooms: HashMap::new(),
        }
    }
    
    pub fn create(&mut self, name: &str) -> &mut WebSocketRoom {
        self.rooms.insert(name.to_string(), WebSocketRoom::new(name));
        self.rooms.get_mut(name).unwrap()
    }
    
    pub fn join(&mut self, member: &str, room: &str) {
        if let Some(r) = self.rooms.get_mut(room) {
            r.join(member);
        }
    }
    
    pub fn leave(&mut self, member: &str, room: &str) {
        if let Some(r) = self.rooms.get_mut(room) {
            r.leave(member);
        }
    }
    
    pub fn broadcast(&self, room: &str, message: &str) {
        if let Some(r) = self.rooms.get(room) {
            r.broadcast(message);
        }
    }
}
