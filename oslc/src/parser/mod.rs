use std::collections::HashMap;
use crate::lexer::{Token, Keyword, Symbol};
use crate::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    program: Program,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, program: Program::new() }
    }

    pub fn parse(&mut self) -> Program {
        let mut program = Program::new();
        while !self.is_at_end() {
            match self.peek() {
                Token::Eof => break,
                _ => program.statements.push(self.parse_stmt()),
            }
        }
        program
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if !self.is_at_end() {
            self.pos += 1;
        }
        t
    }

    fn check(&self, tok: &Token) -> bool {
        self.peek() == tok
    }

    fn check_keyword(&self, kw: &Keyword) -> bool {
        matches!(self.peek(), Token::Keyword(k) if k == kw)
    }

    fn match_token(&mut self, tok: &Token) -> bool {
        if self.check(tok) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_keyword(&mut self, kw: &Keyword) -> bool {
        if self.check_keyword(kw) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: Token, msg: &str) -> Token {
        if self.check(&tok) {
            self.advance()
        } else {
            let got = self.peek();
            // Instead of panicking, try to recover
            if got == &Token::Symbol(Symbol::RBracket) {
                return got.clone();
            }
            panic!("Expected {} at position {}, got {:?}", msg, self.pos, got)
        }
    }

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek() {
            Token::Keyword(Keyword::Import) => self.parse_import(),
            Token::Keyword(Keyword::Let) => self.parse_var_decl(),
            Token::Keyword(Keyword::Fn) => self.parse_function(),
            Token::Keyword(Keyword::If) => self.parse_if(),
            Token::Keyword(Keyword::Loop) => self.parse_loop(),
            Token::Keyword(Keyword::While) => self.parse_while(),
            Token::Keyword(Keyword::For) => self.parse_for(),
            Token::Keyword(Keyword::Return) => self.parse_return(),
            Token::Keyword(Keyword::Break) => { self.advance(); Stmt::Break },
            Token::Keyword(Keyword::Continue) => { self.advance(); Stmt::Continue },
            Token::Keyword(Keyword::Server) => self.parse_server_decl(),
            Token::Keyword(Keyword::Socket) => self.parse_socket_decl(),
            Token::Keyword(Keyword::Http) => self.parse_http_config(),
            Token::Keyword(Keyword::Route) => self.parse_route(),
            Token::Keyword(Keyword::Middleware) => self.parse_middleware(),
            Token::Keyword(Keyword::Apply) => self.parse_apply(),
            Token::Keyword(Keyword::Tls) => self.parse_tls_config(),
            Token::Keyword(Keyword::Auth) => self.parse_auth(),
            Token::Keyword(Keyword::Security) => self.parse_security(),
            Token::Keyword(Keyword::RateLimit) => self.parse_rate_limit(),
            Token::Keyword(Keyword::Process) => self.parse_process_config(),
            Token::Keyword(Keyword::Thread) => self.parse_thread_pool(),
            Token::Keyword(Keyword::Spawn) => self.parse_spawn(),
            Token::Keyword(Keyword::Upstream) => self.parse_upstream(),
            Token::Keyword(Keyword::Proxy) => self.parse_proxy(),
            Token::Keyword(Keyword::Db) => self.parse_db(),
            Token::Keyword(Keyword::Cache) => self.parse_cache(),
            Token::Keyword(Keyword::Log) => self.parse_log_config(),
            Token::Keyword(Keyword::Health) => self.parse_health(),
            Token::Keyword(Keyword::Metrics) => self.parse_metrics(),
            Token::Keyword(Keyword::Monitor) => self.parse_monitor(),
            Token::Keyword(Keyword::Static) => self.parse_static(),
            Token::Keyword(Keyword::Websocket) => self.parse_websocket(),
            Token::Keyword(Keyword::Env) => self.parse_env_config(),
            Token::Keyword(Keyword::Config) => self.parse_config_file(),
            Token::Keyword(Keyword::Try) => self.parse_try_catch(),
            Token::Keyword(Keyword::On) => self.parse_on_panic(),
            Token::Symbol(Symbol::LBrace) => Stmt::Block(self.parse_block()),
            _ => Stmt::Expr(self.parse_expr()),
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(Token::Symbol(Symbol::LBrace), "block start");
        let mut stmts = Vec::new();
        while !self.check(&Token::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            stmts.push(self.parse_stmt());
        }
        self.expect(Token::Symbol(Symbol::RBrace), "block end");
        stmts
    }

    fn parse_var_decl(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected variable name"),
        };
        self.advance();
        
        let ty = if self.match_token(&Token::Symbol(Symbol::Colon)) {
            self.parse_type()
        } else {
            Type::Infer
        };
        
        let value = if self.match_token(&Token::Symbol(Symbol::Assign)) {
            Some(self.parse_expr())
        } else {
            None
        };
        
        self.expect(Token::Symbol(Symbol::Semicolon), "var declaration");
        Stmt::VarDecl { name, ty, value }
    }

    fn parse_import(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek() {
            Token::String(s) => s.clone(),
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected import path"),
        };
        self.advance();
        
        let alias = if self.match_keyword(&Keyword::As) {
            match self.peek() {
                Token::Ident(s) => {
                    let a = s.clone();
                    self.advance();
                    Some(a)
                }
                _ => None,
            }
        } else {
            None
        };
        
        self.expect(Token::Symbol(Symbol::Semicolon), "import statement");
        
        let import = Import { path: path.clone(), alias };
        self.program.imports.push(import);
        
        Stmt::Expr(Expr::Ident(path))
    }

    fn parse_type(&mut self) -> Type {
        let tok = self.peek().clone();
        match tok {
            Token::Ident(s) => {
                self.advance();
                match s.as_str() {
                    "int" => Type::Int,
                    "float" => Type::Float,
                    "bool" => Type::Bool,
                    "str" => Type::Str,
                    "byte" => Type::Byte,
                    "void" => Type::Void,
                    _ => {
                        if self.match_token(&Token::Symbol(Symbol::Lt)) {
                            let inner = Box::new(self.parse_type());
                            self.expect(Token::Symbol(Symbol::Gt), "generic close");
                            Type::List(inner)
                        } else {
                            Type::Custom(s)
                        }
                    }
                }
            }
            Token::Keyword(Keyword::Map) => {
                self.advance();
                self.expect(Token::Symbol(Symbol::Lt), "map key type");
                let k = Box::new(self.parse_type());
                self.expect(Token::Symbol(Symbol::Comma), "map value type");
                let v = Box::new(self.parse_type());
                self.expect(Token::Symbol(Symbol::Gt), "map close");
                Type::Map(k, v)
            }
            Token::Keyword(Keyword::List) => {
                self.advance();
                self.expect(Token::Symbol(Symbol::Lt), "list element type");
                let inner = Box::new(self.parse_type());
                self.expect(Token::Symbol(Symbol::Gt), "list close");
                Type::List(inner)
            }
            _ => Type::Infer,
        }
    }

    fn parse_function(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected function name"),
        };
        self.advance();
        
        self.expect(Token::Symbol(Symbol::LParen), "function params");
        let mut params = Vec::new();
        while !self.check(&Token::Symbol(Symbol::RParen)) {
            let param_name = match self.peek() {
                Token::Ident(s) => s.clone(),
                _ => panic!("Expected parameter name"),
            };
            self.advance();
            self.expect(Token::Symbol(Symbol::Colon), "parameter type");
            let param_type = self.parse_type();
            params.push((param_name, param_type));
            if self.check(&Token::Symbol(Symbol::Comma)) {
                self.advance();
            }
        }
        self.expect(Token::Symbol(Symbol::RParen), "function params close");
        
        let ret = if self.match_keyword(&Keyword::Return) {
            self.parse_type()
        } else {
            Type::Void
        };
        
        // Handle both block and single-statement body
        let body = if self.check(&Token::Symbol(Symbol::LBrace)) {
            Box::new(Stmt::Block(self.parse_block()))
        } else if self.check(&Token::Keyword(Keyword::Return)) {
            // Single return statement
            let return_stmt = self.parse_return();
            Box::new(Stmt::Block(vec![return_stmt]))
        } else {
            Box::new(Stmt::Block(vec![]))
        };
        
        Stmt::Function { name, params, ret, body }
    }

    fn parse_if(&mut self) -> Stmt {
        self.advance();
        let cond = self.parse_expr();
        let then = Box::new(self.parse_stmt_or_block());
        
        let else_ = if self.match_keyword(&Keyword::Else) {
            Some(Box::new(self.parse_stmt_or_block()))
        } else {
            None
        };
        
        Stmt::If { cond, then, else_ }
    }

    fn parse_stmt_or_block(&mut self) -> Stmt {
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            Stmt::Block(self.parse_block())
        } else if self.check(&Token::Symbol(Symbol::Semicolon)) {
            self.advance();
            Stmt::Block(vec![])
        } else {
            // Parse single statement as a block
            let stmt = self.parse_stmt();
            Stmt::Block(vec![stmt])
        }
    }

    fn parse_loop(&mut self) -> Stmt {
        self.advance();
        let times = match self.peek() {
            Token::Int(_) => Some(self.parse_expr()),
            _ => None
        };
        let body = Box::new(self.parse_stmt_or_block());
        Stmt::Loop { times, body }
    }

    fn parse_while(&mut self) -> Stmt {
        self.advance();
        let cond = self.parse_expr();
        let body = Box::new(self.parse_stmt_or_block());
        Stmt::While { cond, body }
    }

    fn parse_for(&mut self) -> Stmt {
        self.advance();
        let var = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected loop variable"),
        };
        self.advance();
        self.expect_keyword(Keyword::In);
        let iter = self.parse_expr();
        let body = Box::new(self.parse_stmt_or_block());
        Stmt::For { var, iter, body }
    }

    fn expect_keyword(&mut self, kw: Keyword) {
        if !self.match_keyword(&kw) {
            panic!("Expected keyword {:?}", kw);
        }
    }

    fn parse_return(&mut self) -> Stmt {
        self.advance();
        if self.check(&Token::Symbol(Symbol::Semicolon)) {
            self.advance();
            Stmt::Return(None)
        } else {
            let expr = self.parse_expr();
            self.expect(Token::Symbol(Symbol::Semicolon), "return value");
            Stmt::Return(Some(expr))
        }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_comma()
    }

    fn can_start_expr(&self) -> bool {
        match self.peek() {
            Token::Int(_) | Token::Float(_) | Token::String(_) | Token::Bool(_) |
            Token::Keyword(Keyword::Null) | Token::Ident(_) | Token::Symbol(Symbol::LBracket) |
            Token::Symbol(Symbol::LBrace) | Token::Symbol(Symbol::LParen) | Token::Symbol(Symbol::Minus) |
            Token::Symbol(Symbol::Not) => true,
            _ => false,
        }
    }

    fn parse_expr_or_null(&mut self) -> Option<Expr> {
        if self.can_start_expr() {
            Some(self.parse_expr())
        } else {
            None
        }
    }

    fn parse_comma(&mut self) -> Expr {
        let mut left = self.parse_assign();
        while self.match_token(&Token::Symbol(Symbol::Comma)) {
            let right = self.parse_assign();
            left = Expr::Binary(Box::new(left), BinOp::Concat, Box::new(right));
        }
        left
    }

    fn parse_assign(&mut self) -> Expr {
        let left = self.parse_ternary();
        match self.peek() {
            Token::Symbol(Symbol::Assign) => {
                self.advance();
                let value = self.parse_assign();
                return Expr::Assign(Box::new(left), Box::new(value));
            }
            Token::Symbol(Symbol::PlusAssign) => {
                self.advance();
                let value = self.parse_assign();
                return Expr::Assign(Box::new(left.clone()), Box::new(Expr::Binary(Box::new(left), BinOp::Add, Box::new(value))));
            }
            Token::Symbol(Symbol::MinusAssign) => {
                self.advance();
                let value = self.parse_assign();
                return Expr::Assign(Box::new(left.clone()), Box::new(Expr::Binary(Box::new(left), BinOp::Sub, Box::new(value))));
            }
            Token::Symbol(Symbol::MulAssign) => {
                self.advance();
                let value = self.parse_assign();
                return Expr::Assign(Box::new(left.clone()), Box::new(Expr::Binary(Box::new(left), BinOp::Mul, Box::new(value))));
            }
            Token::Symbol(Symbol::DivAssign) => {
                self.advance();
                let value = self.parse_assign();
                return Expr::Assign(Box::new(left.clone()), Box::new(Expr::Binary(Box::new(left), BinOp::Div, Box::new(value))));
            }
            _ => left,
        }
    }

    fn parse_ternary(&mut self) -> Expr {
        let cond = self.parse_or();
        if self.match_token(&Token::Symbol(Symbol::Question)) {
            let then = Box::new(self.parse_or());
            self.expect(Token::Symbol(Symbol::Colon), "ternary else");
            let else_ = Box::new(self.parse_ternary());
            Expr::Ternary { cond: Box::new(cond), then, else_ }
        } else if self.match_token(&Token::Symbol(Symbol::Question)) && self.check(&Token::Symbol(Symbol::Question)) {
            // Null coalescing ??
            let right = Box::new(self.parse_or());
            Expr::NullCoalesce(Box::new(cond), right)
        } else {
            cond
        }
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.match_token(&Token::Symbol(Symbol::Or)) {
            let right = self.parse_and();
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right));
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_bitwise_or();
        while self.match_token(&Token::Symbol(Symbol::And)) {
            let right = self.parse_bitwise_or();
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right));
        }
        left
    }

    fn parse_bitwise_or(&mut self) -> Expr {
        let mut left = self.parse_bitwise_xor();
        while self.check(&Token::Symbol(Symbol::Pipe)) {
            self.advance();
            let right = self.parse_bitwise_xor();
            left = Expr::Binary(Box::new(left), BinOp::BitOr, Box::new(right));
        }
        left
    }

    fn parse_bitwise_xor(&mut self) -> Expr {
        let mut left = self.parse_bitwise_and();
        while self.check(&Token::Symbol(Symbol::Caret)) {
            self.advance();
            let right = self.parse_bitwise_and();
            left = Expr::Binary(Box::new(left), BinOp::BitXor, Box::new(right));
        }
        left
    }

    fn parse_bitwise_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while self.check(&Token::Symbol(Symbol::And)) {
            self.advance();
            let right = self.parse_equality();
            left = Expr::Binary(Box::new(left), BinOp::BitAnd, Box::new(right));
        }
        left
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Eq)) {
                let right = self.parse_comparison();
                left = Expr::Binary(Box::new(left), BinOp::Eq, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Ne)) {
                let right = self.parse_comparison();
                left = Expr::Binary(Box::new(left), BinOp::Ne, Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_range();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Lt)) {
                let right = self.parse_range();
                left = Expr::Binary(Box::new(left), BinOp::Lt, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Le)) {
                let right = self.parse_range();
                left = Expr::Binary(Box::new(left), BinOp::Le, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Gt)) {
                let right = self.parse_range();
                left = Expr::Binary(Box::new(left), BinOp::Gt, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Ge)) {
                let right = self.parse_range();
                left = Expr::Binary(Box::new(left), BinOp::Ge, Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_range(&mut self) -> Expr {
        let mut left = self.parse_add();
        if self.match_token(&Token::Keyword(Keyword::In)) {
            let right = self.parse_add();
            // Handle 'in' for loop iteration
            left = Expr::In(Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_add(&mut self) -> Expr {
        let mut left = self.parse_mul();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Plus)) {
                let right = self.parse_mul();
                left = Expr::Binary(Box::new(left), BinOp::Add, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Minus)) {
                let right = self.parse_mul();
                left = Expr::Binary(Box::new(left), BinOp::Sub, Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_mul(&mut self) -> Expr {
        let mut left = self.parse_pow();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Star)) {
                let right = self.parse_pow();
                left = Expr::Binary(Box::new(left), BinOp::Mul, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Slash)) {
                let right = self.parse_pow();
                left = Expr::Binary(Box::new(left), BinOp::Div, Box::new(right));
            } else if self.match_token(&Token::Symbol(Symbol::Percent)) {
                let right = self.parse_pow();
                left = Expr::Binary(Box::new(left), BinOp::Mod, Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_pow(&mut self) -> Expr {
        let left = self.parse_unary();
        if self.match_token(&Token::Symbol(Symbol::Caret)) {
            let right = self.parse_pow();
            return Expr::Binary(Box::new(left), BinOp::Pow, Box::new(right));
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        if self.match_token(&Token::Symbol(Symbol::Minus)) {
            let right = self.parse_unary();
            Expr::Unary(UnaryOp::Neg, Box::new(right))
        } else if self.match_token(&Token::Symbol(Symbol::Not)) {
            let right = self.parse_unary();
            Expr::Unary(UnaryOp::Not, Box::new(right))
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if self.match_token(&Token::Symbol(Symbol::LParen)) {
                let mut args = Vec::new();
                while !self.check(&Token::Symbol(Symbol::RParen)) {
                    args.push(self.parse_expr());
                    if self.check(&Token::Symbol(Symbol::Comma)) {
                        self.advance();
                    }
                }
                self.expect(Token::Symbol(Symbol::RParen), "call args close");
                expr = Expr::Call(Box::new(expr), args);
            } else if self.match_token(&Token::Symbol(Symbol::Dot)) {
                let field = match self.peek() {
                    Token::Ident(s) => s.clone(),
                    _ => panic!("Expected field name"),
                };
                self.advance();
                expr = Expr::Field(Box::new(expr), field);
            } else if self.match_token(&Token::Symbol(Symbol::LBracket)) {
                let index = Box::new(self.parse_expr());
                self.expect(Token::Symbol(Symbol::RBracket), "index close");
                expr = Expr::Index(Box::new(expr), index);
            } else {
                break;
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let tok = self.peek().clone();
        self.advance();
        match tok {
            Token::Int(n) => Expr::Literal(Literal::Int(n)),
            Token::Float(n) => Expr::Literal(Literal::Float(n)),
            Token::String(s) => Expr::Literal(Literal::String(s)),
            Token::Bool(b) => Expr::Literal(Literal::Bool(b)),
            Token::Keyword(Keyword::Null) => Expr::Literal(Literal::Null),
            Token::Ident(s) => Expr::Ident(s),
            Token::Keyword(kw) => {
                match kw {
                    Keyword::True => Expr::Literal(Literal::Bool(true)),
                    Keyword::False => Expr::Literal(Literal::Bool(false)),
                    _ => Expr::Ident(kw.to_string()),
                }
            }
            Token::Symbol(Symbol::LBracket) => self.parse_list(),
            Token::Symbol(Symbol::LBrace) => self.parse_map(),
            Token::Symbol(Symbol::LParen) => {
                let expr = self.parse_expr();
                self.expect(Token::Symbol(Symbol::RParen), "grouped expr");
                expr
            }
            _ => Expr::Literal(Literal::Null),
        }
    }

    fn parse_list(&mut self) -> Expr {
        // Consume LBracket if not already done (handles both cases)
        if self.check(&Token::Symbol(Symbol::LBracket)) {
            self.advance();
        }
        
        let mut items = Vec::new();
        
        // Handle empty list
        if self.check(&Token::Symbol(Symbol::RBracket)) {
            self.advance();
            return Expr::List(items);
        }
        
        // Parse items
        loop {
            if self.check(&Token::Symbol(Symbol::RBracket)) {
                self.advance();
                break;
            }
            
            let tok = self.peek().clone();
            let item = match tok {
                Token::String(s) => {
                    self.advance();
                    Expr::Literal(Literal::String(s))
                }
                Token::Int(n) => { self.advance(); Expr::Literal(Literal::Int(n)) }
                Token::Float(n) => { self.advance(); Expr::Literal(Literal::Float(n)) }
                Token::Bool(b) => { self.advance(); Expr::Literal(Literal::Bool(b)) }
                Token::Keyword(Keyword::Null) => { self.advance(); Expr::Literal(Literal::Null) }
                Token::Keyword(Keyword::True) => { self.advance(); Expr::Literal(Literal::Bool(true)) }
                Token::Keyword(Keyword::False) => { self.advance(); Expr::Literal(Literal::Bool(false)) }
                Token::Ident(s) => {
                    self.advance();
                    Expr::Literal(Literal::String(s))
                }
                Token::Symbol(Symbol::LBracket) => self.parse_list(),
                Token::Symbol(Symbol::LBrace) => self.parse_map(),
                _ => { 
                    self.advance(); 
                    Expr::Literal(Literal::Null) 
                }
            };
            items.push(item);
            
            if self.check(&Token::Symbol(Symbol::Comma)) {
                self.advance();
            } else if !self.check(&Token::Symbol(Symbol::RBracket)) {
                break;
            }
        }
        
        Expr::List(items)
    }

    fn parse_map(&mut self) -> Expr {
        self.expect(Token::Symbol(Symbol::LBrace), "map start");
        let mut map = HashMap::new();
        while !self.check(&Token::Symbol(Symbol::RBrace)) {
            let key = match self.peek() {
                Token::Ident(s) => s.clone(),
                Token::String(s) => s.clone(),
                _ => panic!("Expected map key"),
            };
            self.advance();
            self.expect(Token::Symbol(Symbol::Colon), "map key-value");
            let value = self.parse_expr();
            map.insert(key, value);
            if self.check(&Token::Symbol(Symbol::Comma)) {
                self.advance();
            }
        }
        self.expect(Token::Symbol(Symbol::RBrace), "map close");
        Expr::Map(map)
    }

    fn parse_server_decl(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::String(s) => s.clone(),
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected server name"),
        };
        self.advance();
        
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_key_value_block()
        } else {
            self.expect(Token::Symbol(Symbol::Semicolon), "server decl");
            HashMap::new()
        };
        
        Stmt::ServerDecl { name, config }
    }

    fn parse_socket_decl(&mut self) -> Stmt {
        self.advance();
        let socktype = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected socket type"),
        };
        self.advance();
        
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_key_value_block()
        } else {
            HashMap::new()
        };
        
        Stmt::SocketDecl { socktype, config }
    }

    fn parse_http_config(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::HttpConfig(config)
    }

    fn parse_route(&mut self) -> Stmt {
        self.advance();
        let method = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected HTTP method"),
        };
        self.advance();
        
        let path = match self.peek() {
            Token::String(s) => s.clone(),
            _ => panic!("Expected route path"),
        };
        self.advance();
        
        let body = Box::new(self.parse_stmt_or_block());
        
        Stmt::Route { method, path, body }
    }

    fn parse_middleware(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected middleware name"),
        };
        self.advance();
        
        let mut config = HashMap::new();
        let mut before = None;
        let mut after = None;
        
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            let block = self.parse_block();
            for stmt in block {
                match stmt {
                    Stmt::Block(_) => {
                        if before.is_none() {
                            before = Some(Box::new(stmt));
                        } else if after.is_none() {
                            after = Some(Box::new(stmt));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Stmt::Middleware { name, config, before, after }
    }

    fn parse_apply(&mut self) -> Stmt {
        self.advance();
        let middleware = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected middleware name"),
        };
        self.advance();
        
        self.expect_keyword(Keyword::To);
        
        let mut paths = Vec::new();
        if self.check(&Token::Keyword(Keyword::All)) {
            self.advance();
            paths.push("*".to_string());
        } else {
            while !self.check(&Token::Symbol(Symbol::Semicolon)) && !self.is_at_end() {
                match self.peek() {
                    Token::String(s) => paths.push(s.clone()),
                    Token::Ident(s) => paths.push(s.clone()),
                    _ => {}
                }
                self.advance();
            }
        }
        
        self.expect(Token::Symbol(Symbol::Semicolon), "apply statement");
        
        Stmt::Apply { middleware, paths }
    }

    fn parse_tls_config(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::TlsConfig(config)
    }

    fn parse_auth(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected auth name"),
        };
        self.advance();
        
        // Auth syntax is: auth <name> { config }
        // where name is the kind (jwt, basic, api_key)
        let kind = if self.check(&Token::Symbol(Symbol::LBrace)) {
            name.clone()
        } else {
            // Has explicit kind token
            match self.peek() {
                Token::Ident(s) => s.clone(),
                _ => name.clone(),
            }
        };
        
        if !self.check(&Token::Symbol(Symbol::LBrace)) {
            self.advance();
        }
        
        let config = self.parse_key_value_block();
        
        Stmt::Auth { name, kind, config }
    }

    fn parse_security(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::Security(config)
    }

    fn parse_rate_limit(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected rate limit name"),
        };
        self.advance();
        
        let config = self.parse_key_value_block();
        
        Stmt::RateLimit { name, config }
    }

    fn parse_process_config(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::ProcessConfig(config)
    }

    fn parse_thread_pool(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected thread pool name"),
        };
        self.advance();
        
        let config = self.parse_key_value_block();
        
        Stmt::ThreadPool { name, config }
    }

    fn parse_spawn(&mut self) -> Stmt {
        self.advance();
        let kind = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected spawn kind"),
        };
        self.advance();
        
        let mut config = HashMap::new();
        let body;
        
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            body = Box::new(Stmt::Block(self.parse_block()));
        } else {
            body = Box::new(Stmt::Block(vec![]));
        }
        
        Stmt::Spawn { kind, config, body }
    }

    fn parse_upstream(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected upstream name"),
        };
        self.advance();
        
        let mut config = HashMap::new();
        let mut servers = Vec::new();
        
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            let block = self.parse_block();
            for stmt in block {
                match stmt {
                    Stmt::VarDecl { name, value, .. } => {
                        if let Some(Expr::Literal(Literal::String(s))) = value {
                            servers.push((name, Expr::Literal(Literal::String(s))));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Stmt::Upstream { name, config, servers }
    }

    fn parse_proxy(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek() {
            Token::String(s) => s.clone(),
            _ => panic!("Expected proxy path"),
        };
        self.advance();
        
        self.expect(Token::Symbol(Symbol::Arrow), "proxy target");
        
        let target = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected proxy target"),
        };
        self.advance();
        
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_key_value_block()
        } else {
            HashMap::new()
        };
        
        Stmt::Proxy { path, target, config }
    }

    fn parse_db(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected database name"),
        };
        self.advance();
        
        let kind = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected database kind"),
        };
        self.advance();
        
        let config = self.parse_key_value_block();
        
        Stmt::Db { name, kind, config }
    }

    fn parse_cache(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected cache name"),
        };
        self.advance();
        
        let kind = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected cache kind"),
        };
        self.advance();
        
        let config = self.parse_key_value_block();
        
        Stmt::Cache { name, kind, config }
    }

    fn parse_log_config(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::LogConfig(config)
    }

    fn parse_health(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::Health(config)
    }

    fn parse_metrics(&mut self) -> Stmt {
        self.advance();
        let kind = match self.peek() {
            Token::Ident(s) => s.clone(),
            _ => panic!("Expected metrics kind"),
        };
        self.advance();
        
        let config = self.parse_key_value_block();
        
        Stmt::Metrics { kind, config }
    }

    fn parse_monitor(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::Monitor(config)
    }

    fn parse_static(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek() {
            Token::String(s) => s.clone(),
            _ => panic!("Expected static path"),
        };
        self.advance();
        
        self.expect(Token::Symbol(Symbol::Arrow), "static root");
        
        let root = match self.peek() {
            Token::String(s) => s.clone(),
            _ => panic!("Expected static root"),
        };
        self.advance();
        
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_key_value_block()
        } else {
            HashMap::new()
        };
        
        Stmt::Static { path, root, config }
    }

    fn parse_websocket(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek() {
            Token::String(s) => s.clone(),
            _ => panic!("Expected websocket path"),
        };
        self.advance();
        
        let mut config = HashMap::new();
        let mut handlers = HashMap::new();
        
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            let block = self.parse_block();
            for stmt in block {
                match stmt {
                    Stmt::Block(stmts) => handlers.insert("handler".to_string(), Box::new(Stmt::Block(stmts))),
                    _ => None,
                };
            }
        }
        
        Stmt::WebSocket { path, config, handlers }
    }

    fn parse_env_config(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::EnvConfig(config)
    }

    fn parse_config_file(&mut self) -> Stmt {
        self.advance();
        let config = self.parse_key_value_block();
        Stmt::ConfigFile(config)
    }

    fn parse_try_catch(&mut self) -> Stmt {
        self.advance();
        let try_block = Box::new(self.parse_stmt_or_block());
        
        let mut catches = Vec::new();
        while self.match_keyword(&Keyword::Catch) {
            let err_type = match self.peek() {
                Token::Ident(s) => s.clone(),
                _ => "Error".to_string(),
            };
            self.advance();
            
            let err_var = match self.peek() {
                Token::Ident(s) => s.clone(),
                _ => "err".to_string(),
            };
            self.advance();
            
            let catch_body = Box::new(self.parse_stmt_or_block());
            catches.push((err_type, err_var, catch_body));
        }
        
        let finally = if self.match_keyword(&Keyword::Finally) {
            Some(Box::new(self.parse_stmt_or_block()))
        } else {
            None
        };
        
        Stmt::TryCatch { try_block, catches, finally }
    }

    fn parse_on_panic(&mut self) -> Stmt {
        self.advance();
        let body = Box::new(self.parse_stmt_or_block());
        Stmt::OnPanic { body }
    }

    fn parse_key_value_block(&mut self) -> HashMap<String, Expr> {
        self.expect(Token::Symbol(Symbol::LBrace), "config block start");
        let mut config = HashMap::new();
        
        while !self.check(&Token::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            let key = match self.peek() {
                Token::Ident(s) => s.clone(),
                Token::String(s) => s.clone(),
                _ => {
                    self.advance();
                    continue;
                }
            };
            self.advance();
            
            if self.match_token(&Token::Symbol(Symbol::Colon)) {
                let value = self.parse_expr();
                if self.check(&Token::Symbol(Symbol::Semicolon)) {
                    self.advance();
                }
                config.insert(key, value);
            } else if self.check(&Token::Symbol(Symbol::LBracket)) {
                let value = self.parse_list();
                if self.check(&Token::Symbol(Symbol::Semicolon)) {
                    self.advance();
                }
                config.insert(key, value);
            } else if self.check(&Token::Symbol(Symbol::LBrace)) {
                let value = self.parse_expr();
                if self.check(&Token::Symbol(Symbol::Semicolon)) {
                    self.advance();
                }
                config.insert(key, value);
            } else {
                let value = if let Some(v) = self.parse_expr_or_null() {
                    v
                } else {
                    Expr::Ident(key.clone())
                };
                
                if self.check(&Token::Symbol(Symbol::Semicolon)) {
                    self.advance();
                }
                config.insert(key, value);
            }
        }
        
        self.expect(Token::Symbol(Symbol::RBrace), "config block end");
        config
    }
}