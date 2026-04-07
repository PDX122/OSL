use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Keyword(Keyword),
    Symbol(Symbol),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Let, Fn, If, Else, Loop, While, For, In, Return, Break, Continue,
    True, False, Null, Try, Catch, Finally, Throw, On, Panic,
    Server, Socket, Http, Route, Middleware, Apply, To, All,
    Tls, Auth, Security, RateLimit, Process, Thread, Spawn,
    Upstream, Proxy, Db, Cache, Log, Health, Metrics, Monitor,
    Static, Websocket, Env, Config,
    Respond, Request, Response, RequestBody, Params,
    Import, As, From, Map, List,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Dot, Colon, Semicolon, Arrow, FatArrow,
    Plus, Minus, Star, Slash, Percent, Caret,
    Eq, Ne, Lt, Le, Gt, Ge, And, Or, Not,
    Pipe, Question, At,
    Assign, PlusAssign, MinusAssign, MulAssign, DivAssign,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Int(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::Bool(b) => write!(f, "{}", b),
            Token::Keyword(k) => write!(f, "{}", keyword_str(k)),
            Token::Symbol(s) => write!(f, "{}", symbol_str(s)),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}

fn keyword_str(k: &Keyword) -> &'static str {
    match k {
        Keyword::Let => "let", Keyword::Fn => "fn", Keyword::If => "if",
        Keyword::Else => "else", Keyword::Loop => "loop", Keyword::While => "while",
        Keyword::For => "for", Keyword::In => "in", Keyword::Return => "return",
        Keyword::Break => "break", Keyword::Continue => "continue", Keyword::True => "true",
        Keyword::False => "false", Keyword::Null => "null", Keyword::Try => "try",
        Keyword::Catch => "catch", Keyword::Finally => "finally", Keyword::Throw => "throw",
        Keyword::On => "on", Keyword::Panic => "panic", Keyword::Server => "server",
        Keyword::Socket => "socket", Keyword::Http => "http", Keyword::Route => "route",
        Keyword::Middleware => "middleware", Keyword::Apply => "apply", Keyword::To => "to",
        Keyword::All => "all", Keyword::Tls => "tls", Keyword::Auth => "auth",
        Keyword::Security => "security", Keyword::RateLimit => "rate_limit",
        Keyword::Process => "process", Keyword::Thread => "thread", Keyword::Spawn => "spawn",
        Keyword::Upstream => "upstream", Keyword::Proxy => "proxy", Keyword::Db => "db",
        Keyword::Cache => "cache", Keyword::Log => "log", Keyword::Health => "health",
        Keyword::Metrics => "metrics", Keyword::Monitor => "monitor",
        Keyword::Static => "static", Keyword::Websocket => "websocket",
        Keyword::Env => "env", Keyword::Config => "config",
        Keyword::Respond => "respond", Keyword::Request => "request",
        Keyword::Response => "response", Keyword::RequestBody => "body",
        Keyword::Params => "params",
        Keyword::Import => "import", Keyword::As => "as", Keyword::From => "from",
        Keyword::Map => "map", Keyword::List => "list",
    }
}

impl Keyword {
    pub fn to_string(&self) -> String {
        keyword_str(self).to_string()
    }
}

fn symbol_str(s: &Symbol) -> &'static str {
    match s {
        Symbol::LParen => "(", Symbol::RParen => ")", Symbol::LBrace => "{",
        Symbol::RBrace => "}", Symbol::LBracket => "[", Symbol::RBracket => "]",
        Symbol::Comma => ",", Symbol::Dot => ".", Symbol::Colon => ":",
        Symbol::Semicolon => ";", Symbol::Arrow => "->", Symbol::FatArrow => "=>",
        Symbol::Plus => "+", Symbol::Minus => "-", Symbol::Star => "*",
        Symbol::Slash => "/", Symbol::Percent => "%", Symbol::Caret => "^",
        Symbol::Eq => "==", Symbol::Ne => "!=", Symbol::Lt => "<", Symbol::Le => "<=",
        Symbol::Gt => ">", Symbol::Ge => ">=", Symbol::And => "&&", Symbol::Or => "||",
        Symbol::Not => "!", Symbol::Pipe => "|", Symbol::Question => "?",
        Symbol::At => "@", Symbol::Assign => "=", Symbol::PlusAssign => "+=",
        Symbol::MinusAssign => "-=", Symbol::MulAssign => "*=", Symbol::DivAssign => "/=",
    }
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    start: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            start: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            self.skip_comment();
            if self.is_at_end() {
                tokens.push(Token::Eof);
                break;
            }
            let token = self.next_token();
            tokens.push(token);
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.input[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn skip_comment(&mut self) {
        if self.peek() == Some('/') {
            if self.peek_next() == Some('/') {
                while !self.is_at_end() && self.peek() != Some('\n') {
                    self.advance();
                }
            } else if self.peek_next() == Some('*') {
                self.advance(); self.advance();
                while !self.is_at_end() {
                    if self.peek() == Some('*') && self.peek_next() == Some('/') {
                        self.advance(); self.advance();
                        break;
                    }
                    self.advance();
                }
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> char {
        if self.pos >= self.input.len() {
            return '\0';
        }
        let c = self.input[self.pos];
        self.pos += 1;
        c
    }

    fn next_token(&mut self) -> Token {
        if self.is_at_end() {
            return Token::Eof;
        }
        let c = self.advance();
        if c == '\0' {
            return Token::Eof;
        }
        match c {
            '(' => Token::Symbol(Symbol::LParen),
            ')' => Token::Symbol(Symbol::RParen),
            '{' => Token::Symbol(Symbol::LBrace),
            '}' => Token::Symbol(Symbol::RBrace),
            '[' => Token::Symbol(Symbol::LBracket),
            ']' => Token::Symbol(Symbol::RBracket),
            ',' => Token::Symbol(Symbol::Comma),
            '.' => Token::Symbol(Symbol::Dot),
            ';' => Token::Symbol(Symbol::Semicolon),
            ':' => Token::Symbol(Symbol::Colon),
            '@' => Token::Symbol(Symbol::At),
            '?' => Token::Symbol(Symbol::Question),
            '^' => Token::Symbol(Symbol::Caret),
            '+' => self.match_assign(Symbol::PlusAssign, Symbol::Plus),
            '-' => self.match_assign_or(Symbol::MinusAssign, Symbol::Minus, '-'),
            '*' => self.match_assign(Symbol::MulAssign, Symbol::Star),
            '/' => self.match_assign(Symbol::DivAssign, Symbol::Slash),
            '%' => Token::Symbol(Symbol::Percent),
            '=' => self.match_or(Symbol::FatArrow, Symbol::Assign, '='),
            '!' => self.match_or(Symbol::Ne, Symbol::Not, '='),
            '<' => self.match_assign_or(Symbol::Le, Symbol::Lt, '<'),
            '>' => self.match_assign_or(Symbol::Ge, Symbol::Gt, '>'),
            '&' => self.match_or(Symbol::And, Symbol::And, '&'),
            '|' => self.match_or(Symbol::Or, Symbol::Pipe, '|'),
            '"' => self.string(),
            c if c.is_alphabetic() || c == '_' => self.identifier(c),
            c if c.is_ascii_digit() => self.number(c),
            c if c.is_whitespace() => { self.advance(); return self.next_token(); }
            _ => panic!("Unexpected character: {:?}", c),
        }
    }

    fn match_assign(&mut self, assign: Symbol, base: Symbol) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::Symbol(assign)
        } else {
            Token::Symbol(base)
        }
    }

    fn match_or(&mut self, or: Symbol, base: Symbol, next: char) -> Token {
        if self.peek() == Some(next) {
            self.advance();
            Token::Symbol(or)
        } else {
            Token::Symbol(base)
        }
    }

    fn match_assign_or(&mut self, assign: Symbol, base: Symbol, next: char) -> Token {
        match self.peek() {
            Some('=') => { self.advance(); Token::Symbol(assign) }
            Some(n) if n == next => { self.advance(); Token::Symbol(Symbol::Or) }
            _ => Token::Symbol(base),
        }
    }

    fn string(&mut self) -> Token {
        let mut s = String::new();
        while !self.is_at_end() {
            match self.peek() {
                Some('"') => break,
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => { s.push('\n'); self.advance(); }
                        Some('t') => { s.push('\t'); self.advance(); }
                        Some('r') => { s.push('\r'); self.advance(); }
                        Some('\\') => { s.push('\\'); self.advance(); }
                        Some('"') => { s.push('"'); self.advance(); }
                        Some('\'') => { s.push('\''); self.advance(); }
                        Some(c) => { s.push(c); self.advance(); }
                        None => break,
                    }
                }
                Some(c) => { s.push(c); self.advance(); }
                None => break,
            }
        }
        if self.peek() == Some('"') {
            self.advance();
        }
        Token::String(s)
    }

    fn identifier(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while !self.is_at_end() {
            let c = match self.peek() {
                Some(c) => c,
                None => break,
            };
            if c.is_alphanumeric() || c == '_' || c == '-' {
                s.push(self.advance());
            } else {
                break;
            }
        }
        match s.as_str() {
            "let" => Token::Keyword(Keyword::Let),
            "fn" => Token::Keyword(Keyword::Fn),
            "if" => Token::Keyword(Keyword::If),
            "else" => Token::Keyword(Keyword::Else),
            "loop" => Token::Keyword(Keyword::Loop),
            "while" => Token::Keyword(Keyword::While),
            "for" => Token::Keyword(Keyword::For),
            "in" => Token::Keyword(Keyword::In),
            "return" => Token::Keyword(Keyword::Return),
            "break" => Token::Keyword(Keyword::Break),
            "continue" => Token::Keyword(Keyword::Continue),
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "null" => Token::Keyword(Keyword::Null),
            "try" => Token::Keyword(Keyword::Try),
            "catch" => Token::Keyword(Keyword::Catch),
            "finally" => Token::Keyword(Keyword::Finally),
            "throw" => Token::Keyword(Keyword::Throw),
            "on" => Token::Keyword(Keyword::On),
            "panic" => Token::Keyword(Keyword::Panic),
            "server" => Token::Keyword(Keyword::Server),
            "socket" => Token::Keyword(Keyword::Socket),
            "http" => Token::Keyword(Keyword::Http),
            "route" => Token::Keyword(Keyword::Route),
            "middleware" => Token::Keyword(Keyword::Middleware),
            "apply" => Token::Keyword(Keyword::Apply),
            "to" => Token::Keyword(Keyword::To),
            "all" => Token::Keyword(Keyword::All),
            "tls" => Token::Keyword(Keyword::Tls),
            "auth" => Token::Keyword(Keyword::Auth),
            "security" => Token::Keyword(Keyword::Security),
            "rate_limit" => Token::Keyword(Keyword::RateLimit),
            "process" => Token::Keyword(Keyword::Process),
            "thread" => Token::Keyword(Keyword::Thread),
            "spawn" => Token::Keyword(Keyword::Spawn),
            "upstream" => Token::Keyword(Keyword::Upstream),
            "proxy" => Token::Keyword(Keyword::Proxy),
            "db" => Token::Keyword(Keyword::Db),
            "cache" => Token::Keyword(Keyword::Cache),
            "log" => Token::Keyword(Keyword::Log),
            "health" => Token::Keyword(Keyword::Health),
            "metrics" => Token::Keyword(Keyword::Metrics),
            "monitor" => Token::Keyword(Keyword::Monitor),
            "static" => Token::Keyword(Keyword::Static),
            "websocket" => Token::Keyword(Keyword::Websocket),
            "env" => Token::Keyword(Keyword::Env),
            "config" => Token::Keyword(Keyword::Config),
            "respond" => Token::Keyword(Keyword::Respond),
            "request" => Token::Keyword(Keyword::Request),
            "response" => Token::Keyword(Keyword::Response),
            "body" => Token::Keyword(Keyword::RequestBody),
            "params" => Token::Keyword(Keyword::Params),
            "import" => Token::Keyword(Keyword::Import),
            "as" => Token::Keyword(Keyword::As),
            "from" => Token::Keyword(Keyword::From),
            _ => Token::Ident(s),
        }
    }

    fn number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;
        while !self.is_at_end() {
            match self.peek() {
                Some(c) if c.is_ascii_digit() => s.push(self.advance()),
                Some('.') => {
                    if is_float { break; }
                    is_float = true;
                    s.push(self.advance());
                }
                _ => break,
            }
        }
        if is_float {
            Token::Float(s.parse().unwrap_or(0.0))
        } else {
            Token::Int(s.parse().unwrap_or(0))
        }
    }
}