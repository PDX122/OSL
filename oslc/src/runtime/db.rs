use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DbPool {
    pub name: String,
    pub kind: DbKind,
    pub config: DbConfig,
}

#[derive(Debug, Clone)]
pub enum DbKind {
    Postgres,
    MySQL,
    SQLite,
}

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub pass: String,
    pub pool_min: usize,
    pub pool_max: usize,
    pub timeout: u64,
    pub ssl: bool,
}

impl DbPool {
    pub fn postgres(config: DbConfig) -> Self {
        DbPool {
            name: "postgres".to_string(),
            kind: DbKind::Postgres,
            config,
        }
    }

    pub fn mysql(config: DbConfig) -> Self {
        DbPool {
            name: "mysql".to_string(),
            kind: DbKind::MySQL,
            config,
        }
    }

    pub fn sqlite(path: &str) -> Self {
        DbPool {
            name: "sqlite".to_string(),
            kind: DbKind::SQLite,
            config: DbConfig {
                host: "localhost".to_string(),
                port: 0,
                name: path.to_string(),
                user: String::new(),
                pass: String::new(),
                pool_min: 1,
                pool_max: 10,
                timeout: 10,
                ssl: false,
            },
        }
    }

    pub async fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>, DbError> {
        Err(DbError::NotConnected)
    }

    pub async fn execute(&self, sql: &str) -> Result<u64, DbError> {
        Err(DbError::NotConnected)
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Blob(Vec<u8>),
}

#[derive(Debug, Clone)]
pub enum DbError {
    NotConnected,
    QueryFailed(String),
    ConnectionFailed(String),
    PoolExhausted,
    Timeout,
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NotConnected => write!(f, "Database not connected"),
            DbError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            DbError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            DbError::PoolExhausted => write!(f, "Connection pool exhausted"),
            DbError::Timeout => write!(f, "Database operation timed out"),
        }
    }
}

pub struct Transaction {
    pool: DbPool,
}

impl Transaction {
    pub async fn begin(pool: &DbPool) -> Result<Transaction, DbError> {
        Ok(Transaction { pool: pool.clone() })
    }

    pub async fn commit(self) -> Result<(), DbError> {
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), DbError> {
        Ok(())
    }

    pub async fn query(&self, sql: &str) -> Result<Vec<HashMap<String, Value>>, DbError> {
        self.pool.query(sql).await
    }

    pub async fn execute(&self, sql: &str) -> Result<u64, DbError> {
        self.pool.execute(sql).await
    }
}

pub fn build_connection_string(config: &DbConfig) -> String {
    match config.host.as_str() {
        "" => format!("sqlite:{}", config.name),
        _ => format!(
            "postgres://{}:{}@{}:{}/{}",
            config.user, config.pass, config.host, config.port, config.name
        ),
    }
}