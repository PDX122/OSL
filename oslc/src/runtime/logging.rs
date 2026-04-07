use std::collections::HashMap;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Logger {
    config: LogConfig,
    output: Vec<Box<dyn Write + Send>>,
}

#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: Vec<LogOutput>,
    pub rotate: Option<LogRotate>,
    pub include_request_id: bool,
    pub include_timestamp: bool,
    pub include_caller: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    Json,
    Text,
    Pretty,
}

#[derive(Debug, Clone)]
pub enum LogOutput {
    Stdout,
    Stderr,
    File(String),
}

#[derive(Debug, Clone)]
pub struct LogRotate {
    pub max_size: u64,
    pub max_files: usize,
    pub compress: bool,
}

impl Logger {
    pub fn new(config: LogConfig) -> Logger {
        Logger {
            config,
            output: Vec::new(),
        }
    }

    pub fn trace(&mut self, msg: &str) {
        if self.config.level <= LogLevel::Trace {
            self.log(LogLevel::Trace, msg);
        }
    }

    pub fn debug(&mut self, msg: &str) {
        if self.config.level <= LogLevel::Debug {
            self.log(LogLevel::Debug, msg);
        }
    }

    pub fn info(&mut self, msg: &str) {
        if self.config.level <= LogLevel::Info {
            self.log(LogLevel::Info, msg);
        }
    }

    pub fn warn(&mut self, msg: &str) {
        if self.config.level <= LogLevel::Warn {
            self.log(LogLevel::Warn, msg);
        }
    }

    pub fn error(&mut self, msg: &str) {
        if self.config.level <= LogLevel::Error {
            self.log(LogLevel::Error, msg);
        }
    }

    pub fn fatal(&mut self, msg: &str) {
        self.log(LogLevel::Fatal, msg);
    }

    fn log(&mut self, level: LogLevel, msg: &str) {
        let line = match self.config.format {
            LogFormat::Json => self.format_json(level, msg),
            LogFormat::Text | LogFormat::Pretty => self.format_text(level, msg),
        };
        
        for out in &mut self.output {
            let _ = out.write_all(line.as_bytes());
        }
    }

    fn format_json(&self, level: LogLevel, msg: &str) -> String {
        let level_str = match level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Fatal => "fatal",
        };
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        format!(r#"{{"level":"{}","message":"{}","timestamp":{}}}"#, 
            level_str, msg, timestamp)
    }

    fn format_text(&self, level: LogLevel, msg: &str) -> String {
        let level_str = match level {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        };
        
        if self.config.include_timestamp {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("[{}] {}: {}\n", timestamp, level_str, msg)
        } else {
            format!("{}: {}\n", level_str, msg)
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: LogLevel::Info,
            format: LogFormat::Text,
            output: vec![LogOutput::Stdout],
            rotate: None,
            include_request_id: false,
            include_timestamp: true,
            include_caller: false,
        }
    }
}

pub fn init_logging(config: LogConfig) -> Logger {
    Logger::new(config)
}