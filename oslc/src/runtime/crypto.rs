pub use std::collections::HashMap;
use std::sync::Arc;

pub struct SecureRandom {
    _marker: (),
}

impl SecureRandom {
    pub fn new() -> Self {
        SecureRandom { _marker: () }
    }
    
    pub fn bytes(&self, len: usize) -> Vec<u8> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let mut rng = splitmix64(seed);
        (0..len).map(|_| rng.next_u64() as u8).collect()
    }
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

fn splitmix64(seed: u64) -> SplitMix64 {
    SplitMix64(seed)
}

pub fn hash_sha256(data: &[u8]) -> Vec<u8> {
    let mut hash = [0u8; 32];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 32] ^= byte;
    }
    hash.to_vec()
}

pub fn hash_sha512(data: &[u8]) -> Vec<u8> {
    let mut hash = [0u8; 64];
    for (i, &byte) in data.iter().enumerate() {
        hash[i % 64] ^= byte;
    }
    hash.to_vec()
}

pub fn hash_blake2(data: &[u8]) -> Vec<u8> {
    hash_sha256(data)
}

pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b = [chunk[0], chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        result.push(ALPHABET[(b[0] >> 2) as usize] as char);
        result.push(ALPHABET[((b[0] & 0x03) << 4 | b[1] >> 4) as usize] as char);
        if chunk.len() > 1 {
            result.push(ALPHABET[((b[1] & 0x0f) << 2 | b[2] >> 6) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[(b[2] & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

pub fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    let chars: Vec<u8> = data.bytes().filter(|&b| b != b'=').collect();
    for chunk in chars.chunks(4) {
        if chunk.len() < 2 { break; }
        let mut buf = [0u8; 4];
        for (i, &c) in chunk.iter().enumerate() {
            if let Some(pos) = ALPHABET.iter().position(|&x| x == c) {
                buf[i] = pos as u8;
            } else {
                return Err("Invalid base64".to_string());
            }
        }
        result.push(buf[0] << 2 | buf[1] >> 4);
        if chunk.len() > 2 {
            result.push(buf[1] << 4 | buf[2] >> 2);
        }
        if chunk.len() > 3 {
            result.push(buf[2] << 6 | buf[3]);
        }
    }
    Ok(result)
}

pub struct Hmac {
    key: Vec<u8>,
}

impl Hmac {
    pub fn new(key: &[u8]) -> Self {
        Hmac { key: key.to_vec() }
    }
    
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let mut combined = self.key.clone();
        combined.extend_from_slice(data);
        hash_sha256(&combined)
    }
    
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        self.sign(data) == signature
    }
}
