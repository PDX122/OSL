use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Expr {
    Ident(String),
    Literal(Literal),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    Lambda(Vec<(String, Type)>, Box<Type>, Box<Stmt>),
    List(Vec<Expr>),
    Map(HashMap<String, Expr>),
    Ternary { cond: Box<Expr>, then: Box<Expr>, else_: Box<Expr> },
    NullCoalesce(Box<Expr>, Box<Expr>),
    Assign(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod, Pow,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
    Concat,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg, Not, BitNot,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    VarDecl { name: String, ty: Type, value: Option<Expr> },
    Assign { target: Expr, value: Expr },
    Block(Vec<Stmt>),
    If { cond: Expr, then: Box<Stmt>, else_: Option<Box<Stmt>> },
    Loop { times: Option<Expr>, body: Box<Stmt> },
    While { cond: Expr, body: Box<Stmt> },
    For { var: String, iter: Expr, body: Box<Stmt> },
    Return(Option<Expr>),
    Break,
    Continue,
    Function { name: String, params: Vec<(String, Type)>, ret: Type, body: Box<Stmt> },
    ServerDecl { name: String, config: HashMap<String, Expr> },
    SocketDecl { socktype: String, config: HashMap<String, Expr> },
    HttpConfig(HashMap<String, Expr>),
    Route { method: String, path: String, body: Box<Stmt> },
    Middleware { name: String, config: HashMap<String, Expr>, before: Option<Box<Stmt>>, after: Option<Box<Stmt>> },
    Apply { middleware: String, paths: Vec<String> },
    TlsConfig(HashMap<String, Expr>),
    Auth { name: String, kind: String, config: HashMap<String, Expr> },
    Security(HashMap<String, Expr>),
    RateLimit { name: String, config: HashMap<String, Expr> },
    ProcessConfig(HashMap<String, Expr>),
    ThreadPool { name: String, config: HashMap<String, Expr> },
    Spawn { kind: String, config: HashMap<String, Expr>, body: Box<Stmt> },
    Upstream { name: String, config: HashMap<String, Expr>, servers: Vec<(String, Expr)> },
    Proxy { path: String, target: String, config: HashMap<String, Expr> },
    Db { name: String, kind: String, config: HashMap<String, Expr> },
    Cache { name: String, kind: String, config: HashMap<String, Expr> },
    LogConfig(HashMap<String, Expr>),
    Health(HashMap<String, Expr>),
    Metrics { kind: String, config: HashMap<String, Expr> },
    Monitor(HashMap<String, Expr>),
    Static { path: String, root: String, config: HashMap<String, Expr> },
    WebSocket { path: String, config: HashMap<String, Expr>, handlers: HashMap<String, Box<Stmt>> },
    EnvConfig(HashMap<String, Expr>),
    ConfigFile(HashMap<String, Expr>),
    TryCatch { try_block: Box<Stmt>, catches: Vec<(String, String, Box<Stmt>)>, finally: Option<Box<Stmt>> },
    OnPanic { body: Box<Stmt> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int, Float, Bool, Str, Byte,
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Func(Vec<Type>, Box<Type>),
    Void,
    Custom(String),
    Infer,
}

impl Type {
    pub fn to_string(&self) -> String {
        match self {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "str".to_string(),
            Type::Byte => "byte".to_string(),
            Type::List(t) => format!("list<{}>", t.to_string()),
            Type::Map(k, v) => format!("map<{}, {}>", k.to_string(), v.to_string()),
            Type::Func(args, ret) => format!("fn({}) -> {}", 
                args.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
                ret.to_string()),
            Type::Void => "void".to_string(),
            Type::Custom(s) => s.clone(),
            Type::Infer => "infer".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub exports: HashMap<String, Type>,
    pub imports: Vec<Import>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
}

impl Program {
    pub fn new() -> Self {
        Program { statements: Vec::new(), exports: HashMap::new(), imports: Vec::new() }
    }
}