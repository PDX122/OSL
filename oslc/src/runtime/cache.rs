use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Cache {
    pub name: String,
    pub kind: CacheKind,
    config: CacheConfig,
}

#[derive(Debug, Clone)]
pub enum CacheKind {
    Redis,
    Memory,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub host: String,
    pub port: u16,
    pub pass: Option<String>,
    pub db: u8,
    pub max_size: Option<usize>,
    pub eviction: EvictionPolicy,
    pub timeout: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    TTL,
    Random,
}

impl Cache {
    pub fn redis(config: CacheConfig) -> Self {
        Cache {
            name: "redis".to_string(),
            kind: CacheKind::Redis,
            config,
        }
    }

    pub fn memory(max_size: usize, eviction: EvictionPolicy) -> Self {
        Cache {
            name: "memory".to_string(),
            kind: CacheKind::Memory,
            config: CacheConfig {
                host: String::new(),
                port: 0,
                pass: None,
                db: 0,
                max_size: Some(max_size),
                eviction,
                timeout: 5,
            },
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<CacheValue>, CacheError> {
        Err(CacheError::NotConnected)
    }

    pub async fn set(&self, key: &str, value: CacheValue, ttl: Option<u64>) -> Result<(), CacheError> {
        Err(CacheError::NotConnected)
    }

    pub async fn delete(&self, key: &str) -> Result<bool, CacheError> {
        Err(CacheError::NotConnected)
    }

    pub async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        Err(CacheError::NotConnected)
    }

    pub async fn expire(&self, key: &str, ttl: u64) -> Result<bool, CacheError> {
        Err(CacheError::NotConnected)
    }

    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64, CacheError> {
        Err(CacheError::NotConnected)
    }

    pub async fn flush(&self) -> Result<(), CacheError> {
        Err(CacheError::NotConnected)
    }
}

#[derive(Debug, Clone)]
pub enum CacheValue {
    String(String),
    Int(i64),
    Float(f64),
    Bytes(Vec<u8>),
    Json(serde_json::Value),
}

#[derive(Debug)]
pub enum CacheError {
    NotConnected,
    KeyNotFound,
    SerializationFailed(String),
    Timeout,
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotConnected => write!(f, "Cache not connected"),
            CacheError::KeyNotFound => write!(f, "Key not found"),
            CacheError::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            CacheError::Timeout => write!(f, "Cache operation timed out"),
        }
    }
}

pub struct MemoryCache {
    data: HashMap<String, CacheEntry>,
    max_size: usize,
    eviction: EvictionPolicy,
}

struct CacheEntry {
    value: CacheValue,
    created: Instant,
    ttl: Option<Duration>,
    access_count: usize,
}

impl MemoryCache {
    pub fn new(max_size: usize, eviction: EvictionPolicy) -> Self {
        MemoryCache {
            data: HashMap::new(),
            max_size,
            eviction,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<CacheValue> {
        if let Some(entry) = self.data.get_mut(key) {
            if let Some(ttl) = entry.ttl {
                if entry.created.elapsed() > ttl {
                    self.data.remove(key);
                    return None;
                }
            }
            entry.access_count += 1;
            Some(entry.value.clone())
        } else {
            None
        }
    }

    pub fn set(&mut self, key: &str, value: CacheValue, ttl: Option<u64>) {
        if self.data.len() >= self.max_size {
            self.evict();
        }
        
        self.data.insert(key.to_string(), CacheEntry {
            value,
            created: Instant::now(),
            ttl: ttl.map(Duration::from_secs),
            access_count: 0,
        });
    }

    fn evict(&mut self) {
        let key_to_remove = match self.eviction {
            EvictionPolicy::LRU => {
                self.data.iter().min_by_key(|(_, e)| e.created).map(|(k, _)| k.clone())
            }
            EvictionPolicy::LFU => {
                self.data.iter().min_by_key(|(_, e)| e.access_count).map(|(k, _)| k.clone())
            }
            EvictionPolicy::TTL => {
                self.data.iter().min_by_key(|(_, e)| e.created).map(|(k, _)| k.clone())
            }
            EvictionPolicy::Random => {
                self.data.keys().next().cloned()
            }
        };
        if let Some(key) = key_to_remove {
            self.data.remove(&key);
        }
    }
}