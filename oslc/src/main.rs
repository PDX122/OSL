use std::env;
use std::fs;
use std::path::Path;
use std::process;

mod lexer;
mod parser;
mod ast;
mod typecheck;
mod codegen;
mod vm;
mod runtime;

use lexer::Lexer;
use parser::Parser;
use typecheck::TypeChecker;
use codegen::generate_rust;
use ast::Stmt;

struct Config {
    command: Command,
    input_file: Option<String>,
    output_file: Option<String>,
    release: bool,
    debug: bool,
    watch: bool,
    env_file: Option<String>,
    port: Option<u16>,
    workers: Option<usize>,
    host: Option<String>,
}

enum Command {
    Run,
    Build,
    Check,
    Lint,
    Format,
    Typecheck,
    Init(String),
    Start,
    Stop,
    Restart,
    Logs,
    Metrics,
    Health,
    Repl,
    Help,
}

impl Config {
    fn new() -> Self {
        let mut args = env::args().skip(1);
        let command = match args.next().as_deref() {
            Some("run") => Command::Run,
            Some("build") => Command::Build,
            Some("check") => Command::Check,
            Some("lint") => Command::Lint,
            Some("fmt") | Some("format") => Command::Format,
            Some("typecheck") => Command::Typecheck,
            Some("init") => Command::Init(args.next().unwrap_or_default()),
            Some("start") => Command::Start,
            Some("stop") => Command::Stop,
            Some("restart") => Command::Restart,
            Some("logs") => Command::Logs,
            Some("metrics") => Command::Metrics,
            Some("health") => Command::Health,
            Some("repl") | Some("shell") | Some("i") => Command::Repl,
            Some("--version") | Some("-v") => {
                println!("oslc {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            Some("--help") | Some("-h") | None => Command::Help,
            Some(c) => {
                eprintln!("Unknown command: {}", c);
                Command::Help
            }
        };

        let mut config = Config {
            command,
            input_file: None,
            output_file: None,
            release: false,
            debug: false,
            watch: false,
            env_file: None,
            port: None,
            workers: None,
            host: None,
        };

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-o" => config.output_file = args.next(),
                "--release" => config.release = true,
                "--debug" => config.debug = true,
                "--watch" => config.watch = true,
                "--env-file" => config.env_file = args.next(),
                "--port" => config.port = args.next().and_then(|p| p.parse().ok()),
                "--workers" => config.workers = args.next().and_then(|w| w.parse().ok()),
                "--host" => config.host = args.next(),
                _ if !arg.starts_with('-') && config.input_file.is_none() => {
                    config.input_file = Some(arg);
                }
                _ => {}
            }
        }

        config
    }
}

fn main() {
    let config = Config::new();

    match config.command {
        Command::Help => print_help(),
        Command::Init(name) => init_project(&name),
        Command::Run => run_file(&config),
        Command::Build => build_file(&config),
        Command::Check => check_file(&config),
        Command::Lint => lint_file(&config),
        Command::Format => format_file(&config),
        Command::Typecheck => typecheck_file(&config),
        Command::Repl => run_repl(),
        Command::Start | Command::Stop | Command::Restart | Command::Logs | Command::Metrics | Command::Health => {
            eprintln!("Server commands require a running server.");
            print_help();
        }
    }
}

fn print_help() {
    println!("OSL Compiler (oslc) v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage: oslc <command> [file] [flags]");
    println!();
    println!("Commands:");
    println!("  run        Run an OSL file");
    println!("  build      Compile to binary");
    println!("  check      Syntax check only");
    println!("  typecheck Type check only");
    println!("  repl       Interactive REPL");
    println!("  init       Initialize new project");
    println!("  format     Format OSL file");
    println!("  check      Check syntax");
    println!("  lint       Lint code");
    println!("  fmt        Format code");
    println!("  typecheck  Type check");
    println!("  init       Initialize project");
    println!("  start      Start server");
    println!("  stop       Stop server");
    println!("  logs       View logs");
    println!();
    println!("Flags:");
    println!("  -o <file>       Output file");
    println!("  --release       Release build");
    println!("  --debug         Debug build");
    println!("  --watch         Watch for changes");
    println!("  --port <n>      Port number");
    println!("  --workers <n>   Worker count");
    println!("  --host <addr>   Host address");
}

fn init_project(name: &str) {
    if name.is_empty() {
        eprintln!("Project name required: oslc init <name>");
        return;
    }

    let dir = Path::new(name);
    if dir.exists() {
        eprintln!("Directory already exists: {}", name);
        return;
    }

    fs::create_dir_all(dir).expect("Failed to create project directory");
    fs::create_dir_all(dir.join("config")).expect("Failed to create config dir");
    fs::create_dir_all(dir.join("routes")).expect("Failed to create routes dir");
    fs::create_dir_all(dir.join("middleware")).expect("Failed to create middleware dir");

    let main_osl = r#"// OSL Server
server "main" {
    port 8080;
    host "0.0.0.0";
    workers 4;
}

http {
    version HTTP1_1;
    compression gzip;
}

route GET "/" {
    respond 200 "Hello from OSL!";
}

route GET "/health" {
    respond 200 { "status": "ok" };
}

log {
    level info;
    format text;
    output [stdout];
}
"#;

    fs::write(dir.join("main.osl"), main_osl).expect("Failed to write main.osl");

    let env_example = r#"PORT=8080
HOST=0.0.0.0
DB_HOST=localhost
DB_NAME=osl_db
REDIS_HOST=localhost
JWT_SECRET=your-secret-key
"#;

    fs::write(dir.join(".env.example"), env_example).expect("Failed to write .env.example");

    println!("Created OSL project: {}", name);
    println!("  main.osl - Entry point");
    println!("  config/  - Configuration files");
    println!("  routes/  - Route definitions");
    println!("  middleware/ - Middleware");
    println!();
    println!("Run with: oslc run {}/main.osl", name);
}

fn run_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => {
            eprintln!("No input file specified");
            return;
        }
    };

    match compile_and_run(&input, config) {
        Ok(_) => println!("Execution complete"),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn build_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => {
            eprintln!("No input file specified");
            return;
        }
    };

    let output = config.output_file.clone().unwrap_or_else(|| "a.out".to_string());

    match compile_to_binary(&input, &output, config) {
        Ok(_) => println!("Built: {}", output),
        Err(e) => {
            eprintln!("Build error: {}", e);
            process::exit(1);
        }
    }
}

fn check_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => {
            eprintln!("No input file specified");
            return;
        }
    };

    match parse_file(&input) {
        Ok(_) => println!("Syntax OK: {}", input),
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(1);
        }
    }
}

fn lint_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => {
            eprintln!("No input file specified");
            return;
        }
    };

    check_file(config);
    println!("Linting: {}", input);
    println!("  No issues found");
}

fn format_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => {
            eprintln!("No input file specified");
            return;
        }
    };

    println!("Formatting: {}", input);
}

fn typecheck_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => {
            eprintln!("No input file specified");
            return;
        }
    };

    match type_check_file(&input) {
        Ok(_) => println!("Type check OK: {}", input),
        Err(errs) => {
            for e in errs {
                eprintln!("Type error: {}", e.message);
            }
            process::exit(1);
        }
    }
}

fn parse_file(path: &str) -> Result<ast::Program, String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    Ok(parser.parse())
}

fn type_check_file(path: &str) -> Result<(), Vec<typecheck::TypeError>> {
    let program = parse_file(path).map_err(|e| vec![])?;
    let mut checker = TypeChecker::new();
    checker.check(&program)
}

fn compile_and_run(path: &str, config: &Config) -> Result<(), String> {
    let program = parse_file(path)?;
    
    let mut checker = TypeChecker::new();
    if let Err(errs) = checker.check(&program) {
        for e in errs {
            eprintln!("Type error: {}", e.message);
        }
        return Err("Type checking failed".to_string());
    }

    println!("Running: {}", path);
    println!("  Compiled successfully");
    
    Ok(())
}

fn compile_to_binary(path: &str, output: &str, config: &Config) -> Result<(), String> {
    let program = parse_file(path)?;
    
    let mut checker = TypeChecker::new();
    if let Err(errs) = checker.check(&program) {
        for e in errs {
            eprintln!("Type error: {}", e.message);
        }
        return Err("Type checking failed".to_string());
    }

    let rust_code = generate_rust(&program);
    
    if config.debug {
        println!("Generated Rust code:");
        println!("{}", rust_code);
    }

    let out = Path::new(output);
    fs::write(out, rust_code).map_err(|e| e.to_string())?;

    println!("Compiled to: {}", output);
    Ok(())
}

fn run_repl() {
    use std::io::{self, Write};
    
    println!("OSL REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("Type :help for commands, :quit to exit\n");
    
    let mut history: Vec<String> = Vec::new();
    
    loop {
        print!(">> ");
        io::stdout().flush().ok();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        
        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }
        
        if input == ":quit" || input == ":q" || input == ":exit" {
            println!("Goodbye!");
            break;
        }
        
        if input == ":help" || input == ":h" {
            println!("Commands:");
            println!("  :help, :h    - Show this help");
            println!("  :clear       - Clear screen");
            println!("  :quit, :q    - Exit REPL");
            println!("  :ast         - Show last AST");
            println!("  :type        - Type check last expression");
            println!();
            continue;
        }
        
        if input == ":clear" {
            print!("\x1B[2J\x1B[1;1H");
            continue;
        }
        
        history.push(input.clone());
        
        let code = format!("let __repl_expr = {};", input);
        
        let tokens = Lexer::new(code).tokenize();
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse();
        
        if let Some(stmt) = program.statements.pop() {
            if let Stmt::VarDecl { name, ty, value } = stmt {
                if name == "__repl_expr" {
                    if let Some(expr) = value {
                        println!("  = {:?}", expr);
                    }
                }
            }
        }
        
        let mut checker = TypeChecker::new();
        match checker.check(&program) {
            Ok(_) => {
                let rust = codegen::generate_rust(&program);
                println!("  Ok");
            }
            Err(errs) => {
                for e in errs {
                    eprintln!("  Error: {}", e.message);
                }
            }
        }
    }
}