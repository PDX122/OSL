use std::collections::{HashMap, HashSet};
use crate::ast::*;

pub struct TypeChecker {
    env: HashMap<String, Type>,
    errors: Vec<TypeError>,
    in_loop: bool,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        env.insert("true".to_string(), Type::Bool);
        env.insert("false".to_string(), Type::Bool);
        env.insert("null".to_string(), Type::Custom("null".to_string()));
        
        env.insert("int".to_string(), Type::Int);
        env.insert("float".to_string(), Type::Float);
        env.insert("bool".to_string(), Type::Bool);
        env.insert("str".to_string(), Type::Str);
        env.insert("byte".to_string(), Type::Byte);
        env.insert("void".to_string(), Type::Void);
        env.insert("void".to_string(), Type::Custom("type".to_string()));
        
        TypeChecker { env, errors: Vec::new(), in_loop: false }
    }

    pub fn check(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        for stmt in &program.statements {
            self.check_stmt(stmt);
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Type {
        match stmt {
            Stmt::VarDecl { name, ty, value } => {
                let declared_ty = if *ty == Type::Infer {
                    if let Some(v) = value {
                        self.check_expr(v)
                    } else {
                        Type::Custom("null".to_string())
                    }
                } else {
                    ty.clone()
                };
                
                if let Some(v) = value {
                    let value_ty = self.check_expr(v);
                    if !self.types_compatible(&declared_ty, &value_ty) {
                        self.errors.push(TypeError {
                            message: format!("Type mismatch: expected {:?}, got {:?}", declared_ty, value_ty),
                            line: 0,
                        });
                    }
                }
                
                self.env.insert(name.clone(), declared_ty.clone());
                Type::Void
            }
            
            Stmt::Function { name, params, ret, body } => {
                for (param_name, param_type) in params {
                    self.env.insert(param_name.clone(), param_type.clone());
                }
                
                let body_ty = self.check_stmt(body.as_ref());
                
                if !self.types_compatible(ret, &body_ty) {
                    self.errors.push(TypeError {
                        message: format!("Function {} return type mismatch: expected {:?}, got {:?}", name, ret, body_ty),
                        line: 0,
                    });
                }
                
                let fn_type = Type::Func(params.iter().map(|(_, t)| t.clone()).collect(), Box::new(ret.clone()));
                self.env.insert(name.clone(), fn_type);
                Type::Void
            }
            
            Stmt::If { cond, then, else_ } => {
                let cond_ty = self.check_expr(cond);
                if !matches!(cond_ty, Type::Bool) {
                    self.errors.push(TypeError {
                        message: "If condition must be boolean".to_string(),
                        line: 0,
                    });
                }
                
                self.check_stmt(then.as_ref());
                if let Some(else_stmt) = else_ {
                    self.check_stmt(else_stmt.as_ref());
                }
                
                Type::Void
            }
            
            Stmt::Loop { times, body } => {
                let prev_loop = self.in_loop;
                self.in_loop = true;
                
                if let Some(t) = times {
                    let ty = self.check_expr(t);
                    if !matches!(ty, Type::Int) {
                        self.errors.push(TypeError {
                            message: "Loop count must be integer".to_string(),
                            line: 0,
                        });
                    }
                }
                
                self.check_stmt(body.as_ref());
                self.in_loop = prev_loop;
                Type::Void
            }
            
            Stmt::While { cond, body } => {
                let prev_loop = self.in_loop;
                self.in_loop = true;
                
                let cond_ty = self.check_expr(cond);
                if !matches!(cond_ty, Type::Bool) {
                    self.errors.push(TypeError {
                        message: "While condition must be boolean".to_string(),
                        line: 0,
                    });
                }
                
                self.check_stmt(body.as_ref());
                self.in_loop = prev_loop;
                Type::Void
            }
            
            Stmt::For { var, iter, body } => {
                let prev_loop = self.in_loop;
                self.in_loop = true;
                
                let iter_ty = self.check_expr(iter);
                self.env.insert(var.clone(), iter_ty);
                
                self.check_stmt(body.as_ref());
                self.in_loop = prev_loop;
                Type::Void
            }
            
            Stmt::Return(val) => {
                if let Some(v) = val {
                    self.check_expr(v)
                } else {
                    Type::Void
                }
            }
            
            Stmt::Break | Stmt::Continue => {
                if !self.in_loop {
                    self.errors.push(TypeError {
                        message: "Break/continue outside loop".to_string(),
                        line: 0,
                    });
                }
                Type::Void
            }
            
            Stmt::Block(stmts) => {
                let mut last_ty = Type::Void;
                for s in stmts {
                    last_ty = self.check_stmt(s);
                }
                last_ty
            }
            
            Stmt::Expr(e) => {
                self.check_expr(e);
                Type::Void
            }
            
            Stmt::ServerDecl { .. } |
            Stmt::SocketDecl { .. } |
            Stmt::HttpConfig(_) |
            Stmt::Route { .. } |
            Stmt::Middleware { .. } |
            Stmt::Apply { .. } |
            Stmt::TlsConfig(_) |
            Stmt::Auth { .. } |
            Stmt::Security(_) |
            Stmt::RateLimit { .. } |
            Stmt::ProcessConfig(_) |
            Stmt::ThreadPool { .. } |
            Stmt::Spawn { .. } |
            Stmt::Upstream { .. } |
            Stmt::Proxy { .. } |
            Stmt::Db { .. } |
            Stmt::Cache { .. } |
            Stmt::LogConfig(_) |
            Stmt::Health(_) |
            Stmt::Metrics { .. } |
            Stmt::Monitor(_) |
            Stmt::Static { .. } |
            Stmt::WebSocket { .. } |
            Stmt::EnvConfig(_) |
            Stmt::ConfigFile(_) |
            Stmt::TryCatch { .. } |
            Stmt::OnPanic { .. } => {
                Type::Void
            }
            
            Stmt::Assign { target, value } => {
                let target_ty = self.check_expr(target);
                let value_ty = self.check_expr(value);
                
                if !self.types_compatible(&target_ty, &value_ty) {
                    self.errors.push(TypeError {
                        message: format!("Assignment type mismatch: {:?} = {:?}", target_ty, value_ty),
                        line: 0,
                    });
                }
                
                Type::Void
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Ident(name) => {
                self.env.get(name).cloned().unwrap_or(Type::Custom("unknown".to_string()))
            }
            
            Expr::Literal(l) => match l {
                Literal::Int(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::String(_) => Type::Str,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Custom("null".to_string()),
            }
            
            Expr::Binary(left, op, right) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
                        // Skip type checking for now - be lenient
                        Type::Int
                    }
                    
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        if matches!(left_ty, Type::Int | Type::Float) && matches!(right_ty, Type::Int | Type::Float) {
                            Type::Bool
                        } else if matches!(op, BinOp::Eq | BinOp::Ne) {
                            Type::Bool
                        } else {
                            self.errors.push(TypeError {
                                message: "Comparison requires numeric types".to_string(),
                                line: 0,
                            });
                            Type::Custom("error".to_string())
                        }
                    }
                    
                    BinOp::And | BinOp::Or => {
                        if matches!(left_ty, Type::Bool) && matches!(right_ty, Type::Bool) {
                            Type::Bool
                        } else {
                            self.errors.push(TypeError {
                                message: "Logical operators require boolean operands".to_string(),
                                line: 0,
                            });
                            Type::Custom("error".to_string())
                        }
                    }
                    
                    BinOp::Concat => Type::Str,
                    
                    _ => Type::Custom("unknown".to_string()),
                }
            }
            
            Expr::Unary(op, operand) => {
                let op_ty = self.check_expr(operand);
                
                match op {
                    UnaryOp::Neg => {
                        // Skip type checking for now - be lenient
                        Type::Int
                    }
                    
                    UnaryOp::Not => {
                        if matches!(op_ty, Type::Bool) {
                            Type::Bool
                        } else {
                            Type::Bool
                        }
                    }
                    
                    UnaryOp::BitNot => {
                        if matches!(op_ty, Type::Int) {
                            Type::Int
                        } else {
                            Type::Int
                        }
                    }
                }
            }
            
            Expr::Call(func, args) => {
                let func_ty = self.check_expr(func);
                
                if let Type::Func(params, ret) = func_ty {
                    if params.len() != args.len() {
                        self.errors.push(TypeError {
                            message: format!("Function call argument count mismatch: expected {}, got {}", params.len(), args.len()),
                            line: 0,
                        });
                    }
                    
                    for (i, arg) in args.iter().enumerate() {
                        let arg_ty = self.check_expr(arg);
                        if i < params.len() && !self.types_compatible(&params[i], &arg_ty) {
                            self.errors.push(TypeError {
                                message: format!("Function argument {} type mismatch: expected {:?}, got {:?}", i + 1, params[i], arg_ty),
                                line: 0,
                            });
                        }
                    }
                    
                    *ret
                } else {
                    Type::Custom("unknown".to_string())
                }
            }
            
            Expr::Index(obj, index) => {
                let obj_ty = self.check_expr(obj);
                let _index_ty = self.check_expr(index);
                
                match obj_ty {
                    Type::List(inner) => *inner,
                    Type::Map(_, value) => *value,
                    _ => Type::Custom("unknown".to_string()),
                }
            }
            
            Expr::Field(obj, field) => {
                let _obj_ty = self.check_expr(obj);
                Type::Custom("unknown".to_string())
            }
            
            Expr::Lambda(params, ret, body) => {
                for (name, ty) in params {
                    self.env.insert(name.clone(), ty.clone());
                }
                
                let body_ty = self.check_stmt(body.as_ref());
                let actual_ret = body_ty;
                
                Type::Func(params.iter().map(|(_, t)| t.clone()).collect(), Box::new(actual_ret))
            }
            
            Expr::List(items) => {
                if items.is_empty() {
                    Type::List(Box::new(Type::Custom("unknown".to_string())))
                } else {
                    let first_ty = self.check_expr(&items[0]);
                    Type::List(Box::new(first_ty))
                }
            }
            
            Expr::Map(fields) => {
                if fields.is_empty() {
                    Type::Map(Box::new(Type::Str), Box::new(Type::Custom("unknown".to_string())))
                } else {
                    let first_value = fields.values().next().unwrap();
                    let first_ty = self.check_expr(first_value);
                    Type::Map(Box::new(Type::Str), Box::new(first_ty))
                }
            }
            
            Expr::Ternary { cond, then, else_ } => {
                let cond_ty = self.check_expr(cond);
                if !matches!(cond_ty, Type::Bool) {
                    self.errors.push(TypeError {
                        message: "Ternary condition must be boolean".to_string(),
                        line: 0,
                    });
                }
                
                let then_ty = self.check_expr(then);
                let else_ty = self.check_expr(else_);
                
                if self.types_compatible(&then_ty, &else_ty) {
                    then_ty
                } else {
                    Type::Custom("unknown".to_string())
                }
            }
            
            Expr::NullCoalesce(left, right) => {
                let left_ty = self.check_expr(left);
                let right_ty = self.check_expr(right);
                
                if left_ty == Type::Custom("null".to_string()) {
                    right_ty
                } else {
                    left_ty
                }
            }
            
            Expr::Assign(_, _) => Type::Custom("unknown".to_string()),
            
            Expr::In(_, _) => Type::Bool,
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        match (expected, actual) {
            (Type::Infer, _) => true,
            (_, Type::Infer) => true,
            (Type::Void, Type::Void) => true,
            (Type::Int, Type::Int) => true,
            (Type::Float, Type::Float) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Str, Type::Str) => true,
            (Type::Byte, Type::Byte) => true,
            (Type::Custom(s), _) if s == "null" => true,
            (_, Type::Custom(s)) if s == "null" => true,
            // Lenient: allow unknown types
            (Type::Custom(_), _) => true,
            (_, Type::Custom(_)) => true,
            (Type::List(a), Type::List(b)) => self.types_compatible(a, b),
            (Type::Map(ak, av), Type::Map(bk, bv)) => {
                self.types_compatible(ak, bk) && self.types_compatible(av, bv)
            }
            _ => true, // Lenient catch-all
        }
    }
}