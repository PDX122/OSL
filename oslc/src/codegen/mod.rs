use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::ast::*;
use crate::package_manager;

pub struct CodeGen {
    output: RefCell<String>,
    indent: usize,
    functions: HashMap<String, String>,
    structs: Vec<String>,
    imports: RefCell<Vec<String>>,
    registry_modules: RefCell<Vec<(String, String)>>,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            output: RefCell::new(String::new()),
            indent: 0,
            functions: HashMap::new(),
            structs: Vec::new(),
            imports: RefCell::new(vec![
                "use std::collections::HashMap;".to_string(),
                "use std::sync::{Arc, Mutex};".to_string(),
                "use std::net::{TcpListener, TcpStream, SocketAddr};".to_string(),
                "use std::io::{Read, Write, BufReader, BufWriter};".to_string(),
                "use std::thread;".to_string(),
                "use std::time::Duration;".to_string(),
            ]),
            registry_modules: RefCell::new(Vec::new()),
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.add_import("use std::collections::HashMap;");
        self.add_import("use std::sync::{Arc, Mutex};");
        self.add_import("use std::net::{TcpListener, TcpStream, SocketAddr};");
        self.add_import("use std::io::{Read, Write, BufReader, BufWriter};");
        self.add_import("use std::thread;");
        self.add_import("use std::time::Duration;");

        for import in &program.imports {
            self.generate_import(import);
        }

        let mut result = String::new();

        for (module_name, _path) in self.registry_modules.borrow().iter() {
            result.push_str(&format!("mod {} {{\n", module_name));
            result.push_str("    #![allow(dead_code)]\n");
            if let Some(lib_path) = self.get_package_lib_path(_path) {
                if let Ok(code) = fs::read_to_string(&lib_path) {
                    result.push_str(&code);
                    // Ensure code ends with newline
                    if !code.is_empty() && !code.ends_with('\n') {
                        result.push('\n');
                    }
                }
            }
            result.push_str("}\n\n");
            // Import all public items from the module
            result.push_str(&format!("use {}::*;\n\n", module_name));
        }

        result.push_str("/// OSL Compiled Server\n\n");

        // Generate all statements in main
        result.push_str("fn main() {\n");

        for stmt in &program.statements {
            // Skip function definitions - they'll be inlined as closures
            if matches!(stmt, Stmt::Function { name, .. } if name == "main") {
                // Generate main function body directly
                if let Stmt::Function { name: _, params: _, ret: _, body } = stmt {
                    match body.as_ref() {
                        Stmt::Block(stmts) => {
                            for s in stmts {
                                self.gen_stmt_to(&mut result, s);
                            }
                        }
                        other => {
                            self.gen_stmt_to(&mut result, other);
                        }
                    }
                }
            } else if !matches!(stmt, Stmt::Function { .. }) {
                self.gen_stmt_to(&mut result, stmt);
            } else {
                // For non-main functions, generate them inline (as closures)
                self.gen_inline_function(&mut result, stmt);
            }
        }

        result.push_str("}\n");
        result
    }

    fn gen_inline_function(&self, output: &mut String, stmt: &Stmt) {
        if let Stmt::Function { name, params, ret: _, body } = stmt {
            let params_rust: Vec<String> = params.iter()
                .map(|(n, t)| format!("{}: {}", n, self.rust_type(t)))
                .collect();
            *output += &format!("    let {} = |{}| {{\n", name, params_rust.join(", "));
            // Body is Box<Stmt> - if it's a Block, iterate over its statements
            match body.as_ref() {
                Stmt::Block(stmts) => {
                    for s in stmts {
                        self.gen_stmt_to(output, s);
                    }
                }
                other => {
                    self.gen_stmt_to(output, other);
                }
            }
            *output += "    };\n\n";
        }
    }

    fn gen_stmt_to(&self, output: &mut String, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl { name, ty, value } => {
                if let Some(v) = value {
                    let val = self.gen_expr(v);
                    if matches!(ty, Type::Infer) {
                        *output += &format!("    let {} = {};\n", name, val);
                    } else {
                        let rust_ty = self.rust_type(ty);
                        *output += &format!("    let {}: {} = {};\n", name, rust_ty, val);
                    }
                }
            }

            Stmt::Expr(Expr::Ident(s)) => {
                if s.contains('/') {
                    return;
                }
                *output += &format!("    {};\n", s);
            }

            Stmt::Function { name, params, ret, body } => {
                // Skip function definitions here - handled in main via gen_inline_function
            }

            Stmt::Expr(e) => {
                let val = self.gen_expr(e);
                *output += &format!("    {};\n", val);
            }

            Stmt::Return(val) => {
                if let Some(v) = val {
                    let gen = self.gen_expr(v);
                    *output += &format!("    return {};\n", gen);
                } else {
                    *output += "    return;\n";
                }
            }

            Stmt::If { cond, then, else_ } => {
                let cond_rust = self.gen_expr(cond);
                *output += &format!("    if {} {{\n", cond_rust);
                self.gen_stmt_to(output, then.as_ref());
                if let Some(else_stmt) = else_ {
                    *output += "    } else {\n";
                    self.gen_stmt_to(output, else_stmt.as_ref());
                }
                *output += "    }\n";
            }

            Stmt::Loop { times, body } => {
                if let Some(t) = times {
                    let n = self.gen_expr(t);
                    *output += &format!("    for _ in 0..{} {{\n", n);
                } else {
                    *output += "    loop {\n";
                }
                self.gen_stmt_to(output, body.as_ref());
                *output += "    }\n";
            }

            Stmt::While { cond, body } => {
                let cond_rust = self.gen_expr(cond);
                *output += &format!("    while {} {{\n", cond_rust);
                self.gen_stmt_to(output, body.as_ref());
                *output += "    }\n";
            }

            Stmt::For { var, iter, body } => {
                let iter_rust = self.gen_expr(iter);
                *output += &format!("    for {} in {} {{\n", var, iter_rust);
                self.gen_stmt_to(output, body.as_ref());
                *output += "    }\n";
            }

            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.gen_stmt_to(output, stmt);
                }
            }

            Stmt::Break => *output += "    break;\n",
            Stmt::Continue => *output += "    continue;\n",

            Stmt::ServerDecl { name, config } => {
                *output += &format!("    // Server: {}\n", name);
                *output += "    use actix_web::{App, HttpServer, web};\n";
                let port = config.get("port").and_then(|e| self.gen_expr(e).parse::<i32>().ok()).unwrap_or(8080);
                *output += &format!("    let port = {};\n", port);
            }

            Stmt::HttpConfig(_config) => {
                *output += "    // HTTP Configuration\n";
            }

            Stmt::Route { method, path, body: _ } => {
                *output += &format!("    // Route: {} {}\n", method, path);
            }

            Stmt::Middleware { name, .. } => {
                *output += &format!("    // Middleware: {}\n", name);
            }

            Stmt::Apply { middleware, paths } => {
                *output += &format!("    // Apply {} to {:?}\n", middleware, paths);
            }

            Stmt::Db { name, kind, .. } => {
                *output += &format!("    // Database: {} ({})\n", name, kind);
                *output += "    use sqlx::{PgPool, MySqlPool};\n";
            }

            Stmt::Cache { name, kind, .. } => {
                *output += &format!("    // Cache: {} ({})\n", name, kind);
                *output += "    use redis::Client;\n";
            }

            Stmt::LogStmt { level, message } => {
                let msg = self.gen_expr(message);
                *output += &format!("    println!(\"[{}] {{}}\", {});\n", level.to_uppercase(), msg);
            }

            Stmt::Assign { target, value } => {
                let target_rust = self.gen_expr(target);
                let value_rust = self.gen_expr(value);
                *output += &format!("    {} = {};\n", target_rust, value_rust);
            }

            _ => {}
        }
    }

    fn gen_block_to(&self, output: &mut String, stmt: &Stmt) {
        match stmt {
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.gen_stmt_to(output, stmt);
                }
            }
            _ => self.gen_stmt_to(output, stmt),
        }
    }

    fn generate_import(&self, import: &Import) {
        let path = &import.path;

        if path.starts_with("std/") {
            let pkg = path.trim_start_matches("std/");
            let module = format!("crate::runtime::{}", pkg);
            self.add_import(&format!("use {};", module));
        } else if path.starts_with("osl/") {
            let pkg = path.trim_start_matches("osl/");
            let module = format!("crate::runtime::{}", pkg);
            self.add_import(&format!("use {};", module));
        } else if path.starts_with("community/") {
            let pkg = path.trim_start_matches("community/");
            let module = format!("crate::runtime::community::{}", pkg);
            self.add_import(&format!("use {};", module));
        } else if path.contains('/') {
            self.add_registry_module(path);
        } else {
            self.add_import(&format!("use {};", path));
        }
    }

    fn add_registry_module(&self, path: &str) {
        let safe_name = path.replace('/', "_").replace('-', "_");
        self.add_module_declaration(&safe_name);
        self.registry_modules.borrow_mut().push((safe_name, path.to_string()));
    }

    fn add_module_declaration(&self, module_name: &str) {
        let decl = format!("mod {};", module_name);
        let imports = self.imports.borrow();
        if !imports.contains(&decl) {
            drop(imports);
            self.imports.borrow_mut().insert(0, decl);
        }
    }

    fn get_package_lib_path(&self, path: &str) -> Option<String> {
        package_manager::get_package_include_path(path).map(|p| {
            p.join("lib.rs")
                .to_str()
                .unwrap_or("")
                .to_string()
        }).filter(|p| !p.is_empty() && Path::new(p).exists())
    }

    fn add_import(&self, import: &str) {
        let imports = self.imports.borrow();
        if !imports.contains(&import.to_string()) {
            drop(imports);
            self.imports.borrow_mut().push(import.to_string());
        }
    }

    fn line<S: AsRef<str>>(&self, s: S) {
        self.output.borrow_mut().push_str(&"    ".repeat(self.indent));
        self.output.borrow_mut().push_str(s.as_ref());
        self.output.borrow_mut().push('\n');
    }

    fn indent(&mut self) { self.indent += 1; }
    fn dedent(&mut self) { self.indent = self.indent.saturating_sub(1); }

    fn gen_stmt(&self, stmt: &Stmt) {
        self.gen_stmt_to(&mut self.output.borrow_mut(), stmt);
    }

    fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(s) => s.clone(),

            Expr::Literal(l) => match l {
                Literal::Int(n)        => n.to_string(),
                Literal::Float(f)      => f.to_string(),
                Literal::String(s)     => format!("\"{}\"", s.replace("\"", "\\\"")),
                Literal::Bool(true)    => "true".to_string(),
                Literal::Bool(false)   => "false".to_string(),
                Literal::Null          => "None".to_string(),
            },

            Expr::Binary(left, op, right) => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                match op {
                    BinOp::Add => {
                        format!("(format!(\"{{}}{{}}\", {}, {}))", l, r)
                    }
                    BinOp::Sub    => format!("({} - {})", l, r),
                    BinOp::Mul    => format!("({} * {})", l, r),
                    BinOp::Div    => format!("({} / {})", l, r),
                    BinOp::Mod    => format!("({} % {})", l, r),
                    BinOp::Pow    => format!("{}.pow({})", l, r),
                    BinOp::Eq     => format!("({} == {})", l, r),
                    BinOp::Ne     => format!("({} != {})", l, r),
                    BinOp::Lt     => format!("({} < {})", l, r),
                    BinOp::Le     => format!("({} <= {})", l, r),
                    BinOp::Gt     => format!("({} > {})", l, r),
                    BinOp::Ge     => format!("({} >= {})", l, r),
                    BinOp::And    => format!("({} && {})", l, r),
                    BinOp::Or     => format!("({} || {})", l, r),
                    BinOp::Concat => format!("format!(\"{{}}{{}}\", {}, {})", l, r),
                    _             => format!("({} {:?} {})", l, op, r),
                }
            }

            Expr::Unary(op, expr) => {
                let e = self.gen_expr(expr);
                match op {
                    UnaryOp::Neg    => format!("-{}", e),
                    UnaryOp::Not    => format!("!{}", e),
                    UnaryOp::BitNot => format!("!{}", e),
                }
            }

            Expr::Call(func, args) => {
                let f = self.gen_expr(func);
                // Handle 'string' function calls (OSL built-in for to_string)
                if f == "string" {
                    if args.len() == 1 {
                        let arg = self.gen_expr(&args[0]);
                        return format!("{}.to_string()", arg);
                    }
                }
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

            Expr::Lambda(params, _ret, _body) => {
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
                .map(|(k, v)| format!("(\"{}\", {})", k, self.gen_expr(v)))
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

            Expr::In(_, _) => "true /* in */".to_string(),
        }
    }

    fn rust_type(&self, ty: &Type) -> String {
        match ty {
            Type::Int        => "i64".to_string(),
            Type::Float      => "f64".to_string(),
            Type::Bool       => "bool".to_string(),
            Type::Str        => "&str".to_string(),
            Type::Byte       => "u8".to_string(),
            Type::Void       => "()".to_string(),
            Type::List(t)    => format!("Vec<{}>", self.rust_type(t)),
            Type::Map(k, v)  => format!("HashMap<{}, {}>", self.rust_type(k), self.rust_type(v)),
            Type::Func(args, ret) => {
                let args_str: Vec<String> = args.iter().map(|a| self.rust_type(a)).collect();
                format!("Box<dyn Fn({}) -> {}>", args_str.join(", "), self.rust_type(ret))
            }
            Type::Custom(s)  => s.clone(),
            // FIX: Infer should never appear in a type annotation — handled at call site
            Type::Infer      => "_".to_string(),
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

    fn gen_socket_config(&mut self, _socktype: &str, config: &HashMap<String, Expr>) {
        if let Some(expr) = config.get("bind") {
            let gen = self.gen_expr(expr);
            self.line(format!("let bind_addr = {};", gen));
        }
    }

    fn gen_http_config(&mut self, _config: &HashMap<String, Expr>) {
        self.line("// HTTP config handled at runtime");
    }

    fn gen_route(&mut self, method: &str, path: &str, body: &Stmt) {
        self.line(format!("// Route handler: {} {}", method, path));
        match body {
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    match stmt {
                        Stmt::Return(Some(expr)) => {
                            let result = self.gen_expr(expr);
                            self.line(format!("// respond: {}", result));
                        }
                        _ => self.gen_stmt(stmt),
                    }
                }
            }
            _ => self.gen_stmt(body),
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
