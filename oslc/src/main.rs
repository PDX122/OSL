use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Instant;

mod lexer;
mod parser;
mod ast;
mod typecheck;
mod codegen;
mod vm;
mod runtime;
mod package_manager;

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
    Lex,
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
    Install(String),
    Uninstall(String),
    List,
    Search(String),
    Help,
}

impl Config {
    fn new() -> Self {
        let mut args = env::args().skip(1);
        let command = match args.next().as_deref() {
            Some("run")                          => Command::Run,
            Some("build")                        => Command::Build,
            Some("check")                        => Command::Check,
            Some("lex")                          => Command::Lex,
            Some("lint")                         => Command::Lint,
            Some("fmt") | Some("format")         => Command::Format,
            Some("typecheck")                    => Command::Typecheck,
            Some("init")                         => Command::Init(args.next().unwrap_or_default()),
            Some("start")                        => Command::Start,
            Some("stop")                         => Command::Stop,
            Some("restart")                      => Command::Restart,
            Some("logs")                         => Command::Logs,
            Some("metrics")                      => Command::Metrics,
            Some("health")                       => Command::Health,
            Some("repl") | Some("shell") | Some("i") => Command::Repl,
            Some("install")                      => Command::Install(args.next().unwrap_or_default()),
            Some("uninstall") | Some("remove") | Some("rm") => Command::Uninstall(args.next().unwrap_or_default()),
            Some("list") | Some("ls")            => Command::List,
            Some("search")                        => Command::Search(args.next().unwrap_or_default()),
            Some("--version") | Some("-v")       => {
                println!("oslc {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            Some("--help") | Some("-h") | None   => Command::Help,
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
                "-o"         => config.output_file = args.next(),
                "--release"  => config.release = true,
                "--debug"    => config.debug = true,
                "--watch"    => config.watch = true,
                "--env-file" => config.env_file = args.next(),
                "--port"     => config.port    = args.next().and_then(|p| p.parse().ok()),
                "--workers"  => config.workers = args.next().and_then(|w| w.parse().ok()),
                "--host"     => config.host    = args.next(),
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
        Command::Help    => print_help(),
        Command::Init(ref name) => init_project(name),
        Command::Run     => run_file(&config),
        Command::Build   => build_file(&config),
        Command::Check   => check_file(&config),
        Command::Lex     => lex_file(&config),
        Command::Lint    => lint_file(&config),
        Command::Format  => format_file(&config),
        Command::Typecheck => typecheck_file(&config),
        Command::Repl    => run_repl(),
        Command::Install(ref spec) => install_package(spec),
        Command::Uninstall(ref spec) => uninstall_package(spec),
        Command::List    => list_packages(),
        Command::Search(ref query) => search_packages(query),
        Command::Start | Command::Stop | Command::Restart
        | Command::Logs | Command::Metrics | Command::Health => {
            eprintln!("Server commands require a running server.");
            print_help();
        }
    }
}

fn print_help() {
    use package_manager::{ANSI_GREEN, ANSI_CYAN, ANSI_YELLOW, ANSI_RESET};
    println!("{}OSL Compiler{} (oslc) v{}", ANSI_GREEN, ANSI_RESET, env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage: {}oslc{} <command> [file] [flags]", ANSI_CYAN, ANSI_RESET);
    println!();
    println!("Commands:");
    println!("  {}run{}        Run an OSL file", ANSI_CYAN, ANSI_RESET);
    println!("  {}build{}      Compile to binary", ANSI_CYAN, ANSI_RESET);
    println!("  {}check{}      Syntax check only", ANSI_CYAN, ANSI_RESET);
    println!("  {}typecheck{}  Type check only", ANSI_CYAN, ANSI_RESET);
    println!("  {}repl{}       Interactive REPL", ANSI_CYAN, ANSI_RESET);
    println!("  {}init{}       Initialize new project", ANSI_CYAN, ANSI_RESET);
    println!("  {}format{}     Format OSL file", ANSI_CYAN, ANSI_RESET);
    println!("  {}lint{}       Lint code", ANSI_CYAN, ANSI_RESET);
    println!("  {}install{}    Install a package", ANSI_CYAN, ANSI_RESET);
    println!("  {}uninstall{}  Uninstall a package", ANSI_CYAN, ANSI_RESET);
    println!("  {}list{}       List installed packages", ANSI_CYAN, ANSI_RESET);
    println!("  {}search{}      Search registry packages", ANSI_CYAN, ANSI_RESET);
    println!();
    println!("Server:");
    println!("  {}start{}      Start server", ANSI_CYAN, ANSI_RESET);
    println!("  {}stop{}       Stop server", ANSI_CYAN, ANSI_RESET);
    println!("  {}restart{}    Restart server", ANSI_CYAN, ANSI_RESET);
    println!("  {}logs{}       View logs", ANSI_CYAN, ANSI_RESET);
    println!();
    println!("Flags:");
    println!("  {}--release{}    Release build", ANSI_YELLOW, ANSI_RESET);
    println!("  {}--debug{}      Debug build / print generated code", ANSI_YELLOW, ANSI_RESET);
    println!("  {}--watch{}      Watch for changes", ANSI_YELLOW, ANSI_RESET);
    println!("  {}--port <n>{}   Port number", ANSI_YELLOW, ANSI_RESET);
    println!("  {}--host <addr>{}    Host address", ANSI_YELLOW, ANSI_RESET);
    println!();
    println!("Examples:");
    println!("  {}oslc run{} myapp.osl", ANSI_CYAN, ANSI_RESET);
    println!("  {}oslc install{} discord/tester", ANSI_CYAN, ANSI_RESET);
    println!("  {}oslc search{} http", ANSI_CYAN, ANSI_RESET);
    println!("  {}oslc list{}", ANSI_CYAN, ANSI_RESET);
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
println!("  main.osl      - Entry point");
println!("  config/       - Configuration files");
println!("  routes/       - Route definitions");
println!("  middleware/   - Middleware");
println!();
println!("Run with: oslc run {}/main.osl", name);
}

// ── run ───────────────────────────────────────────────────────────────────────

fn run_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => {
            let mut f = f.clone();
            if !f.contains('.') {
                f.push_str(".osl");
            }
            f
        }
        None => { eprintln!("No input file specified"); return; }
    };

    let start = Instant::now();

    match compile_and_run(&input, config) {
        Ok(exit_code) => {
            let elapsed = start.elapsed();
            eprintln!();
            eprintln!("─────────────────────────────────────");
            if exit_code == 0 {
                eprintln!("✓ Exited successfully  ({:.0}ms)", elapsed.as_millis());
            } else {
                eprintln!("✗ Exited with code {}  ({:.0}ms)", exit_code, elapsed.as_millis());
                process::exit(exit_code);
            }
        }
        Err(e) => {
            eprintln!();
            eprintln!("─────────────────────────────────────");
            eprintln!("✗ Error: {}", e);
            process::exit(1);
        }
    }
}

// ── build ─────────────────────────────────────────────────────────────────────

fn build_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => { eprintln!("No input file specified"); return; }
    };

    let output = config.output_file.clone().unwrap_or_else(|| "a.out".to_string());
    let start = Instant::now();

    match compile_to_binary(&input, &output, config) {
        Ok(_) => {
            let elapsed = start.elapsed();
            println!("✓ Built: {}  ({:.0}ms)", output, elapsed.as_millis());
        }
        Err(e) => {
            eprintln!("✗ Build error: {}", e);
            process::exit(1);
        }
    }
}

// ── check / lint / format / typecheck ─────────────────────────────────────────

fn check_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => { eprintln!("No input file specified"); return; }
    };

    match parse_file(&input) {
        Ok(program) => {
            println!("✓ Syntax OK: {}  ({} statements)", input, program.statements.len());
        }
        Err(e) => {
            eprintln!("✗ Parse error in {}: {}", input, e);
            process::exit(1);
        }
    }
}

fn lex_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => { eprintln!("No input file specified"); return; }
    };

    match fs::read_to_string(&input) {
        Ok(source) => {
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize();
            for (i, token) in tokens.iter().enumerate() {
                println!("{:3}: {:?}", i, token);
            }
        }
        Err(e) => {
            eprintln!("Could not read '{}': {}", input, e);
            process::exit(1);
        }
    }
}

fn lint_file(config: &Config) {
    check_file(config);
    if let Some(f) = &config.input_file {
        println!("✓ Lint OK: {}", f);
    }
}

fn format_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => { eprintln!("No input file specified"); return; }
    };
    println!("Formatting: {} (not yet implemented)", input);
}

fn typecheck_file(config: &Config) {
    let input = match &config.input_file {
        Some(f) => f.clone(),
        None => { eprintln!("No input file specified"); return; }
    };

    match type_check_file(&input) {
        Ok(_)     => println!("✓ Type check OK: {}", input),
        Err(errs) => {
            for e in &errs {
                eprintln!("✗ Type error: {}", e.message);
            }
            process::exit(1);
        }
    }
}

// ── core helpers ──────────────────────────────────────────────────────────────

fn parse_file(path: &str) -> Result<ast::Program, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("Could not read '{}': {}", path, e))?;
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

/// Compile and run, returning the process exit code.
fn compile_and_run(path: &str, config: &Config) -> Result<i32, String> {
    // ── 1. parse ──────────────────────────────────────────────────────────────
    let source = fs::read_to_string(path)
    .map_err(|e| format!("Could not read '{}': {}", path, e))?;

    eprint!("  Parsing...     ");
    let tokens = Lexer::new(source.clone()).tokenize();
    let program = Parser::new(tokens).parse();
    eprintln!("✓  ({} statements)", program.statements.len());

    // ── 2. typecheck ──────────────────────────────────────────────────────────
    eprint!("  Typechecking...");
    let mut checker = TypeChecker::new();
    if let Err(errs) = checker.check(&program) {
        eprintln!();
        for e in errs {
            eprintln!("  ✗ type error: {}", e.message);
        }
        return Err("Type checking failed".to_string());
    }
    eprintln!("✓");

    // ── 3. codegen ────────────────────────────────────────────────────────────
    eprint!("  Codegen...     ");
    let rust_code = generate_rust(&program);
    eprintln!("✓");

    if config.debug {
        eprintln!();
        eprintln!("── Generated Rust ────────────────────");
        eprintln!("{}", rust_code);
        eprintln!("──────────────────────────────────────");
        eprintln!();
    }

    // ── 4. write to temp dir ──────────────────────────────────────────────────
    let tmp = tempdir_path();
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;

    let src_path   = tmp.join("main.rs");
    let bin_path   = tmp.join("out");

    fs::write(&src_path, &rust_code).map_err(|e| e.to_string())?;

    // ── 5. compile with rustc ─────────────────────────────────────────────────
    eprint!("  Compiling...   ");
    let mut cmd = process::Command::new("rustc");
    cmd.arg(&src_path)
       .arg("-o").arg(&bin_path);
    if config.release {
        cmd.arg("-O");
    }
    let rustc = cmd.output()
    .map_err(|e| format!("rustc not found: {}", e))?;

    if !rustc.status.success() {
        eprintln!("✗");
        eprintln!();
        eprintln!("── rustc stderr ──────────────────────");
        eprintln!("{}", String::from_utf8_lossy(&rustc.stderr));
        eprintln!("──────────────────────────────────────");
        return Err("Compilation failed".to_string());
    }
    eprintln!("✓");

    // ── 6. run ────────────────────────────────────────────────────────────────
    eprintln!();
    eprintln!("── Output ────────────────────────────");

    let status = process::Command::new(&bin_path)
    .status()
    .map_err(|e| format!("Failed to run binary: {}", e))?;

    eprintln!("──────────────────────────────────────");

    Ok(status.code().unwrap_or(1))
}

fn compile_to_binary(path: &str, output: &str, config: &Config) -> Result<(), String> {
    let source = fs::read_to_string(path)
    .map_err(|e| format!("Could not read '{}': {}", path, e))?;

    let tokens  = Lexer::new(source).tokenize();
    let program = Parser::new(tokens).parse();

    let mut checker = TypeChecker::new();
    if let Err(errs) = checker.check(&program) {
        for e in errs { eprintln!("✗ type error: {}", e.message); }
        return Err("Type checking failed".to_string());
    }

    let rust_code = generate_rust(&program);

    if config.debug {
        println!("── Generated Rust ────────────────────");
        println!("{}", rust_code);
        println!("──────────────────────────────────────");
    }

    let tmp      = tempdir_path();
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let src_path = tmp.join("main.rs");
    fs::write(&src_path, &rust_code).map_err(|e| e.to_string())?;

    let mut cmd = process::Command::new("rustc");
    cmd.arg(&src_path)
       .arg("-o").arg(output);
    if config.release {
        cmd.arg("-O");
    }
    let rustc = cmd.output()
    .map_err(|e| format!("rustc not found: {}", e))?;

    if !rustc.status.success() {
        eprintln!("── rustc stderr ──────────────────────");
        eprintln!("{}", String::from_utf8_lossy(&rustc.stderr));
        return Err("Compilation failed".to_string());
    }

    Ok(())
}

/// Return a per-run temp directory path (no external crates needed).
fn tempdir_path() -> std::path::PathBuf {
    let pid = process::id();
    std::env::temp_dir().join(format!("oslc-{}", pid))
}

// ── REPL ──────────────────────────────────────────────────────────────────────

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
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let input = input.trim().to_string();
        if input.is_empty() { continue; }

        match input.as_str() {
            ":quit" | ":q" | ":exit" => { println!("Goodbye!"); break; }
            ":help" | ":h" => {
                println!("Commands:");
                println!("  :help, :h    Show this help");
                println!("  :clear       Clear screen");
                println!("  :quit, :q    Exit REPL");
                println!("  :ast         Show last AST");
                println!("  :type        Type check last expression");
                println!();
                continue;
            }
            ":clear" => { print!("\x1B[2J\x1B[1;1H"); continue; }
            _ => {}
        }

        history.push(input.clone());

        let code = format!("let __repl_expr = {};", input);
        let tokens = Lexer::new(code).tokenize();
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse();

        if let Some(stmt) = program.statements.pop() {
            if let Stmt::VarDecl { name, value: Some(expr), .. } = stmt {
                if name == "__repl_expr" {
                    println!("  = {:?}", expr);
                }
            }
        }

        let mut checker = TypeChecker::new();
        match checker.check(&program) {
            Ok(_) => {
                let _rust = codegen::generate_rust(&program);
                println!("  ok");
            }
            Err(errs) => {
                for e in errs { eprintln!("  error: {}", e.message); }
            }
        }
    }
}

fn install_package(spec: &str) {
    use package_manager::{ANSI_GREEN, ANSI_RED, ANSI_YELLOW, ANSI_RESET, confirm_install, is_installed};
    
    if spec.is_empty() {
        eprintln!("{}Package required:{} oslc install <owner/name>", ANSI_RED, ANSI_RESET);
        return;
    }

    if is_installed(spec) {
        eprintln!("{} is already installed", spec);
        eprintln!("  Use {}oslc uninstall {}{} to remove it first", ANSI_YELLOW, spec, ANSI_RESET);
        return;
    }

    if !confirm_install(spec) {
        println!("Aborted.");
        return;
    }

    match package_manager::install_from_registry(spec) {
        Ok(result) => {
            println!("{}✓ Installed{} {}", ANSI_GREEN, ANSI_RESET, result.spec);
            println!("  location: {}", result.install_dir.display());
        }
        Err(err) => {
            eprintln!("{}✗ Install error:{} {}", ANSI_RED, ANSI_RESET, err);
            process::exit(1);
        }
    }
}

fn uninstall_package(spec: &str) {
    use package_manager::{ANSI_GREEN, ANSI_RED, ANSI_RESET};
    
    if spec.is_empty() {
        eprintln!("{}Package required:{} oslc uninstall <owner/name>", ANSI_RED, ANSI_RESET);
        return;
    }

    match package_manager::uninstall_package(spec) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}✗ Error:{} {}", ANSI_RED, ANSI_RESET, err);
            process::exit(1);
        }
    }
}

fn list_packages() {
    use package_manager::{ANSI_GREEN, ANSI_CYAN, ANSI_RED, ANSI_RESET};
    
    match package_manager::list_installed_packages() {
        Ok(packages) => {
            if packages.is_empty() {
                println!("No packages installed.");
                println!("  Use {}oslc install <pkg>{} to install a package", ANSI_CYAN, ANSI_RESET);
            } else {
                let count = packages.len();
                println!("{}Installed packages:{}", ANSI_GREEN, ANSI_RESET);
                for pkg in &packages {
                    println!("  • {}", pkg.name);
                }
                println!("\n{} total", count);
            }
        }
        Err(err) => {
            eprintln!("{}✗ Error:{} {}", ANSI_RED, ANSI_RESET, err);
            process::exit(1);
        }
    }
}

fn search_packages(query: &str) {
    use package_manager::{ANSI_GREEN, ANSI_CYAN, ANSI_YELLOW, ANSI_RED, ANSI_RESET};
    
    if query.is_empty() {
        eprintln!("{}Search query required:{} oslc search <term>", ANSI_RED, ANSI_RESET);
        return;
    }

    print!("Searching for {}... ", query);
    io::Write::flush(&mut io::stdout()).ok();

    match package_manager::search_packages(query) {
        Ok(results) => {
            println!("done");
            if results.is_empty() {
                println!("No packages found for {}", query);
            } else {
                println!("\n{}Found packages:{}", ANSI_GREEN, ANSI_RESET);
                for pkg in results {
                    println!("  • {}", pkg);
                }
                println!("\nInstall with: {}oslc install <pkg>{}", ANSI_CYAN, ANSI_RESET);
            }
        }
        Err(err) => {
            println!("failed");
            eprintln!("{}✗ Search error:{} {}", ANSI_RED, ANSI_RESET, err);
            process::exit(1);
        }
    }
}
