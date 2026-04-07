use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn parse(json: &str) -> Result<Self, String> {
        parse_value(&mut json.chars().peekable())
    }

    pub fn to_string(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::String(s) => format!("\"{}\"", escape_string(s)),
            JsonValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            JsonValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\": {}", escape_string(k), v.to_string()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    pub fn index(&self, i: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(i),
            _ => None,
        }
    }
}

fn parse_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    skip_whitespace(chars);
    let ch = chars.peek().ok_or("Unexpected EOF")?;
    
    match ch {
        'n' => parse_null(chars),
        't' => parse_true(chars),
        'f' => parse_false(chars),
        '"' => parse_string(chars),
        '[' => parse_array(chars),
        '{' => parse_object(chars),
        '-' | '0'..='9' => parse_number(chars),
        _ => Err(format!("Unexpected character: {}", ch)),
    }
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn parse_null(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    let s: String = chars.by_ref().take(4).collect();
    if s == "null" { Ok(JsonValue::Null) } else { Err("Invalid null".to_string()) }
}

fn parse_true(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    let s: String = chars.by_ref().take(4).collect();
    if s == "true" { Ok(JsonValue::Bool(true)) } else { Err("Invalid true".to_string()) }
}

fn parse_false(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    let s: String = chars.by_ref().take(5).collect();
    if s == "false" { Ok(JsonValue::Bool(false)) } else { Err("Invalid false".to_string()) }
}

fn parse_string(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    chars.next();
    let mut s = String::new();
    while let Some(&ch) = chars.peek() {
        match ch {
            '"' => { chars.next(); return Ok(JsonValue::String(s)); }
            '\\' => {
                chars.next();
                let escaped = chars.next().ok_or("Unexpected EOF")?;
                match escaped {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    _ => s.push(escaped),
                }
            }
            _ => s.push(ch),
        }
        chars.next();
    }
    Err("Unterminated string".to_string())
}

fn parse_number(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    let mut num_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_numeric() || ch == '.' || ch == '-' || ch == 'e' || ch == 'E' || ch == '+' {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    num_str.parse::<f64>()
        .map(JsonValue::Number)
        .map_err(|_| "Invalid number".to_string())
}

fn parse_array(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    chars.next();
    let mut arr = Vec::new();
    skip_whitespace(chars);
    while let Some(&ch) = chars.peek() {
        if ch == ']' { chars.next(); return Ok(JsonValue::Array(arr)); }
        arr.push(parse_value(chars)?);
        skip_whitespace(chars);
        if let Some(&',') = chars.peek() { chars.next(); }
        skip_whitespace(chars);
    }
    Err("Unterminated array".to_string())
}

fn parse_object(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<JsonValue, String> {
    chars.next();
    let mut obj = HashMap::new();
    skip_whitespace(chars);
    while let Some(&ch) = chars.peek() {
        if ch == '}' { chars.next(); return Ok(JsonValue::Object(obj)); }
        let key = match parse_value(chars)? {
            JsonValue::String(s) => s,
            _ => return Err("Expected string key".to_string()),
        };
        skip_whitespace(chars);
        if let Some(&':') = chars.peek() { chars.next(); }
        let value = parse_value(chars)?;
        obj.insert(key, value);
        skip_whitespace(chars);
        if let Some(&',') = chars.peek() { chars.next(); }
        skip_whitespace(chars);
    }
    Err("Unterminated object".to_string())
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}

pub fn parse(json: &str) -> Result<JsonValue, String> {
    JsonValue::parse(json)
}

pub fn stringify(value: &JsonValue) -> String {
    value.to_string()
}
