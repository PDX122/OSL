use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::ast::*;

pub struct CodeGen {
    output: String,
    indent: usize,
    functions: HashMap<String, String>,
    structs: Vec<String>,
    imports: Vec<String>,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            output: String::new(),
            indent: 0,
            functions: HashMap::new(),
            structs: Vec::new(),
            imports: vec![
                "use std::collections::HashMap;".to_string(),
                "use std::sync::{Arc, Mutex};".to_string(),
                "use std::net::{TcpListener, TcpStream, SocketAddr};".to_string(),
                "use std::io::{Read, Write, BufReader, BufWriter};".to_string(),
                "use std::thread;".to_string(),
                "use std::time::Duration;".to_string(),
            ],
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.line("//! OSL Compiled Server");
        self.line("");
        self.add_import("use std::collections::HashMap;");
        self.add_import("use std::sync::{Arc, Mutex};");
        self.add_import("use std::net::{TcpListener, TcpStream, SocketAddr};");
        self.add_import("use std::io::{Read, Write, BufReader, BufWriter};");
        self.add_import("use std::thread;");
        self.add_import("use std::time::Duration;");
        self.line("");
        
        for import in &program.imports {
            self.generate_import(import);
        }
        
        self.line("fn main() {");
        self.indent();
        
        for stmt in &program.statements {
            self.gen_stmt(stmt);
        }
        
        self.dedent();
        self.line("}");
        
        self.imports.join("\n") + "\n\n" + &self.output
    }

    fn generate_import(&mut self, import: &Import) {
        let path = &import.path;
        let module = if path.starts_with("std/") {
            let pkg = path.trim_start_matches("std/");
            format!("crate::runtime::{}", pkg)
        } else if path.starts_with("osl/") {
            let pkg = path.trim_start_matches("osl/");
            format!("crate::runtime::{}", pkg)
        } else if path.starts_with("community/") {
            let pkg = path.trim_start_matches("community/");
            format!("crate::runtime::community::{}", pkg)
        } else {
            path.to_string()
        };
        
        self.add_import(&format!("use {};", module));
        self.line(&format!("// Imported: {}", path));
    }

    fn add_import(&mut self, import: &str) {
        if !self.imports.contains(&import.to_string()) {
            self.imports.push(import.to_string());
        }
    }

    fn line<S: AsRef<str>>(&mut self, s: S) {
        self.output.push_str(&"    ".repeat(self.indent));
        self.output.push_str(s.as_ref());
        self.output.push('\n');
    }

    fn indent(&mut self) { self.indent += 1; }
    fn dedent(&mut self) { self.indent = self.indent.saturating_sub(1); }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, ty, value } => {
                let rust_ty = self.rust_type(ty);
                if let Some(v) = value {
                    let val = self.gen_expr(v);
                    self.line(format!("let {}: {} = {};", name, rust_ty, val));
                } else {
                    self.line(format!("let {}: {};", name, rust_ty));
                }
            }
            
            Stmt::Function { name, params, ret, body } => {
                let params_rust: Vec<String> = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.rust_type(t)))
                    .collect();
                let ret_rust = self.rust_type(ret);
                
                self.line(format!("fn {}({}) -> {} {{", name, params_rust.join(", "), ret_rust));
                self.indent();
                self.gen_block(body.as_ref());
                self.dedent();
                self.line("}");
                self.line("");
            }
            
            Stmt::Expr(e) => {
                let _ = self.gen_expr(e);
                self.line(";");
            }
            
            Stmt::Return(val) => {
                if let Some(v) = val {
                    let gen = self.gen_expr(v);
                    self.line(format!("return {};", gen));
                } else {
                    self.line("return;");
                }
            }
            
            Stmt::If { cond, then, else_ } => {
                let cond_rust = self.gen_expr(cond);
                self.line(format!("if {} {{", cond_rust));
                self.indent();
                self.gen_stmt(then.as_ref());
                self.dedent();
                
                if let Some(else_stmt) = else_ {
                    self.line("} else {");
                    self.indent();
                    self.gen_stmt(else_stmt.as_ref());
                    self.dedent();
                }
                self.line("}");
            }
            
            Stmt::Loop { times, body } => {
                if let Some(t) = times {
                    let n = self.gen_expr(t);
                    self.line(format!("for _ in 0..{} {{", n));
                } else {
                    self.line("loop {");
                }
                self.indent();
                self.gen_stmt(body.as_ref());
                self.dedent();
                self.line("}");
            }
            
            Stmt::While { cond, body } => {
                let cond_rust = self.gen_expr(cond);
                self.line(format!("while {} {{", cond_rust));
                self.indent();
                self.gen_stmt(body.as_ref());
                self.dedent();
                self.line("}");
            }
            
            Stmt::For { var, iter, body } => {
                let iter_rust = self.gen_expr(iter);
                self.line(format!("for {} in {} {{", var, iter_rust));
                self.indent();
                self.gen_stmt(body.as_ref());
                self.dedent();
                self.line("}");
            }
            
            Stmt::Block(stmts) => {
                self.gen_block_list(stmts);
            }
            
            Stmt::Break => self.line("break;"),
            Stmt::Continue => self.line("continue;"),
            
            Stmt::ServerDecl { name, config } => {
                self.line(format!("// Server: {}", name));
                self.add_import("use actix_web::{App, HttpServer, web};");
                self.gen_server_config(config);
            }
            
            Stmt::SocketDecl { socktype, config } => {
                self.line(format!("// Socket: {}", socktype));
                self.gen_socket_config(socktype, config);
            }
            
            Stmt::HttpConfig(config) => {
                self.line("// HTTP Configuration");
                self.gen_http_config(config);
            }
            
            Stmt::Route { method, path, body } => {
                self.line(format!("// Route: {} {}", method, path));
                self.gen_route(method, path, body.as_ref());
            }
            
            Stmt::Middleware { name, config, before, after } => {
                self.line(format!("// Middleware: {}", name));
                if let Some(b) = before {
                    self.line("// Before middleware");
                }
                if let Some(a) = after {
                    self.line("// After middleware");
                }
            }
            
            Stmt::Apply { middleware, paths } => {
                self.line(format!("// Apply {} to {:?}", middleware, paths));
            }
            
            Stmt::TlsConfig(config) => {
                self.line("// TLS Configuration");
                self.gen_tls_config(config);
            }
            
            Stmt::Auth { name, kind, config } => {
                self.line(format!("// Auth: {} ({})", name, kind));
            }
            
            Stmt::Security(config) => {
                self.line("// Security Configuration");
            }
            
            Stmt::RateLimit { name, config } => {
                self.line(format!("// Rate Limit: {}", name));
            }
            
            Stmt::ProcessConfig(config) => {
                self.line("// Process Configuration");
            }
            
            Stmt::ThreadPool { name, config } => {
                self.line(format!("// Thread Pool: {}", name));
            }
            
            Stmt::Spawn { kind, config, body } => {
                self.line(format!("// Spawn: {}", kind));
            }
            
            Stmt::Upstream { name, config, servers } => {
                self.line(format!("// Upstream: {}", name));
            }
            
            Stmt::Proxy { path, target, config } => {
                self.line(format!("// Proxy: {} -> {}", path, target));
            }
            
            Stmt::Db { name, kind, config } => {
                self.line(format!("// Database: {} ({})", name, kind));
                self.add_import("use sqlx::{PgPool, MySqlPool};");
            }
            
            Stmt::Cache { name, kind, config } => {
                self.line(format!("// Cache: {} ({})", name, kind));
                self.add_import("use redis::Client;");
            }
            
            Stmt::LogConfig(config) => {
                self.line("// Log Configuration");
            }
            
            Stmt::Health(config) => {
                self.line("// Health Check Configuration");
            }
            
            Stmt::Metrics { kind, config } => {
                self.line(format!("// Metrics: {}", kind));
            }
            
            Stmt::Monitor(config) => {
                self.line("// Monitor Configuration");
            }
            
            Stmt::Static { path, root, config } => {
                self.line(format!("// Static: {} -> {}", path, root));
            }
            
            Stmt::WebSocket { path, config, handlers } => {
                self.line(format!("// WebSocket: {}", path));
            }
            
            Stmt::EnvConfig(config) => {
                self.line("// Environment Configuration");
            }
            
            Stmt::ConfigFile(config) => {
                self.line("// Config File");
            }
            
            Stmt::TryCatch { try_block, catches, finally } => {
                self.line("// Try-Catch");
                self.gen_stmt(try_block.as_ref());
            }
            
            Stmt::OnPanic { body } => {
                self.line("// On Panic Handler");
            }
            
            Stmt::Assign { target, value } => {
                let target_rust = self.gen_expr(target);
                let value_rust = self.gen_expr(value);
                self.line(format!("{} = {};", target_rust, value_rust));
            }
        }
    }

    fn gen_block(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => self.gen_block_list(stmts),
            _ => self.gen_stmt(stmt),
        }
    }

    fn gen_block_list(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.gen_stmt(stmt);
        }
    }

    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(s) => s.clone(),
            
            Expr::Literal(l) => match l {
                Literal::Int(n) => n.to_string(),
                Literal::Float(f) => f.to_string(),
                Literal::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
                Literal::Bool(true) => "true".to_string(),
                Literal::Bool(false) => "false".to_string(),
                Literal::Null => "None".to_string(),
            },
            
            Expr::Binary(left, op, right) => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                match op {
                    BinOp::Add => format!("({} + {})", l, r),
                    BinOp::Sub => format!("({} - {})", l, r),
                    BinOp::Mul => format!("({} * {})", l, r),
                    BinOp::Div => format!("({} / {})", l, r),
                    BinOp::Mod => format!("({} % {})", l, r),
                    BinOp::Pow => format!("({}.pow({}))", l, r),
                    BinOp::Eq => format!("({} == {})", l, r),
                    BinOp::Ne => format!("({} != {})", l, r),
                    BinOp::Lt => format!("({} < {})", l, r),
                    BinOp::Le => format!("({} <= {})", l, r),
                    BinOp::Gt => format!("({} > {})", l, r),
                    BinOp::Ge => format!("({} >= {})", l, r),
                    BinOp::And => format!("({} && {})", l, r),
                    BinOp::Or => format!("({} || {})", l, r),
                    BinOp::Concat => format!("format!(\"{}{}\", {}, {})", l, r, l, r),
                    _ => format!("({} {:?} {})", l, op, r),
                }
            }
            
            Expr::Unary(op, expr) => {
                let e = self.gen_expr(expr);
                match op {
                    UnaryOp::Neg => format!("-{}", e),
                    UnaryOp::Not => format!("!{}", e),
                    UnaryOp::BitNot => format!("!{}", e),
                }
            }
            
            Expr::Call(func, args) => {
                let f = self.gen_expr(func);
                let a: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                format!("{}({})", f, a.join(", "))
            }
            
            Expr::Index(obj, index) => {
                let o = self.gen_expr(obj);
                let i = self.gen_expr(index);
                format!("{}[{}]", o, i)
            }
            
            Expr::Field(obj, field) => {
                let o = self.gen_expr(obj);
                format!("{}.{}", o, field)
            }
            
            Expr::Lambda(params, ret, body) => {
                let p: Vec<String> = params.iter()
                    .map(|(n, t)| format!("{}: {}", n, self.rust_type(t)))
                    .collect();
                format!("|{}| {{ /* body */ }}", p.join(", "))
            }
            
            Expr::List(items) => {
                let items_str: Vec<String> = items.iter().map(|i| self.gen_expr(i)).collect();
                format!("vec![{}]", items_str.join(", "))
            }
            
            Expr::Map(fields) => {
                let fields_str: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("\"{}\" => {}", k, self.gen_expr(v)))
                    .collect();
                format!("HashMap::from([{}])", fields_str.join(", "))
            }
            
            Expr::Ternary { cond, then, else_ } => {
                let c = self.gen_expr(cond);
                let t = self.gen_expr(then);
                let e = self.gen_expr(else_);
                format!("if {} {{ {} }} else {{ {} }}", c, t, e)
            }
            
            Expr::NullCoalesce(left, right) => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                format!("{}.or_else(|| {})", l, r)
            }
            
            Expr::Assign(_, _) => "/* assignment */".to_string(),
            
            Expr::In(_, _) => "true".to_string(),
        }
    }

    fn rust_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "i64".to_string(),
            Type::Float => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "String".to_string(),
            Type::Byte => "u8".to_string(),
            Type::Void => "()".to_string(),
            Type::List(t) => format!("Vec<{}>", self.rust_type(t)),
            Type::Map(k, v) => format!("HashMap<{}, {}>", self.rust_type(k), self.rust_type(v)),
            Type::Func(args, ret) => {
                let args_str: Vec<String> = args.iter().map(|a| self.rust_type(a)).collect();
                format!("Box<dyn Fn({}) -> {}>", args_str.join(", "), self.rust_type(ret))
            }
            Type::Custom(s) => s.clone(),
            Type::Infer => "let".to_string(),
        }
    }

    fn gen_server_config(&mut self, config: &HashMap<String, Expr>) {
        if let Some(expr) = config.get("port") {
            let gen = self.gen_expr(expr);
            self.line(format!("let port = {};", gen));
        }
        if let Some(expr) = config.get("host") {
            let gen = self.gen_expr(expr);
            self.line(format!("let host = {};", gen));
        }
        if let Some(expr) = config.get("workers") {
            let gen = self.gen_expr(expr);
            self.line(format!("let workers = {};", gen));
        }
    }

    fn gen_socket_config(&mut self, socktype: &str, config: &HashMap<String, Expr>) {
        if let Some(expr) = config.get("bind") {
            let gen = self.gen_expr(expr);
            self.line(format!("let bind_addr = {};", gen));
        }
    }

    fn gen_http_config(&mut self, _config: &HashMap<String, Expr>) {
        self.line("// HTTP config handled at runtime");
    }

    fn gen_route(&mut self, method: &str, path: &str, body: &Stmt) {
        self.line(format!("// {} {}", method, path));
        match body {
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    match stmt {
                        Stmt::Return(Some(expr)) => {
                            let result = self.gen_expr(expr);
                            self.line(format!("Ok({})", result));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn gen_tls_config(&mut self, _config: &HashMap<String, Expr>) {
        self.line("// TLS config handled at runtime");
    }
}

pub fn generate_rust(program: &Program) -> String {
    let mut gen = CodeGen::new();
    gen.generate(program)
}

pub fn write_output(code: &str, output_path: &str) -> Result<(), std::io::Error> {
    fs::write(output_path, code)
}