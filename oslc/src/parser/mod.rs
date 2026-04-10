use std::collections::HashMap;
use crate::lexer::{Token, Keyword, Symbol};
use crate::ast::*;

// ─────────────────────────────────────────────
//  Parser
// ─────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos:    usize,
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
                Token::Symbol(Symbol::Semicolon) => { self.advance(); }
                _ => program.statements.push(self.parse_stmt()),
            }
        }
        program.imports = self.program.imports.clone();
        program
    }

    // ── primitives ────────────────────────────

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek_next(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if !self.is_at_end() { self.pos += 1; }
        t
    }

    fn check(&self, tok: &Token) -> bool {
        self.peek() == tok
    }

    fn check_keyword(&self, kw: &Keyword) -> bool {
        matches!(self.peek(), Token::Keyword(k) if k == kw)
    }

    fn match_token(&mut self, tok: &Token) -> bool {
        if self.check(tok) { self.advance(); true } else { false }
    }

    fn match_keyword(&mut self, kw: &Keyword) -> bool {
        if self.check_keyword(kw) { self.advance(); true } else { false }
    }

    /// Consume a token or panic with position + got token.
    fn expect(&mut self, tok: Token, msg: &str) -> Token {
        if self.check(&tok) {
            self.advance()
        } else {
            panic!(
                "Parse error at pos {}: expected {} ({:?}), got {:?}",
                   self.pos, msg, tok, self.peek()
            )
        }
    }

    fn expect_keyword(&mut self, kw: Keyword) {
        if !self.match_keyword(&kw) {
            panic!("Parse error at pos {}: expected keyword {:?}, got {:?}", self.pos, kw, self.peek());
        }
    }

    /// Skip an optional semicolon.
    fn skip_semi(&mut self) {
        self.match_token(&Token::Symbol(Symbol::Semicolon));
    }

    // ── statement dispatch ────────────────────

    fn parse_stmt(&mut self) -> Stmt {
        match self.peek().clone() {
            Token::Keyword(Keyword::Import)     => self.parse_import(),
            Token::Keyword(Keyword::Let)        => self.parse_var_decl(),
            Token::Keyword(Keyword::Fn)         => self.parse_function(),
            Token::Keyword(Keyword::If)         => self.parse_if(),
            Token::Keyword(Keyword::Loop)       => self.parse_loop(),
            Token::Keyword(Keyword::While)      => self.parse_while(),
            Token::Keyword(Keyword::For)        => self.parse_for(),
            Token::Keyword(Keyword::Return)     => self.parse_return(),
            Token::Keyword(Keyword::Respond)    => self.parse_respond(),
            Token::Keyword(Keyword::Break)      => { self.advance(); self.skip_semi(); Stmt::Break }
            Token::Keyword(Keyword::Continue)   => { self.advance(); self.skip_semi(); Stmt::Continue }
            Token::Keyword(Keyword::Server)     => self.parse_server_decl(),
            Token::Keyword(Keyword::Socket)     => self.parse_socket_decl(),
            Token::Keyword(Keyword::Http)       => self.parse_http_config(),
            Token::Keyword(Keyword::Route)      => self.parse_route(),
            Token::Keyword(Keyword::Middleware) => self.parse_middleware(),
            Token::Keyword(Keyword::Apply)      => self.parse_apply(),
            Token::Keyword(Keyword::Tls)        => self.parse_tls_config(),
            Token::Keyword(Keyword::Auth)       => self.parse_auth(),
            Token::Keyword(Keyword::Security)   => self.parse_security(),
            Token::Keyword(Keyword::RateLimit)  => self.parse_rate_limit(),
            Token::Keyword(Keyword::Process)    => self.parse_process_config(),
            Token::Keyword(Keyword::Thread)     => self.parse_thread_pool(),
            Token::Keyword(Keyword::Spawn)      => self.parse_spawn(),
            Token::Keyword(Keyword::Upstream)   => self.parse_upstream(),
            Token::Keyword(Keyword::Proxy)      => self.parse_proxy(),
            Token::Keyword(Keyword::Db)         => self.parse_db(),
            Token::Keyword(Keyword::Cache)      => self.parse_cache(),
            Token::Keyword(Keyword::Log)        => self.parse_log(),
            Token::Keyword(Keyword::Health)     => self.parse_health(),
            Token::Keyword(Keyword::Metrics)    => self.parse_metrics(),
            Token::Keyword(Keyword::Monitor)    => self.parse_monitor(),
            Token::Keyword(Keyword::Static)     => self.parse_static(),
            Token::Keyword(Keyword::Websocket)  => self.parse_websocket(),
            Token::Keyword(Keyword::Env)        => self.parse_env_config(),
            Token::Keyword(Keyword::Config)     => self.parse_config_file(),
            Token::Keyword(Keyword::Try)        => self.parse_try_catch(),
            Token::Keyword(Keyword::On)         => self.parse_on_panic(),
            Token::Keyword(Keyword::Request)    => self.parse_respond(),  // stub
            Token::Keyword(Keyword::Response) => self.parse_respond(), // stub
            Token::Keyword(Keyword::RequestBody) => self.parse_respond(), // stub
            Token::Keyword(Keyword::Params)    => self.parse_respond(), // stub
            Token::Keyword(Keyword::Import)    => self.parse_respond(), // stub
            Token::Keyword(Keyword::As)        => self.parse_respond(), // stub
            Token::Keyword(Keyword::From)     => self.parse_respond(), // stub
            Token::Keyword(Keyword::Map)       => self.parse_respond(), // stub
            Token::Keyword(Keyword::List)      => self.parse_respond(), // stub
            Token::Keyword(Keyword::Static)    => self.parse_respond(), // stub
            Token::Keyword(Keyword::To)        => self.parse_respond(), // stub
            Token::Keyword(Keyword::All)       => self.parse_respond(), // stub
            Token::Keyword(Keyword::Apply)    => self.parse_respond(), // stub
            Token::Keyword(Keyword::Config)   => self.parse_respond(), // stub
            Token::Keyword(Keyword::Try)      => self.parse_respond(), // stub
            Token::Keyword(Keyword::Catch)    => self.parse_respond(), // stub
            Token::Keyword(Keyword::Finally)   => self.parse_respond(), // stub
            Token::Keyword(Keyword::Throw)   => self.parse_respond(), // stub
            Token::Keyword(Keyword::True)   => self.parse_respond(), // stub
            Token::Keyword(Keyword::False)  => self.parse_respond(), // stub
            Token::Keyword(Keyword::Null)   => self.parse_respond(), // stub
            Token::Symbol(Symbol::LBrace)       => Stmt::Block(self.parse_block()),
            Token::Symbol(Symbol::Semicolon)    => { self.advance(); Stmt::Block(vec![]) }
            _ => {
                let expr = self.parse_expr();
                // assignment statement: target = value;
                if self.match_token(&Token::Symbol(Symbol::Assign)) {
                    let value = self.parse_expr();
                    self.skip_semi();
                    return Stmt::Assign { target: expr, value };
                }
                self.skip_semi();
                Stmt::Expr(expr)
            }
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(Token::Symbol(Symbol::LBrace), "'{' to open block");
        let mut stmts = Vec::new();
        while !self.check(&Token::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            if self.check(&Token::Symbol(Symbol::Semicolon)) {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt());
        }
        self.expect(Token::Symbol(Symbol::RBrace), "'}' to close block");
        stmts
    }

    /// Parse either a `{ block }` or a single statement.
    fn parse_body(&mut self) -> Stmt {
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            Stmt::Block(self.parse_block())
        } else if self.check(&Token::Symbol(Symbol::Semicolon)) {
            self.advance();
            Stmt::Block(vec![])
        } else {
            self.parse_stmt()
        }
    }

    // ── declarations ──────────────────────────

    fn parse_import(&mut self) -> Stmt {
        self.advance(); // consume `import`
        let mut path = match self.peek().clone() {
            Token::String(s) | Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected import path at pos {}, got {:?}", self.pos, t),
        };
        // Handle path separators like "discord/tester"
        while self.match_token(&Token::Symbol(Symbol::Slash)) {
            match self.peek().clone() {
                Token::Ident(s) => {
                    path.push('/');
                    path.push_str(&s);
                    self.advance();
                }
                _ => panic!("Expected identifier after '/' in import path at pos {}", self.pos),
            }
        }
        let alias = if self.match_keyword(&Keyword::As) {
            match self.peek().clone() {
                Token::Ident(s) => { self.advance(); Some(s) }
                _ => None,
            }
        } else {
            None
        };
        self.skip_semi();
        let import = Import { path: path.clone(), alias };
        self.program.imports.push(import);
        Stmt::Expr(Expr::Ident(path))
    }

    fn parse_var_decl(&mut self) -> Stmt {
        self.advance(); // consume `let`
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected variable name at pos {}, got {:?}", self.pos, t),
        };
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
        self.skip_semi();
        Stmt::VarDecl { name, ty, value }
    }

    fn parse_function(&mut self) -> Stmt {
        self.advance(); // consume `fn`
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected function name at pos {}, got {:?}", self.pos, t),
        };
        self.expect(Token::Symbol(Symbol::LParen), "'(' for params");
        let mut params = Vec::new();
        while !self.check(&Token::Symbol(Symbol::RParen)) && !self.is_at_end() {
            let pname = match self.peek().clone() {
                Token::Ident(s) => { self.advance(); s }
                t => panic!("Expected parameter name at pos {}, got {:?}", self.pos, t),
            };
            self.expect(Token::Symbol(Symbol::Colon), "':' after param name");
            let ptype = self.parse_type();
            params.push((pname, ptype));
            if !self.check(&Token::Symbol(Symbol::RParen)) {
                self.match_token(&Token::Symbol(Symbol::Comma));
            }
        }
        self.expect(Token::Symbol(Symbol::RParen), "')' after params");
        let ret = if self.match_token(&Token::Symbol(Symbol::Arrow)) {
            self.parse_type()
        } else {
            Type::Void
        };
        let body = Box::new(if self.check(&Token::Symbol(Symbol::LBrace)) {
            Stmt::Block(self.parse_block())
        } else {
            Stmt::Block(vec![])
        });
        Stmt::Function { name, params, ret, body }
    }

    // ── control flow ──────────────────────────

    fn parse_if(&mut self) -> Stmt {
        self.advance(); // consume `if`
        let cond = self.parse_expr();
        let then = Box::new(self.parse_body());
        let else_ = if self.match_keyword(&Keyword::Else) {
            Some(Box::new(self.parse_body()))
        } else {
            None
        };
        Stmt::If { cond, then, else_ }
    }

    fn parse_loop(&mut self) -> Stmt {
        self.advance(); // consume `loop`
        // optional count: `loop 10 { }` or `loop { }`
        let times = if matches!(self.peek(), Token::Int(_) | Token::Float(_)) {
            Some(self.parse_primary())
        } else {
            None
        };
        let body = Box::new(self.parse_body());
        Stmt::Loop { times, body }
    }

    fn parse_while(&mut self) -> Stmt {
        self.advance(); // consume `while`
        let cond = self.parse_expr();
        let body = Box::new(self.parse_body());
        Stmt::While { cond, body }
    }

    fn parse_for(&mut self) -> Stmt {
        self.advance(); // consume `for`
        let var = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected loop variable at pos {}, got {:?}", self.pos, t),
        };
        self.expect_keyword(Keyword::In);
        let iter = self.parse_expr();
        let body = Box::new(self.parse_body());
        Stmt::For { var, iter, body }
    }

    fn parse_return(&mut self) -> Stmt {
        self.advance(); // consume `return`
        if self.check(&Token::Symbol(Symbol::Semicolon)) || self.check(&Token::Symbol(Symbol::RBrace)) {
            self.skip_semi();
            Stmt::Return(None)
        } else {
            let expr = self.parse_expr();
            self.skip_semi();
            Stmt::Return(Some(expr))
        }
    }

    fn parse_respond(&mut self) -> Stmt {
        self.advance(); // consume `respond`
        
        // Parse respond arguments: status code, body
        if self.can_start_expr() {
            self.parse_expr();
            if self.can_start_expr() {
                self.parse_expr();
            }
        }
        
        self.skip_semi();
        Stmt::Block(vec![])
    }

    // ── types ─────────────────────────────────

    fn parse_type(&mut self) -> Type {
        match self.peek().clone() {
            Token::Ident(s) => {
                self.advance();
                match s.as_str() {
                    "int"   => Type::Int,
                    "float" => Type::Float,
                    "bool"  => Type::Bool,
                    "str"   => Type::Str,
                    "byte"  => Type::Byte,
                    "void"  => Type::Void,
                    "list"  => {
                        self.expect(Token::Symbol(Symbol::Lt), "'<' for list type");
                        let inner = Box::new(self.parse_type());
                        self.expect(Token::Symbol(Symbol::Gt), "'>' to close list type");
                        Type::List(inner)
                    }
                    "map" => {
                        self.expect(Token::Symbol(Symbol::Lt), "'<' for map key type");
                        let k = Box::new(self.parse_type());
                        self.expect(Token::Symbol(Symbol::Comma), "','");
                        let v = Box::new(self.parse_type());
                        self.expect(Token::Symbol(Symbol::Gt), "'>'");
                        Type::Map(k, v)
                    }
                    _ => {
                        if self.match_token(&Token::Symbol(Symbol::Lt)) {
                            let inner = Box::new(self.parse_type());
                            self.expect(Token::Symbol(Symbol::Gt), "'>'");
                            Type::List(inner)
                        } else {
                            Type::Custom(s)
                        }
                    }
                }
            }
            Token::Keyword(Keyword::Map) => {
                self.advance();
                self.expect(Token::Symbol(Symbol::Lt), "'<'");
                let k = Box::new(self.parse_type());
                self.expect(Token::Symbol(Symbol::Comma), "','");
                let v = Box::new(self.parse_type());
                self.expect(Token::Symbol(Symbol::Gt), "'>'");
                Type::Map(k, v)
            }
            Token::Keyword(Keyword::List) => {
                self.advance();
                self.expect(Token::Symbol(Symbol::Lt), "'<'");
                let inner = Box::new(self.parse_type());
                self.expect(Token::Symbol(Symbol::Gt), "'>'");
                Type::List(inner)
            }
            _ => Type::Infer,
        }
    }

    // ── expressions ───────────────────────────
    //
    // Precedence (low → high):
    //   assign → ternary → null-coalesce → or → and
    //   → bitwise-or → bitwise-xor → bitwise-and
    //   → equality → comparison → concat(++) → add → mul → pow
    //   → unary → postfix(call/field/index) → primary

    fn parse_expr(&mut self) -> Expr {
        self.parse_assign()
    }

    fn can_start_expr(&self) -> bool {
        matches!(self.peek(),
                 Token::Int(_) | Token::Float(_) | Token::String(_) | Token::Bool(_) |
                 Token::Keyword(Keyword::Null)  | Token::Keyword(Keyword::True)  |
                 Token::Keyword(Keyword::False) | Token::Ident(_) |
                 Token::Symbol(Symbol::LBracket) | Token::Symbol(Symbol::LBrace) |
                 Token::Symbol(Symbol::LParen)   | Token::Symbol(Symbol::Minus)  |
                 Token::Symbol(Symbol::Not)
        )
    }

    fn parse_assign(&mut self) -> Expr {
        let left = self.parse_ternary();
        match self.peek().clone() {
            Token::Symbol(Symbol::Assign) => {
                self.advance();
                let right = self.parse_assign();
                Expr::Assign(Box::new(left), Box::new(right))
            }
            Token::Symbol(Symbol::PlusAssign) => {
                self.advance();
                let right = self.parse_assign();
                Expr::Assign(Box::new(left.clone()),
                             Box::new(Expr::Binary(Box::new(left), BinOp::Add, Box::new(right))))
            }
            Token::Symbol(Symbol::MinusAssign) => {
                self.advance();
                let right = self.parse_assign();
                Expr::Assign(Box::new(left.clone()),
                             Box::new(Expr::Binary(Box::new(left), BinOp::Sub, Box::new(right))))
            }
            Token::Symbol(Symbol::MulAssign) => {
                self.advance();
                let right = self.parse_assign();
                Expr::Assign(Box::new(left.clone()),
                             Box::new(Expr::Binary(Box::new(left), BinOp::Mul, Box::new(right))))
            }
            Token::Symbol(Symbol::DivAssign) => {
                self.advance();
                let right = self.parse_assign();
                Expr::Assign(Box::new(left.clone()),
                             Box::new(Expr::Binary(Box::new(left), BinOp::Div, Box::new(right))))
            }
            _ => left,
        }
    }

    fn parse_ternary(&mut self) -> Expr {
        let cond = self.parse_null_coalesce();
        if self.match_token(&Token::Symbol(Symbol::Question)) {
            let then = Box::new(self.parse_null_coalesce());
            self.expect(Token::Symbol(Symbol::Colon), "':' in ternary");
            let else_ = Box::new(self.parse_ternary());
            Expr::Ternary { cond: Box::new(cond), then, else_ }
        } else {
            cond
        }
    }

    fn parse_null_coalesce(&mut self) -> Expr {
        let left = self.parse_or();
        // look for `??`
        if self.check(&Token::Symbol(Symbol::Question)) &&
            matches!(self.peek_next(), Token::Symbol(Symbol::Question)) {
                self.advance(); self.advance();
                let right = self.parse_or();
                Expr::NullCoalesce(Box::new(left), Box::new(right))
            } else {
                left
            }
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.match_token(&Token::Symbol(Symbol::Or)) {
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(self.parse_and()));
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_bitwise_or();
        while self.match_token(&Token::Symbol(Symbol::And)) {
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(self.parse_bitwise_or()));
        }
        left
    }

    fn parse_bitwise_or(&mut self) -> Expr {
        let mut left = self.parse_bitwise_xor();
        while self.match_token(&Token::Symbol(Symbol::Pipe)) {
            left = Expr::Binary(Box::new(left), BinOp::BitOr, Box::new(self.parse_bitwise_xor()));
        }
        left
    }

    fn parse_bitwise_xor(&mut self) -> Expr {
        let mut left = self.parse_bitwise_and();
        while self.match_token(&Token::Symbol(Symbol::Caret)) {
            left = Expr::Binary(Box::new(left), BinOp::BitXor, Box::new(self.parse_bitwise_and()));
        }
        left
    }

    // FIX 1: Symbol::Ampersand doesn't exist in the lexer — pass through to parse_equality.
    // To enable bitwise-and later, add `Ampersand` to the lexer's Symbol enum and
    // update next_token() to emit it for `&`, then restore the loop here.
    fn parse_bitwise_and(&mut self) -> Expr {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_comparison();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Eq)) {
                left = Expr::Binary(Box::new(left), BinOp::Eq, Box::new(self.parse_comparison()));
            } else if self.match_token(&Token::Symbol(Symbol::Ne)) {
                left = Expr::Binary(Box::new(left), BinOp::Ne, Box::new(self.parse_comparison()));
            } else {
                break;
            }
        }
        left
    }

    fn parse_comparison(&mut self) -> Expr {
        let mut left = self.parse_concat();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Lt)) {
                left = Expr::Binary(Box::new(left), BinOp::Lt, Box::new(self.parse_concat()));
            } else if self.match_token(&Token::Symbol(Symbol::Le)) {
                left = Expr::Binary(Box::new(left), BinOp::Le, Box::new(self.parse_concat()));
            } else if self.match_token(&Token::Symbol(Symbol::Gt)) {
                left = Expr::Binary(Box::new(left), BinOp::Gt, Box::new(self.parse_concat()));
            } else if self.match_token(&Token::Symbol(Symbol::Ge)) {
                left = Expr::Binary(Box::new(left), BinOp::Ge, Box::new(self.parse_concat()));
            } else if self.match_keyword(&Keyword::In) {
                left = Expr::In(Box::new(left), Box::new(self.parse_concat()));
            } else {
                break;
            }
        }
        left
    }

    // FIX 2: Symbol::PlusPlus doesn't exist in the lexer — pass through to parse_add.
    // To enable `++` string concat later, add `PlusPlus` to the lexer's Symbol enum,
    // emit it in next_token() when two `+` are seen, then restore the loop here.
    fn parse_concat(&mut self) -> Expr {
        self.parse_add()
    }

    fn parse_add(&mut self) -> Expr {
        let mut left = self.parse_mul();
        loop {
            if self.match_token(&Token::Symbol(Symbol::Plus)) {
                left = Expr::Binary(Box::new(left), BinOp::Add, Box::new(self.parse_mul()));
            } else if self.match_token(&Token::Symbol(Symbol::Minus)) {
                left = Expr::Binary(Box::new(left), BinOp::Sub, Box::new(self.parse_mul()));
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
                left = Expr::Binary(Box::new(left), BinOp::Mul, Box::new(self.parse_pow()));
            } else if self.match_token(&Token::Symbol(Symbol::Slash)) {
                left = Expr::Binary(Box::new(left), BinOp::Div, Box::new(self.parse_pow()));
            } else if self.match_token(&Token::Symbol(Symbol::Percent)) {
                left = Expr::Binary(Box::new(left), BinOp::Mod, Box::new(self.parse_pow()));
            } else {
                break;
            }
        }
        left
    }

    // FIX 3: Symbol::StarStar doesn't exist in the lexer — pass through to parse_unary.
    // To enable `**` exponentiation later, add `StarStar` to the lexer's Symbol enum,
    // emit it in next_token() when two `*` are seen, then restore the match here.
    fn parse_pow(&mut self) -> Expr {
        self.parse_unary()
    }

    // FIX 4: Symbol::Tilde doesn't exist in the lexer — remove that arm.
    // To enable bitwise-not (~) later, add `Tilde` to the lexer's Symbol enum,
    // emit it in next_token() for `~`, then restore the arm here.
    fn parse_unary(&mut self) -> Expr {
        if self.match_token(&Token::Symbol(Symbol::Minus)) {
            Expr::Unary(UnaryOp::Neg, Box::new(self.parse_unary()))
        } else if self.match_token(&Token::Symbol(Symbol::Not)) {
            Expr::Unary(UnaryOp::Not, Box::new(self.parse_unary()))
        } else {
            self.parse_postfix()
        }
    }

    /// Handles chained call(), .field, [index].
    fn parse_postfix(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if self.match_token(&Token::Symbol(Symbol::LParen)) {
                let mut args = Vec::new();
                while !self.check(&Token::Symbol(Symbol::RParen)) && !self.is_at_end() {
                    args.push(self.parse_expr());
                    if !self.check(&Token::Symbol(Symbol::RParen)) {
                        self.match_token(&Token::Symbol(Symbol::Comma));
                    }
                }
                self.expect(Token::Symbol(Symbol::RParen), "')' to close call");
                expr = Expr::Call(Box::new(expr), args);
            } else if self.match_token(&Token::Symbol(Symbol::Dot)) {
                let field = match self.peek().clone() {
                    Token::Ident(s)    => { self.advance(); s }
                    Token::Keyword(kw) => { self.advance(); kw.to_string() }
                    t => panic!("Expected field name at pos {}, got {:?}", self.pos, t),
                };
                expr = Expr::Field(Box::new(expr), field);
            } else if self.match_token(&Token::Symbol(Symbol::LBracket)) {
                let index = Box::new(self.parse_expr());
                self.expect(Token::Symbol(Symbol::RBracket), "']' to close index");
                expr = Expr::Index(Box::new(expr), index);
            } else {
                break;
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Int(n)    => { self.advance(); Expr::Literal(Literal::Int(n)) }
            Token::Float(n)  => { self.advance(); Expr::Literal(Literal::Float(n)) }
            Token::String(s) => { self.advance(); Expr::Literal(Literal::String(s)) }
            Token::Bool(b)   => { self.advance(); Expr::Literal(Literal::Bool(b)) }

            Token::Keyword(Keyword::True)  => { self.advance(); Expr::Literal(Literal::Bool(true)) }
            Token::Keyword(Keyword::False) => { self.advance(); Expr::Literal(Literal::Bool(false)) }
            Token::Keyword(Keyword::Null)  => { self.advance(); Expr::Literal(Literal::Null) }

            Token::Ident(s) => { self.advance(); Expr::Ident(s) }

            // keywords usable as identifiers in call position
            Token::Keyword(kw) => { self.advance(); Expr::Ident(kw.to_string()) }

            Token::Symbol(Symbol::LBracket) => { self.advance(); self.parse_list_body() }
            Token::Symbol(Symbol::LBrace)   => self.parse_map(),
            Token::Symbol(Symbol::LParen)   => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(Token::Symbol(Symbol::RParen), "')' to close grouped expr");
                expr
            }
            // graceful recovery — emit null, don't advance (caller decides)
            _ => Expr::Literal(Literal::Null),
        }
    }

    // ── list / map literals ───────────────────

    /// Called AFTER the opening `[` has been consumed.
    fn parse_list_body(&mut self) -> Expr {
        let mut items = Vec::new();
        while !self.check(&Token::Symbol(Symbol::RBracket)) && !self.is_at_end() {
            items.push(self.parse_expr());
            if !self.check(&Token::Symbol(Symbol::RBracket)) {
                self.match_token(&Token::Symbol(Symbol::Comma));
            }
        }
        self.expect(Token::Symbol(Symbol::RBracket), "']' to close list");
        Expr::List(items)
    }

    // FIX 5: Expr::Map expects HashMap<String, Expr> but the original built Vec<(String, Expr)>.
    // Now collects into a HashMap via .insert().
    fn parse_map(&mut self) -> Expr {
        self.expect(Token::Symbol(Symbol::LBrace), "'{' to open map");
        let mut fields = HashMap::new();
        while !self.check(&Token::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            let key = match self.peek().clone() {
                Token::Ident(s)    => { self.advance(); s }
                Token::String(s)   => { self.advance(); s }
                Token::Keyword(kw) => { self.advance(); kw.to_string() }
                t => panic!("Expected map key at pos {}, got {:?}", self.pos, t),
            };
            self.expect(Token::Symbol(Symbol::Colon), "':' after map key");
            let value = self.parse_expr();
            fields.insert(key, value);
            if !self.check(&Token::Symbol(Symbol::RBrace)) {
                self.match_token(&Token::Symbol(Symbol::Comma)); // trailing comma OK
            }
        }
        self.expect(Token::Symbol(Symbol::RBrace), "'}' to close map");
        Expr::Map(fields)
    }

    // ── server / network statements ───────────

    fn parse_server_decl(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::String(s) | Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected server name at pos {}, got {:?}", self.pos, t),
        };
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_kv_block()
        } else {
            self.skip_semi();
            HashMap::new()
        };
        Stmt::ServerDecl { name, config }
    }

    fn parse_socket_decl(&mut self) -> Stmt {
        self.advance();
        let socktype = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected socket type at pos {}, got {:?}", self.pos, t),
        };
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_kv_block()
        } else { HashMap::new() };
        Stmt::SocketDecl { socktype, config }
    }

    fn parse_http_config(&mut self) -> Stmt {
        self.advance();
        Stmt::HttpConfig(self.parse_kv_block())
    }

    fn parse_route(&mut self) -> Stmt {
        self.advance();
        let method = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected HTTP method at pos {}, got {:?}", self.pos, t),
        };
        let path = match self.peek().clone() {
            Token::String(s) => { self.advance(); s }
            t => panic!("Expected route path string at pos {}, got {:?}", self.pos, t),
        };
        let body = Box::new(self.parse_body());
        Stmt::Route { method, path, body }
    }

    fn parse_middleware(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected middleware name at pos {}, got {:?}", self.pos, t),
        };
        let mut before = None;
        let mut after  = None;
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            let block = self.parse_block();
            let mut iter = block.into_iter();
            if let Some(s) = iter.next() { before = Some(Box::new(s)); }
            if let Some(s) = iter.next() { after  = Some(Box::new(s)); }
        }
        Stmt::Middleware { name, config: HashMap::new(), before, after }
    }

    fn parse_apply(&mut self) -> Stmt {
        self.advance();
        let middleware = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected middleware name at pos {}, got {:?}", self.pos, t),
        };
        self.expect_keyword(Keyword::To);
        let mut paths = Vec::new();
        if self.match_keyword(&Keyword::All) {
            paths.push("*".to_string());
        } else {
            while !self.check(&Token::Symbol(Symbol::Semicolon)) && !self.is_at_end() {
                match self.peek().clone() {
                    Token::String(s) | Token::Ident(s) => { self.advance(); paths.push(s); }
                    _ => { self.advance(); }
                }
                self.match_token(&Token::Symbol(Symbol::Comma));
            }
        }
        self.skip_semi();
        Stmt::Apply { middleware, paths }
    }

    fn parse_tls_config(&mut self) -> Stmt {
        self.advance();
        Stmt::TlsConfig(self.parse_kv_block())
    }

    fn parse_auth(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected auth name at pos {}, got {:?}", self.pos, t),
        };
        let kind = if !self.check(&Token::Symbol(Symbol::LBrace)) {
            match self.peek().clone() {
                Token::Ident(s) => { self.advance(); s }
                _ => name.clone(),
            }
        } else { name.clone() };
        Stmt::Auth { name, kind, config: self.parse_kv_block() }
    }

    fn parse_security(&mut self) -> Stmt {
        self.advance();
        Stmt::Security(self.parse_kv_block())
    }

    fn parse_rate_limit(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected rate-limit name at pos {}, got {:?}", self.pos, t),
        };
        Stmt::RateLimit { name, config: self.parse_kv_block() }
    }

    fn parse_process_config(&mut self) -> Stmt {
        self.advance();
        Stmt::ProcessConfig(self.parse_kv_block())
    }

    fn parse_thread_pool(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected thread pool name at pos {}, got {:?}", self.pos, t),
        };
        Stmt::ThreadPool { name, config: self.parse_kv_block() }
    }

    fn parse_spawn(&mut self) -> Stmt {
        self.advance();
        let kind = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected spawn kind at pos {}, got {:?}", self.pos, t),
        };
        let body = Box::new(if self.check(&Token::Symbol(Symbol::LBrace)) {
            Stmt::Block(self.parse_block())
        } else { Stmt::Block(vec![]) });
        Stmt::Spawn { kind, config: HashMap::new(), body }
    }

    fn parse_upstream(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected upstream name at pos {}, got {:?}", self.pos, t),
        };
        let mut servers = Vec::new();
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            let block = self.parse_block();
            for stmt in block {
                if let Stmt::VarDecl { name: n, value: Some(Expr::Literal(Literal::String(s))), .. } = stmt {
                    servers.push((n, Expr::Literal(Literal::String(s))));
                }
            }
        }
        Stmt::Upstream { name, config: HashMap::new(), servers }
    }

    fn parse_proxy(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek().clone() {
            Token::String(s) => { self.advance(); s }
            t => panic!("Expected proxy path at pos {}, got {:?}", self.pos, t),
        };
        self.expect(Token::Symbol(Symbol::Arrow), "'->' for proxy target");
        let target = match self.peek().clone() {
            Token::Ident(s) | Token::String(s) => { self.advance(); s }
            t => panic!("Expected proxy target at pos {}, got {:?}", self.pos, t),
        };
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_kv_block()
        } else { self.skip_semi(); HashMap::new() };
        Stmt::Proxy { path, target, config }
    }

    fn parse_db(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected db name at pos {}, got {:?}", self.pos, t),
        };
        let kind = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            _ => "unknown".to_string(),
        };
        Stmt::Db { name, kind, config: self.parse_kv_block() }
    }

    fn parse_cache(&mut self) -> Stmt {
        self.advance();
        let name = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            t => panic!("Expected cache name at pos {}, got {:?}", self.pos, t),
        };
        let kind = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            _ => "unknown".to_string(),
        };
        Stmt::Cache { name, kind, config: self.parse_kv_block() }
    }

    // ── log ───────────────────────────────────
    //
    // Three forms:
    //   log { ... }           → LogConfig
    //   log info "message";   → LogStmt  (bare level ident)
    //   log.info("message");  → LogStmt  (method syntax)

    fn parse_log(&mut self) -> Stmt {
        self.advance(); // consume `log`

        // log { ... }
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            return Stmt::LogConfig(self.parse_kv_block());
        }

        // log.info(...)
        if self.match_token(&Token::Symbol(Symbol::Dot)) {
            let level = match self.peek().clone() {
                Token::Ident(s) => { self.advance(); s }
                t => panic!("Expected log level after '.' at pos {}, got {:?}", self.pos, t),
            };
            self.expect(Token::Symbol(Symbol::LParen), "'(' for log call");
            let msg = self.parse_expr();
            self.expect(Token::Symbol(Symbol::RParen), "')' to close log call");
            self.skip_semi();
            return Stmt::LogStmt { level, message: msg };
        }

        // log info "message";
        let level = match self.peek().clone() {
            Token::Ident(s)    => { self.advance(); s }
            Token::Keyword(kw) => { self.advance(); kw.to_string() }
            _                  => "info".to_string(),
        };
        let message = self.parse_expr();
        self.skip_semi();
        Stmt::LogStmt { level, message }
    }

    fn parse_health(&mut self) -> Stmt {
        self.advance();
        Stmt::Health(self.parse_kv_block())
    }

    fn parse_metrics(&mut self) -> Stmt {
        self.advance();
        let kind = match self.peek().clone() {
            Token::Ident(s) => { self.advance(); s }
            _ => "default".to_string(),
        };
        Stmt::Metrics { kind, config: self.parse_kv_block() }
    }

    fn parse_monitor(&mut self) -> Stmt {
        self.advance();
        Stmt::Monitor(self.parse_kv_block())
    }

    fn parse_static(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek().clone() {
            Token::String(s) => { self.advance(); s }
            t => panic!("Expected static path at pos {}, got {:?}", self.pos, t),
        };
        self.expect(Token::Symbol(Symbol::Arrow), "'->' for static root");
        let root = match self.peek().clone() {
            Token::String(s) => { self.advance(); s }
            t => panic!("Expected static root at pos {}, got {:?}", self.pos, t),
        };
        let config = if self.check(&Token::Symbol(Symbol::LBrace)) {
            self.parse_kv_block()
        } else { self.skip_semi(); HashMap::new() };
        Stmt::Static { path, root, config }
    }

    fn parse_websocket(&mut self) -> Stmt {
        self.advance();
        let path = match self.peek().clone() {
            Token::String(s) => { self.advance(); s }
            t => panic!("Expected websocket path at pos {}, got {:?}", self.pos, t),
        };
        let mut handlers = HashMap::new();
        if self.check(&Token::Symbol(Symbol::LBrace)) {
            let block = self.parse_block();
            handlers.insert("handler".to_string(), Box::new(Stmt::Block(block)));
        }
        Stmt::WebSocket { path, config: HashMap::new(), handlers }
    }

    fn parse_env_config(&mut self) -> Stmt {
        self.advance();
        Stmt::EnvConfig(self.parse_kv_block())
    }

    fn parse_config_file(&mut self) -> Stmt {
        self.advance();
        Stmt::ConfigFile(self.parse_kv_block())
    }

    fn parse_try_catch(&mut self) -> Stmt {
        self.advance();
        let try_block = Box::new(self.parse_body());
        let mut catches = Vec::new();
        while self.match_keyword(&Keyword::Catch) {
            let err_type = match self.peek().clone() {
                Token::Ident(s) => { self.advance(); s }
                _ => "Error".to_string(),
            };
            let err_var = match self.peek().clone() {
                Token::Ident(s) => { self.advance(); s }
                _ => "err".to_string(),
            };
            catches.push((err_type, err_var, Box::new(self.parse_body())));
        }
        let finally = if self.match_keyword(&Keyword::Finally) {
            Some(Box::new(self.parse_body()))
        } else { None };
        Stmt::TryCatch { try_block, catches, finally }
    }

    fn parse_on_panic(&mut self) -> Stmt {
        self.advance();
        Stmt::OnPanic { body: Box::new(self.parse_body()) }
    }

    // ── key-value config block ────────────────
    //
    // `{ key value; key: value; key [list]; ... }`
    // Keys: bare ident, quoted string, or keyword.
    // Values: any expression. Colon separator optional. Terminated by `;`.

fn parse_kv_block(&mut self) -> HashMap<String, Expr> {
        self.expect(Token::Symbol(Symbol::LBrace), "'{' to open config block");
        let mut config = HashMap::new();

        while !self.check(&Token::Symbol(Symbol::RBrace)) && !self.is_at_end() {
            if self.check(&Token::Symbol(Symbol::Semicolon)) {
                self.advance();
                continue;
            }

            let key = match self.peek().clone() {
                Token::Ident(s)    => { self.advance(); s }
                Token::String(s)   => { self.advance(); s }
                Token::Keyword(kw) => { self.advance(); kw.to_string() }
                _ => { self.advance(); continue; }
            };

            // optional colon
            self.match_token(&Token::Symbol(Symbol::Colon));

            let value = if self.check(&Token::Symbol(Symbol::LBracket)) {
                self.advance();
                self.parse_list_body()
            } else if self.can_start_expr() {
                self.parse_expr()
            } else {
                Expr::Ident(key.clone())
            };

            self.skip_semi();
            config.insert(key, value);
        }
        self.expect(Token::Symbol(Symbol::RBrace), "'}' to close config block");
        config
    }
}
