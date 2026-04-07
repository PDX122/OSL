# OSL — Open Server Language

A compiled, statically-typed programming language built with Rust, engineered for writing, managing, and controlling high-performance servers. OSL files use the `.osl` extension.

## Language Properties

| Property       | Value                          |
|----------------|--------------------------------|
| Name           | Open Server Language (OSL)     |
| File Extension | `.osl`                         |
| Built With     | Rust                           |
| Paradigm       | Imperative, Structured         |
| Typing         | Static, Strong                |
| Primary Use    | Server programming & control   |

## Type System

### Primitive Types

- `str` — UTF-8 string
- `int` — Signed 64-bit integer
- `uint` — Unsigned 64-bit integer
- `float` — 64-bit floating point
- `bool` — Boolean (`true` / `false`)
- `byte` — Single byte (0-255)

### Collection Types

- `list<T>` — Generic list/array, e.g., `list<str>`, `list<int>`
- `map<K, V>` — Key-value map, e.g., `map<str, str>`

### Type Annotations

Variables are declared with explicit type annotations:

```osl
let name: str = "my-server";
let port: int = 8080;
let timeout: float = 30.5;
let debug: bool = true;
let tags: list<str> = ["api", "v2", "prod"];
let config: map<str, str> = { "env": "production" };
```

## Function Declarations

Functions are declared with the `fn` keyword, parameter types, and return type:

```osl
fn greet(name: str) -> str {
    return "Hello, " + name;
}

fn add(a: int, b: int) -> int {
    return a + b;
}

fn nothing() -> void {
    log.info("done");
}
```

Return type `void` indicates no return value.

## Control Flow

### Conditional Execution

```osl
if port == 443 {
    tls enable;
} else if port == 80 {
    redirect http -> https;
} else {
    log.warn("Non-standard port: " + port);
}
```

### Loops

```osl
loop 10 {
    retry connect;
}

while server.alive {
    poll events;
}

for worker in workers {
    worker.ping;
}
```

## Server Declaration

```osl
server "my-api" {
    port 8080;
    host "0.0.0.0";
    workers 16;
    backlog 512;
    timeout 30;
    keep_alive true;
    max_connections 10000;
    graceful_shutdown true;
    shutdown_timeout 15;
    restart_on_crash true;
    pid_file "/var/run/my-api.pid";
}
```

### Server Lifecycle Commands

```osl
server.start;
server.stop;
server.restart;
server.reload;
server.status;
server.pause;
server.resume;
server.drain;
server.kill;
server.fork;
```

## Networking & Sockets

### TCP Socket Configuration

```osl
socket tcp {
    bind "0.0.0.0:9000";
    listen 128;
    reuse_addr true;
    reuse_port true;
    no_delay true;
    recv_buffer 65536;
    send_buffer 65536;
    keepalive true;
    keepalive_idle 60;
    keepalive_interval 10;
    keepalive_count 5;
}
```

### UDP Socket

```osl
socket udp {
    bind "0.0.0.0:9001";
    broadcast true;
}
```

### Unix Domain Socket

```osl
socket unix {
    path "/tmp/my-server.sock";
    permissions 0o660;
}
```

### Raw Socket Operations

```osl
let conn = socket.accept;
conn.read_bytes 4096;
conn.write_bytes payload;
conn.flush;
conn.close;
conn.half_close;
conn.set_timeout 5000;
conn.peer_addr;
conn.local_addr;
```

## HTTP Server Configuration

```osl
http {
    version HTTP2;
    compression gzip;
    max_body_size 10mb;
    max_headers 100;
    header_timeout 10;
    body_timeout 30;
    idle_timeout 60;
    pipeline true;
}
```

Supported HTTP versions: `HTTP1.1`, `HTTP2`, `HTTP3`
Supported compression: `gzip`, `br`, `zstd`, `none`

### Route Definitions

```osl
route GET "/" {
    respond 200 "OK";
}

route POST "/users" {
    let body = request.body.json;
    let id = db.insert("users", body);
    respond 201 { "id": id };
}

route PUT "/users/:id" {
    db.update("users", params.id, request.body.json);
    respond 200 { "updated": true };
}

route DELETE "/users/:id" {
    db.delete("users", params.id);
    respond 204;
}

route PATCH "/users/:id" {
    db.patch("users", params.id, request.body.json);
    respond 200;
}

route WS "/live" {
    ws.on_connect { log.info("client joined"); }
    ws.on_message msg { ws.broadcast msg; }
    ws.on_disconnect { log.info("client left"); }
}
```

Supported methods: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS`, `HEAD`, `WS`

### Request Object Properties

```osl
request.method;
request.path;
request.query;
request.query.get("key");
request.headers;
request.headers.get("Authorization");
request.body.raw;
request.body.json;
request.body.text;
request.body.form;
request.ip;
request.ip.real;
request.user_agent;
request.cookies;
request.cookies.get("session");
request.protocol;
request.port;
request.host;
request.timestamp;
request.id;
```

### Response Object Operations

```osl
response.status 200;
response.header "Content-Type" "application/json";
response.header "X-Request-Id" request.id;
response.cookie "session" token { httponly true; secure true; samesite Strict; max_age 3600; };
response.body { "ok": true };
response.send;

response.redirect "/new-path" 301;
response.stream chunks;
response.file "/var/www/file.pdf";
response.abort 503;
```

## Middleware

### Built-in Middleware

```osl
middleware cors {
    allow_origins ["https://myapp.com", "https://api.myapp.com"];
    allow_methods [GET, POST, PUT, DELETE, OPTIONS];
    allow_headers ["Authorization", "Content-Type"];
    expose_headers ["X-Request-Id"];
    allow_credentials true;
    max_age 86400;
}

middleware body_parser {
    json true;
    urlencoded true;
    multipart true;
    max_size 50mb;
}

middleware request_id {
    header "X-Request-Id";
    generate uuid_v4;
}

middleware compression {
    algorithm gzip;
    min_size 1024;
    level 6;
}

middleware timeout {
    request 30;
    response 60;
    idle 120;
}
```

### Custom Middleware

```osl
middleware my_auth {
    before request {
        let token = request.headers.get("Authorization");
        if token == null {
            respond 401 { "error": "Unauthorized" };
            halt;
        }
        let user = auth.verify token;
        if user == null {
            respond 403 { "error": "Forbidden" };
            halt;
        }
        request.set("user", user);
    }

    after response {
        log.info("Request completed by user: " + request.get("user").id);
    }
}

apply my_auth to ["/users", "/admin"];
apply cors to all;
```

## TLS / HTTPS Configuration

```osl
tls {
    cert "/etc/ssl/certs/server.crt";
    key "/etc/ssl/private/server.key";
    ca "/etc/ssl/certs/ca-bundle.crt";
    min_version TLS12;
    max_version TLS13;
    ciphers ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"];
    hsts true;
    hsts_max_age 31536000;
    ocsp_stapling true;
    client_auth optional;
    session_cache true;
    session_timeout 300;
}
```

TLS versions: `TLS10`, `TLS11`, `TLS12`, `TLS13`
Client auth modes: `none`, `optional`, `required`

## Authentication

### JWT Authentication

```osl
auth jwt {
    secret env("JWT_SECRET");
    algorithm HS256;
    expiry 3600;
    refresh_expiry 604800;
    issuer "my-api";
    audience "my-client";
}
```

Algorithms: `HS256`, `RS256`, `ES256`

### Basic Authentication

```osl
auth basic {
    realm "Admin";
    users {
        "admin": env("ADMIN_PASS");
    }
}
```

### API Key Authentication

```osl
auth api_key {
    header "X-API-Key";
    keys env("API_KEYS").split(",");
}
```

## Security Headers

```osl
security {
    xss_protection true;
    content_type_nosniff true;
    frame_options DENY;
    referrer_policy "no-referrer";
    csp "default-src 'self'; script-src 'self'";
    hide_server_header true;
    ip_blacklist ["/etc/osl/blacklist.txt"];
    ip_whitelist ["10.0.0.0/8", "192.168.0.0/16"];
}
```

Frame options: `DENY`, `SAMEORIGIN`

## Rate Limiting

```osl
rate_limit global {
    requests 1000;
    window 60;
    burst 200;
    strategy sliding_window;
}

rate_limit per_ip {
    requests 100;
    window 60;
    burst 20;
    key request.ip;
}

rate_limit per_user {
    requests 500;
    window 60;
    key request.get("user").id;
}

rate_limit route POST "/login" {
    requests 5;
    window 60;
    lockout 300;
    respond 429 { "error": "Too many attempts" };
}
```

Strategies: `fixed`, `sliding_window`, `token_bucket`

## Process & Thread Control

### Process Configuration

```osl
process {
    workers 16;
    threads_per_worker 4;
    affinity auto;
    max_memory 2gb;
    stack_size 8mb;
    nice -10;
    user "www-data";
    group "www-data";
    chroot "/var/www";
    daemonize true;
}
```

### Thread Pool

```osl
thread pool {
    min 4;
    max 64;
    idle_timeout 30;
    queue_size 1024;
    overflow reject;
}
```

Overflow modes: `reject`, `block`, `grow`

### Task Spawning

```osl
spawn async task {
    let result = heavy_computation();
    log.info(result);
}

spawn interval 5 {
    health.check;
}

spawn cron "0 0 * * *" {
    db.vacuum;
}
```

## Load Balancing & Proxy

### Upstream Configuration

```osl
upstream backend {
    strategy round_robin;
    health_check "/health" interval 10 timeout 3;
    max_fails 3;
    fail_timeout 30;

    server "10.0.0.1:8080" weight 3;
    server "10.0.0.2:8080" weight 3;
    server "10.0.0.3:8080" weight 1 backup;
}
```

Strategies: `round_robin`, `least_conn`, `ip_hash`, `random`, `weighted`

### Proxy Configuration

```osl
proxy "/api" -> backend {
    timeout 30;
    buffer_size 8192;
    pass_headers ["Authorization", "X-Request-Id"];
    rewrite "/api/(.*)" -> "/$1";
    retry 3;
    retry_on [502, 503, 504];
}
```

## Database

### PostgreSQL Connection

```osl
db postgres {
    host env("DB_HOST");
    port 5432;
    name env("DB_NAME");
    user env("DB_USER");
    pass env("DB_PASS");
    pool_min 5;
    pool_max 50;
    timeout 10;
    ssl true;
    ssl_mode require;
}
```

### MySQL Connection

```osl
db mysql {
    host "localhost";
    port 3306;
    name "mydb";
    user env("MYSQL_USER");
    pass env("MYSQL_PASS");
    pool_max 30;
}
```

### Database Operations

```osl
let users = db.query("SELECT * FROM users WHERE active = true");
let user = db.find("users", id);
let new_id = db.insert("users", { "name": "Alice", "email": "a@b.com" });
db.update("users", id, { "name": "Bob" });
db.delete("users", id);
db.exec("VACUUM");

db.transaction {
    db.insert("orders", order);
    db.update("inventory", item_id, { "stock": stock - 1 });
    db.insert("audit_log", log_entry);
}
```

## Caching

### Redis Cache

```osl
cache redis {
    host env("REDIS_HOST");
    port 6379;
    pass env("REDIS_PASS");
    db 0;
    pool 20;
    timeout 5;
}
```

### In-Memory Cache

```osl
cache memory {
    max_size 512mb;
    eviction lru;
}
```

Eviction policies: `lru`, `lfu`, `ttl`, `random`

### Cache Operations

```osl
cache.set "user:123" user_data ttl 3600;
cache.get "user:123";
cache.delete "user:123";
cache.exists "user:123";
cache.expire "user:123" 1800;
cache.increment "counter:hits";
cache.decrement "counter:credits";
cache.flush;
```

### Route-Level Caching

```osl
route GET "/users" cache ttl 60 {
    let users = db.query("SELECT * FROM users");
    respond 200 users;
}
```

## Logging Configuration

```osl
log {
    level info;
    format json;
    output [stdout, "/var/log/osl/server.log"];
    rotate {
        max_size 100mb;
        max_files 10;
        compress true;
    }
    include_request_id true;
    include_timestamp true;
    include_caller true;
}
```

Log levels: `trace`, `debug`, `info`, `warn`, `error`, `fatal`
Log formats: `json`, `text`, `pretty`

### Logging Statements

```osl
log.trace "very verbose message";
log.debug "debugging info";
log.info "server started on port " + port;
log.warn "high memory usage: " + mem.used;
log.error "connection failed: " + err.message;
log.fatal "unrecoverable error — shutting down";

log.info {
    "event": "user_login",
    "user_id": user.id,
    "ip": request.ip,
    "timestamp": now()
};
```

## Monitoring & Health

### Health Endpoints

```osl
health {
    endpoint "/health";
    endpoint "/readiness";
    endpoint "/liveness";
    include_uptime true;
    include_memory true;
    include_cpu true;
    include_connections true;
}
```

### Prometheus Metrics

```osl
metrics prometheus {
    endpoint "/metrics";
    prefix "osl_";
    labels { "app": "my-server", "env": "prod" };
}

metrics.counter "http_requests_total" labels ["method", "route", "status"];
metrics.gauge "active_connections";
metrics.histogram "request_duration_ms" buckets [10, 50, 100, 250, 500, 1000];
```

### Alerts

```osl
monitor alerts {
    cpu_above 90 for 60 -> alert "high_cpu";
    memory_above 80 for 120 -> alert "high_mem";
    error_rate_above 5 for 30 -> alert "high_errors";
    connections_above 9000 -> alert "near_limit";
}
```

## Environment & Configuration

```osl
env {
    file ".env";
    override true;
}

config {
    file "/etc/osl/config.osl";
    watch true;
    format osl;
}
```

Config formats: `osl`, `toml`, `json`, `yaml`

### Environment Variables

```osl
let secret = env("JWT_SECRET");
let port = env("PORT").to_int ?? 8080;
let debug = env("DEBUG").to_bool ?? false;
```

Type conversion methods: `.to_int()`, `.to_bool()`, `.to_float()`

## Error Handling

```osl
try {
    let data = db.find("users", id);
    respond 200 data;
} catch DbNotFound err {
    respond 404 { "error": "User not found" };
} catch DbError err {
    log.error err.message;
    respond 500 { "error": "Database error" };
} catch err {
    log.fatal err;
    respond 500 { "error": "Internal server error" };
} finally {
    db.release;
}

on panic {
    log.fatal "Server panicked: " + err.message;
    metrics.increment "panics_total";
    server.restart;
}
```

## Static Files

```osl
static "/public" -> "/var/www/public" {
    cache_control "public, max-age=31536000";
    etag true;
    last_modified true;
    index ["index.html"];
    gzip true;
    brotli true;
    dotfiles deny;
}
```

## WebSockets

```osl
websocket "/ws" {
    max_message_size 1mb;
    ping_interval 30;
    ping_timeout 10;
    compression true;

    on_connect conn {
        rooms.join conn "general";
        ws.send conn { "event": "welcome" };
    }

    on_message conn msg {
        let parsed = msg.json;
        rooms.broadcast parsed.room parsed.data;
    }

    on_disconnect conn code {
        log.info "Client disconnected: " + code;
        rooms.leave_all conn;
    }

    on_error conn err {
        log.error err.message;
        conn.close 1011;
    }
}
```

## CLI Commands

### Build & Run

```bash
oslc run server.osl
oslc build server.osl
oslc build server.osl -o my-server
oslc build server.osl --release
oslc build server.osl --debug
oslc run server.osl --env production
oslc run server.osl --watch
oslc run server.osl --port 3000
oslc run server.osl --workers 32
oslc run server.osl --host 127.0.0.1
oslc run server.osl --daemon
oslc run server.osl --pid /var/run/myserver.pid
```

### Syntax & Validation

```bash
oslc check server.osl
oslc check ./src
oslc check server.osl --ast
oslc lint server.osl
oslc lint server.osl --fix
oslc fmt server.osl
oslc fmt server.osl --dry-run
oslc fmt .
oslc typecheck server.osl
```

### Project Management

```bash
oslc init my-project
oslc init . --template api
oslc info
oslc routes server.osl
oslc middleware server.osl
oslc config server.osl --resolve
```

### Server Process Control

```bash
oslc start my-server
oslc stop --pid /var/run/myserver.pid
oslc stop my-server
oslc restart my-server
oslc reload my-server
oslc drain my-server
oslc kill my-server
oslc status
oslc status my-server
```

### Logging & Monitoring

```bash
oslc logs my-server
oslc logs my-server --lines 100
oslc logs my-server --level error
oslc logs my-server --route /api/users
oslc logs my-server --format json
oslc metrics my-server
oslc metrics my-server --snapshot
oslc metrics my-server --out metrics.json
oslc health http://localhost:8080
oslc health http://localhost:8080 --exit-code
```

### TLS & Certificates

```bash
oslc tls gen --host localhost --out ./certs
oslc tls verify --cert server.crt --key server.key
oslc tls info --cert server.crt
oslc tls check --cert server.crt --warn-days 30
```

### Database Utilities

```bash
oslc db ping server.osl
oslc db query server.osl "SELECT COUNT(*) FROM users"
oslc db pool server.osl
```

### Cache Utilities

```bash
oslc cache ping server.osl
oslc cache flush server.osl
oslc cache stats server.osl
oslc cache get server.osl "user:123"
oslc cache delete server.osl "user:123"
```

### Benchmarking & Testing

```bash
oslc bench http://localhost:8080/api/users
oslc bench http://localhost:8080 --concurrency 100 --requests 10000
oslc bench http://localhost:8080 --duration 30s
oslc bench http://localhost:8080/users --method POST --body '{"name":"test"}'
oslc test server.osl
oslc test .
oslc test . --verbose
oslc test . --format json
```

### Misc

```bash
oslc --version
oslc --help
oslc run --help
oslc flags
oslc env --list
```

## File Structure

```
project/
├── main.osl              # Entry point
├── config/
│   ├── production.osl
│   └── development.osl
├── routes/
│   ├── users.osl
│   ├── auth.osl
│   └── admin.osl
└── middleware/
    ├── auth.osl
    └── logging.osl
```

## Comments

Single-line and multi-line comments are supported:

```osl
// Single-line comment

/*
   Multi-line comment
*/
```