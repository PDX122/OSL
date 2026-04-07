use crate::runtime::{Request, Response};
use std::collections::HashMap;

pub fn json_response(data: serde_json::Value) -> Response {
    Response::new().json(data)
}

pub fn text_response(text: &str) -> Response {
    Response::new().text(text)
}

pub fn error_response(code: u16, message: &str) -> Response {
    Response::new()
        .status(code)
        .json(serde_json::json!({ "error": message }))
}

pub fn redirect(path: &str, code: u16) -> Response {
    Response::new()
        .status(code)
        .header("Location", path)
}

pub fn set_cookie(response: &mut Response, name: &str, value: &str, options: CookieOptions) {
    let mut cookie = format!("{}={}", name, value);
    if let Some(max_age) = options.max_age {
        cookie.push_str(&format!("; Max-Age={}", max_age));
    }
    if options.secure {
        cookie.push_str("; Secure");
    }
    if options.http_only {
        cookie.push_str("; HttpOnly");
    }
    if let Some(same_site) = &options.same_site {
        cookie.push_str(&format!("; SameSite={}", same_site));
    }
    response.headers.insert("Set-Cookie".to_string(), cookie);
}

#[derive(Debug, Clone)]
pub struct CookieOptions {
    pub max_age: Option<u64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

impl Default for CookieOptions {
    fn default() -> Self {
        CookieOptions {
            max_age: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }
}

pub struct RequestParser;

impl RequestParser {
    pub fn parse_method(headers: &HashMap<String, String>) -> Option<String> {
        headers.get("method").cloned().or_else(|| {
            headers.get("x-http-method-override").cloned()
        })
    }

    pub fn parse_path(url: &str) -> (String, HashMap<String, String>) {
        let parts: Vec<&str> = url.split('?').collect();
        let path = parts.first().unwrap_or(&"/").to_string();
        
        let mut query = HashMap::new();
        if let Some(qs) = parts.get(1) {
            for pair in qs.split('&') {
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 {
                    query.insert(kv[0].to_string(), kv[1].to_string());
                }
            }
        }
        
        (path, query)
    }

    pub fn parse_headers(headers: &[(&str, &str)]) -> HashMap<String, String> {
        headers.iter()
            .map(|(k, v)| (k.to_lowercase(), v.to_string()))
            .collect()
    }
}

pub fn cors_middleware(options: CorsOptions) -> impl Fn(Request) -> Result<Request, Response> {
    move |mut req| {
        for (key, value) in &options.allow_origins {
            req.headers.insert("Access-Control-Allow-Origin".to_string(), value.clone());
        }
        
        if options.allow_credentials {
            req.headers.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        }
        
        if let Some(max_age) = options.max_age {
            req.headers.insert("Access-Control-Max-Age".to_string(), max_age.to_string());
        }
        
        Ok(req)
    }
}

#[derive(Debug, Clone)]
pub struct CorsOptions {
    pub allow_origins: Vec<(String, String)>,
    pub allow_methods: Vec<String>,
    pub allow_headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u64>,
}

impl Default for CorsOptions {
    fn default() -> Self {
        CorsOptions {
            allow_origins: vec![("*".to_string(), "*".to_string())],
            allow_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allow_headers: vec!["Content-Type".to_string()],
            expose_headers: vec![],
            allow_credentials: false,
            max_age: None,
        }
    }
}