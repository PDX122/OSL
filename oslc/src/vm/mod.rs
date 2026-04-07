use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Function(String),
    Null,
    VmInstruction(Instruction),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Int(n) => n.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::List(items) => format!("[{}]", items.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            Value::Map(m) => format!("{{{}}}", m.iter().map(|(k, v)| format!("{}: {}", k, v.to_string())).collect::<Vec<_>>().join(", ")),
            Value::Function(name) => format!("fn {}", name),
            Value::Null => "null".to_string(),
            Value::VmInstruction(_) => "instruction".to_string(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::String(_) => "str",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Function(_) => "fn",
            Value::Null => "null",
            Value::VmInstruction(_) => "vm_instruction",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(usize),
    LoadLocal(String),
    StoreLocal(String),
    LoadGlobal(String),
    StoreGlobal(String),
    Call(String, usize),
    Return,
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    LoopStart,
    LoopEnd,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Neg,
    Not,
    Concat,
    BuildList(usize),
    BuildMap(usize),
    Index,
    Field(String),
    MakeFunction(String),
}

#[derive(Debug, Clone)]
pub struct VmFunction {
    pub name: String,
    pub params: Vec<String>,
    pub instructions: Vec<Instruction>,
}

pub struct Vm {
    stack: Vec<Value>,
    locals: HashMap<String, Value>,
    globals: HashMap<String, Value>,
    constants: Vec<Value>,
    functions: HashMap<String, VmFunction>,
    call_stack: Vec<usize>,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            stack: Vec::new(),
            locals: HashMap::new(),
            globals: HashMap::new(),
            constants: Vec::new(),
            functions: HashMap::new(),
            call_stack: Vec::new(),
            ip: 0,
        }
    }

    pub fn load_function(&mut self, name: String, params: Vec<String>, instructions: Vec<Instruction>) {
        self.functions.insert(name.clone(), VmFunction { name, params, instructions });
    }

    pub fn run(&mut self) -> Result<Value, VmError> {
        loop {
            if self.ip >= 1000 {
                return Err(VmError::ExecutionLimit);
            }
            self.ip += 1;
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    pub fn peek(&self) -> Option<&Value> {
        self.stack.last()
    }

    pub fn set_local(&mut self, name: String, value: Value) {
        self.locals.insert(name, value);
    }

    pub fn get_local(&self, name: &str) -> Option<&Value> {
        self.locals.get(name)
    }

    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        let idx = self.constants.len();
        self.constants.push(value);
        idx
    }
}

#[derive(Debug, Clone)]
pub enum VmError {
    StackUnderflow,
    UnknownInstruction,
    UndefinedVariable(String),
    TypeError(String),
    ExecutionLimit,
    DivisionByZero,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VmError::StackUnderflow => write!(f, "Stack underflow"),
            VmError::UnknownInstruction => write!(f, "Unknown instruction"),
            VmError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            VmError::TypeError(msg) => write!(f, "Type error: {}", msg),
            VmError::ExecutionLimit => write!(f, "Execution limit exceeded"),
            VmError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

pub fn execute(program: &super::ast::Program) -> Result<Value, VmError> {
    let mut vm = Vm::new();
    
    vm.set_global("print".to_string(), Value::Function("print".to_string()));
    vm.set_global("len".to_string(), Value::Function("len".to_string()));
    vm.set_global("now".to_string(), Value::Function("now".to_string()));
    
    vm.run()
}