use chrono::Local;
use log::{Level, LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::env::consts::{ARCH, OS};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

static SESSION_LOG_PATH: OnceLock<PathBuf> = OnceLock::new();

struct SessionLogger;

static LOGGER: SessionLogger = SessionLogger;

impl Log for SessionLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let line = format!("[{}] {}: {}", timestamp, record.level(), record.args());

        match record.level() {
            Level::Error | Level::Warn => eprintln!("{}", line),
            _ => println!("{}", line),
        }

        append_session_log_line(&line);
    }

    fn flush(&self) {}
}

pub fn initialize_session_logging() -> Result<(), String> {
    let log_path = get_session_log_path()?;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .map_err(|error| error.to_string())?;

    writeln!(
        file,
        "[{}] SESSION START | app=rusty-g6 version={} platform={}/{}",
        Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        env!("CARGO_PKG_VERSION"),
        OS,
        ARCH
    )
    .map_err(|error| error.to_string())?;

    let _ = SESSION_LOG_PATH.set(log_path);
    Ok(())
}

pub fn initialize_app_logger() -> Result<(), String> {
    set_logger(&LOGGER).map_err(|error| error.to_string())?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

pub fn resolve_session_log_path() -> Result<PathBuf, String> {
    SESSION_LOG_PATH
        .get()
        .cloned()
        .or_else(|| get_session_log_path().ok())
        .ok_or_else(|| "Session log path unavailable".to_string())
}

pub fn get_session_log_text() -> Result<String, String> {
    let log_path = resolve_session_log_path()?;
    std::fs::read_to_string(log_path).map_err(|error| error.to_string())
}

fn append_session_log_line(line: &str) {
    if let Ok(log_path) = resolve_session_log_path() {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
            let _ = writeln!(file, "{}", line);
        }
    }
}

fn get_session_log_path() -> Result<PathBuf, String> {
    let debug_dir = dirs::config_dir()
        .ok_or_else(|| "Could not find config directory".to_string())?
        .join("rusty-g6")
        .join("debug");
    fs::create_dir_all(&debug_dir).map_err(|error| error.to_string())?;
    Ok(debug_dir.join("session.log"))
}

fn set_logger(logger: &'static dyn Log) -> Result<(), SetLoggerError> {
    log::set_logger(logger)
}
