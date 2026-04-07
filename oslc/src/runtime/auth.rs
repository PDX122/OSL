use std::collections::HashMap;

pub struct Authenticator {
    name: String,
    kind: AuthKind,
    config: AuthConfig,
}

#[derive(Debug, Clone)]
pub enum AuthKind {
    Jwt,
    Basic,
    ApiKey,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub secret: String,
    pub algorithm: JwtAlgorithm,
    pub expiry: u64,
    pub refresh_expiry: u64,
    pub issuer: String,
    pub audience: String,
    pub realm: String,
    pub users: HashMap<String, String>,
    pub header: String,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JwtAlgorithm {
    HS256,
    RS256,
    ES256,
}

impl Authenticator {
    pub fn jwt(config: AuthConfig) -> Self {
        Authenticator {
            name: "jwt".to_string(),
            kind: AuthKind::Jwt,
            config,
        }
    }

    pub fn basic(realm: &str, users: HashMap<String, String>) -> Self {
        Authenticator {
            name: "basic".to_string(),
            kind: AuthKind::Basic,
            config: AuthConfig {
                secret: String::new(),
                algorithm: JwtAlgorithm::HS256,
                expiry: 3600,
                refresh_expiry: 604800,
                issuer: String::new(),
                audience: String::new(),
                realm: realm.to_string(),
                users,
                header: String::new(),
                keys: Vec::new(),
            },
        }
    }

    pub fn api_key(header: &str, keys: Vec<String>) -> Self {
        Authenticator {
            name: "api_key".to_string(),
            kind: AuthKind::ApiKey,
            config: AuthConfig {
                secret: String::new(),
                algorithm: JwtAlgorithm::HS256,
                expiry: 0,
                refresh_expiry: 0,
                issuer: String::new(),
                audience: String::new(),
                realm: String::new(),
                users: HashMap::new(),
                header: header.to_string(),
                keys,
            },
        }
    }

    pub fn verify_jwt(&self, token: &str) -> Result<JwtClaims, AuthError> {
        Err(AuthError::NotImplemented)
    }

    pub fn verify_basic(&self, username: &str, password: &str) -> Result<BasicUser, AuthError> {
        if let Some(stored_pass) = self.config.users.get(username) {
            if stored_pass == password {
                return Ok(BasicUser { username: username.to_string() });
            }
        }
        Err(AuthError::InvalidCredentials)
    }

    pub fn verify_api_key(&self, key: &str) -> Result<bool, AuthError> {
        if self.config.keys.contains(&key.to_string()) {
            Ok(true)
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

#[derive(Debug, Clone)]
pub struct JwtClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct BasicUser {
    pub username: String,
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    NotImplemented,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::TokenExpired => write!(f, "Token expired"),
            AuthError::TokenInvalid => write!(f, "Token invalid"),
            AuthError::NotImplemented => write!(f, "Not implemented"),
        }
    }
}

pub fn verify_jwt_token(token: &str, secret: &str) -> Result<JwtClaims, AuthError> {
    Err(AuthError::NotImplemented)
}

pub fn generate_jwt_token(claims: &JwtClaims, secret: &str) -> Result<String, AuthError> {
    Err(AuthError::NotImplemented)
}

pub fn hash_password(password: &str) -> String {
    format!("hashed: {}", password)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    hash == &format!("hashed: {}", password)
}