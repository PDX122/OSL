use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Redis {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl Redis {
    pub fn new() -> Self {
        Redis {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn connect(_url: &str) -> Self {
        Self::new()
    }
    
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.lock().unwrap().get(key).cloned()
    }
    
    pub fn set(&self, key: &str, value: &str) -> bool {
        self.data.lock().unwrap().insert(key.to_string(), value.to_string());
        true
    }
    
    pub fn del(&self, key: &str) -> bool {
        self.data.lock().unwrap().remove(key).is_some()
    }
    
    pub fn exists(&self, key: &str) -> bool {
        self.data.lock().unwrap().contains_key(key)
    }
    
    pub fn incr(&self, key: &str) -> i64 {
        let mut data = self.data.lock().unwrap();
        let current: i64 = data.get(key).and_then(|v| v.parse().ok()).unwrap_or(0);
        let new_val = current + 1;
        data.insert(key.to_string(), new_val.to_string());
        new_val
    }
    
    pub fn decr(&self, key: &str) -> i64 {
        let mut data = self.data.lock().unwrap();
        let current: i64 = data.get(key).and_then(|v| v.parse().ok()).unwrap_or(0);
        let new_val = current - 1;
        data.insert(key.to_string(), new_val.to_string());
        new_val
    }
    
    pub fn expire(&self, _key: &str, _ttl: u64) -> bool {
        true
    }
    
    pub fn ttl(&self, _key: &str) -> i64 {
        -1
    }
    
    pub fn hset(&self, key: &str, field: &str, value: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        let hash_key = format!("{}:{}", key, field);
        data.insert(hash_key, value.to_string());
        true
    }
    
    pub fn hget(&self, key: &str, field: &str) -> Option<String> {
        let data = self.data.lock().unwrap();
        let hash_key = format!("{}:{}", key, field);
        data.get(&hash_key).cloned()
    }
    
    pub fn hgetall(&self, key: &str) -> HashMap<String, String> {
        let data = self.data.lock().unwrap();
        let prefix = format!("{}:", key);
        data.iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(k, v)| (k.strip_prefix(&prefix).unwrap().to_string(), v.clone()))
            .collect()
    }
    
    pub fn lpush(&self, key: &str, value: &str) -> usize {
        self.data.lock().unwrap().insert(format!("{}:lp:0", key), value.to_string());
        1
    }
    
    pub fn rpush(&self, key: &str, value: &str) -> usize {
        self.data.lock().unwrap().insert(format!("{}:rp:0", key), value.to_string());
        1
    }
    
    pub fn lrange(&self, key: &str, _start: i64, _stop: i64) -> Vec<String> {
        let data = self.data.lock().unwrap();
        let prefix = format!("{}:l:", key);
        let mut values: Vec<String> = data.iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v.clone())
            .collect();
        values.sort();
        values
    }
    
    pub fn publish(&self, _channel: &str, message: &str) -> i64 {
        println!("PUBLISH to channel: {}", message);
        1
    }
    
    pub fn flushdb(&self) -> bool {
        self.data.lock().unwrap().clear();
        true
    }
}
