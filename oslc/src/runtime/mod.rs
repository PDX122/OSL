use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub mod http;
pub mod db;
pub mod cache;
pub mod logging;
pub mod auth;
pub mod websocket;

pub use http::*;
pub use db::*;
pub use cache::*;
pub use logging::*;
pub use auth::*;
pub use websocket::*;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub backlog: i32,
    pub timeout: u64,
    pub keep_alive: bool,
    pub max_connections: usize,
    pub graceful_shutdown: bool,
    pub shutdown_timeout: u64,
    pub restart_on_crash: bool,
    pub pid_file: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            name: "osl-server".to_string(),
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            backlog: 128,
            timeout: 30,
            keep_alive: true,
            max_connections: 10000,
            graceful_shutdown: true,
            shutdown_timeout: 15,
            restart_on_crash: false,
            pid_file: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub version: HttpVersion,
    pub compression: Compression,
    pub max_body_size: usize,
    pub max_headers: usize,
    pub header_timeout: u64,
    pub body_timeout: u64,
    pub idle_timeout: u64,
    pub pipeline: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        HttpConfig {
            version: HttpVersion::Http1_1,
            compression: Compression::None,
            max_body_size: 10 * 1024 * 1024,
            max_headers: 100,
            header_timeout: 10,
            body_timeout: 30,
            idle_timeout: 60,
            pipeline: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpVersion {
    Http1_0,
    Http1_1,
    Http2,
    Http3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Compression {
    None,
    Gzip,
    Brotli,
    Zstd,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub ip: SocketAddr,
    pub user_agent: Option<String>,
    pub protocol: String,
    pub port: u16,
    pub host: String,
    pub timestamp: i64,
    pub id: String,
}

impl Request {
    pub fn body_json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).unwrap_or(serde_json::Value::Null)
    }

    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Response {
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn json(mut self, data: serde_json::Value) -> Self {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = serde_json::to_vec(&data).unwrap_or_default();
        self
    }

    pub fn text(mut self, text: &str) -> Self {
        self.headers.insert("Content-Type".to_string(), "text/plain".to_string());
        self.body = text.as_bytes().to_vec();
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: Arc<dyn Fn(Request) -> Response + Send + Sync>,
}

pub struct Router {
    routes: Vec<Route>,
    middleware: Vec<Arc<dyn Fn(Request) -> Result<Request, Response> + Send + Sync>>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    pub fn get(&mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> &mut Self {
        self.routes.push(Route {
            method: "GET".to_string(),
            path: path.to_string(),
            handler: Arc::new(handler),
        });
        self
    }

    pub fn post(&mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> &mut Self {
        self.routes.push(Route {
            method: "POST".to_string(),
            path: path.to_string(),
            handler: Arc::new(handler),
        });
        self
    }

    pub fn put(&mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> &mut Self {
        self.routes.push(Route {
            method: "PUT".to_string(),
            path: path.to_string(),
            handler: Arc::new(handler),
        });
        self
    }

    pub fn delete(&mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> &mut Self {
        self.routes.push(Route {
            method: "DELETE".to_string(),
            path: path.to_string(),
            handler: Arc::new(handler),
        });
        self
    }

    pub fn add_middleware(&mut self, middleware: impl Fn(Request) -> Result<Request, Response> + Send + Sync + 'static) -> &mut Self {
        self.middleware.push(Arc::new(middleware));
        self
    }

    pub fn handle(&self, mut req: Request) -> Response {
        for mw in &self.middleware {
            match mw(req.clone()) {
                Ok(r) => req = r,
                Err(resp) => return resp,
            }
        }

        for route in &self.routes {
            if route.method == req.method && route.path == req.path {
                return (route.handler)(req);
            }
        }

        Response::new().status(404).text("Not Found")
    }
}

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: Option<String>,
    pub min_version: TlsVersion,
    pub max_version: TlsVersion,
    pub client_auth: ClientAuth,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TlsVersion {
    TLS10,
    TLS11,
    TLS12,
    TLS13,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClientAuth {
    None,
    Optional,
    Required,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: usize,
    window: u64,
    burst: usize,
    strategy: RateLimitStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitStrategy {
    Fixed,
    SlidingWindow,
    TokenBucket,
}

pub struct OslServer {
    config: ServerConfig,
    http_config: HttpConfig,
    router: Router,
    tls_config: Option<TlsConfig>,
}

impl OslServer {
    pub fn new(name: &str) -> Self {
        OslServer {
            config: ServerConfig {
                name: name.to_string(),
                ..Default::default()
            },
            http_config: HttpConfig::default(),
            router: Router::new(),
            tls_config: None,
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.config.host = host.to_string();
        self
    }

    pub fn workers(mut self, workers: usize) -> Self {
        self.config.workers = workers;
        self
    }

    pub fn route_get(mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.router.get(path, handler);
        self
    }

    pub fn route_post(mut self, path: &str, handler: impl Fn(Request) -> Response + Send + Sync + 'static) -> Self {
        self.router.post(path, handler);
        self
    }

    pub fn start(self) -> Result<(), std::io::Error> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        println!("Starting OSL server on {}", addr);
        Ok(())
    }
}