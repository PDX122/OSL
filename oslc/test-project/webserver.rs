use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::time::Duration;
use std::io::{Read, Write};
use std::thread;

fn main() {
    let addr = "127.0.0.1:8080";
    println!("Starting OSL Web Server on http://{}", addr);
    println!("Available routes:");
    println!("  GET /          - Home page");
    println!("  GET /about     - About page");
    println!("  GET /contact   - Contact page");
    println!("  GET /api       - API info");
    println!("  GET /status    - JSON status");
    println!("");
    
    let listener = std::net::TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).unwrap();
    
    let mut request_count = 0;
    
    loop {
        match listener.accept() {
            Ok((mut stream, client_addr)) => {
                request_count += 1;
                println!("[{}] Request from {}", request_count, client_addr);
                
                let html = get_html_response("/");
                let response = format_http_response(&html);
                
                if let Err(e) = stream.write_all(response.as_bytes()) {
                    eprintln!("Error writing response: {}", e);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
            }
        }
    }
}

fn get_html_response(path: &str) -> String {
    match path {
        "/" => r#"
<!DOCTYPE html>
<html>
<head>
    <title>OSL Web Server</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            color: #fff;
        }
        .container {
            background: rgba(255,255,255,0.15);
            backdrop-filter: blur(10px);
            padding: 60px 80px;
            border-radius: 30px;
            text-align: center;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        h1 {
            font-size: 3.5em;
            margin-bottom: 20px;
            text-shadow: 2px 2px 10px rgba(0,0,0,0.3);
        }
        .tagline {
            font-size: 1.4em;
            opacity: 0.9;
            margin-bottom: 40px;
        }
        .nav {
            display: flex;
            gap: 20px;
            justify-content: center;
            flex-wrap: wrap;
        }
        .btn {
            display: inline-block;
            padding: 18px 36px;
            background: #fff;
            color: #667eea;
            text-decoration: none;
            border-radius: 50px;
            font-weight: bold;
            font-size: 1.1em;
            transition: all 0.3s ease;
            box-shadow: 0 5px 20px rgba(0,0,0,0.2);
        }
        .btn:hover {
            transform: translateY(-3px);
            box-shadow: 0 10px 30px rgba(0,0,0,0.3);
        }
        .btn-secondary {
            background: rgba(255,255,255,0.2);
            color: #fff;
            border: 2px solid #fff;
        }
        .btn-secondary:hover {
            background: #fff;
            color: #667eea;
        }
        .status {
            margin-top: 40px;
            padding: 20px;
            background: rgba(0,255,0,0.2);
            border-radius: 15px;
            font-size: 1.2em;
        }
        .logo {
            font-size: 5em;
            margin-bottom: 20px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="logo">🌐</div>
        <h1>OSL Web Server</h1>
        <p class="tagline">Powered by Open Server Language</p>
        <div class="nav">
            <a href="/about" class="btn">About</a>
            <a href="/contact" class="btn">Contact</a>
            <a href="/api" class="btn">API</a>
            <a href="/status" class="btn btn-secondary">Status</a>
        </div>
        <div class="status">✓ Server Running</div>
    </div>
</body>
</html>
"#.to_string(),
        
        "/about" => r#"
<!DOCTYPE html>
<html>
<head>
    <title>About - OSL Web Server</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            color: #fff;
            padding: 20px;
        }
        .container {
            background: rgba(255,255,255,0.15);
            backdrop-filter: blur(10px);
            padding: 50px;
            border-radius: 30px;
            max-width: 700px;
            text-align: center;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        h1 { font-size: 2.5em; margin-bottom: 30px; }
        p { font-size: 1.2em; line-height: 1.8; margin-bottom: 20px; }
        .features {
            text-align: left;
            background: rgba(0,0,0,0.2);
            padding: 30px;
            border-radius: 20px;
            margin: 30px 0;
        }
        .features li { margin: 10px 0; font-size: 1.1em; }
        a { color: #fff; font-size: 1.2em; }
    </style>
</head>
<body>
    <div class="container">
        <h1>About OSL</h1>
        <p>Open Server Language (OSL) is a modern programming language designed specifically for building high-performance servers.</p>
        <div class="features">
            <ul>
                <li>✓ Compiled language (built with Rust)</li>
                <li>✓ Statically typed</li>
                <li>✓ Built-in HTTP server</li>
                <li>✓ Route definitions</li>
                <li>✓ Middleware support</li>
                <li>✓ Database connections</li>
                <li>✓ Caching layer</li>
                <li>✓ WebSocket support</li>
            </ul>
        </div>
        <p>Version: 1.0.0 | License: MIT</p>
        <a href="/">← Back to Home</a>
    </div>
</body>
</html>
"#.to_string(),
        
        "/contact" => r#"
<!DOCTYPE html>
<html>
<head>
    <title>Contact - OSL Web Server</title>
    <style>
        body {
            font-family: 'Segoe UI', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
            color: #fff;
        }
        .container {
            background: rgba(255,255,255,0.15);
            backdrop-filter: blur(10px);
            padding: 50px;
            border-radius: 30px;
            text-align: center;
        }
        h1 { font-size: 2.5em; margin-bottom: 30px; }
        .contact-info {
            background: rgba(0,0,0,0.2);
            padding: 30px;
            border-radius: 20px;
            margin: 20px 0;
        }
        p { font-size: 1.2em; margin: 15px 0; }
        a { color: #fff; font-size: 1.2em; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Contact Us</h1>
        <div class="contact-info">
            <p><strong>Email:</strong> info@osl.dev</p>
            <p><strong>GitHub:</strong> github.com/osl-lang</p>
            <p><strong>License:</strong> MIT</p>
        </div>
        <a href="/">← Back to Home</a>
    </div>
</body>
</html>
"#.to_string(),
        
        "/api" => r#"
<!DOCTYPE html>
<html>
<head>
    <title>API - OSL Web Server</title>
    <style>
        body {
            font-family: 'Fira Code', monospace;
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 40px;
            min-height: 100vh;
        }
        h1 { color: #569cd6; font-size: 2em; margin-bottom: 30px; }
        .endpoint {
            background: #2d2d2d;
            padding: 20px;
            margin: 15px 0;
            border-radius: 10px;
            border-left: 4px solid #4ec9b0;
        }
        .method {
            color: #4ec9b0;
            font-weight: bold;
        }
        .path {
            color: #ce9178;
        }
        .desc {
            color: #6a9955;
        }
        a { color: #569cd6; }
    </style>
</head>
<body>
    <h1>API Endpoints</h1>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/</span>
        <br><span class="desc">→ Home page (HTML)</span>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/about</span>
        <br><span class="desc">→ About page (HTML)</span>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/contact</span>
        <br><span class="desc">→ Contact page (HTML)</span>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/api</span>
        <br><span class="desc">→ This API documentation (HTML)</span>
    </div>
    <div class="endpoint">
        <span class="method">GET</span> <span class="path">/status</span>
        <br><span class="desc">→ Server status (JSON)</span>
    </div>
    <br>
    <a href="/">← Back to Home</a>
</body>
</html>
"#.to_string(),
        
        "/status" => r#"{"status":"running","server":"OSL Web Server","version":"1.0.0","uptime":0,"timestamp":2024}"#.to_string(),
        
        _ => r#"{"error":"Not Found"}"#.to_string(),
    }
}

fn format_http_response(body: &str) -> String {
    let content_type = if body.starts_with("{") {
        "application/json"
    } else {
        "text/html"
    };
    
    format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: {}\r\n\
        Content-Length: {}\r\n\
        Connection: keep-alive\r\n\
        Server: OSL/1.0\r\n\
        \r\n\
        {}",
        content_type,
        body.len(),
        body
    )
}