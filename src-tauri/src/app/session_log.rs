//! Logging (App Standard section 3).
//!
//! `debug/session.log` is this run's log. At start-up the previous run's log
//! becomes `last-session.log`, so the run before a crash or restart is kept.
//! Every session starts with a header describing the build and the machine
//! (`SystemInfo`), and every line carries an RFC 3339 local timestamp with
//! milliseconds and offset.

use crate::app::system_info::SystemInfo;
use chrono::{Local, SecondsFormat};
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;

const SESSION_LOG_FILE_NAME: &str = "session.log";
const LAST_SESSION_LOG_FILE_NAME: &str = "last-session.log";

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

        let line = format!(
            "[{}] {}: {}",
            log_timestamp(),
            record.level(),
            record.args()
        );

        match record.level() {
            Level::Error | Level::Warn => eprintln!("{}", line),
            _ => println!("{}", line),
        }

        append_session_log_line(&line);
    }

    fn flush(&self) {}
}

/// RFC 3339 local time with milliseconds and offset,
/// e.g. `2026-11-03T19:55:39.123+11:00`.
fn log_timestamp() -> String {
    Local::now().to_rfc3339_opts(SecondsFormat::Millis, false)
}

/// Starts this run's session log: rotates the previous one to
/// `last-session.log` and writes the header. Runs after the single-instance
/// guard, so a second launch never rotates the running instance's log.
pub fn initialize_session_logging() -> Result<(), String> {
    let debug_dir = crate::paths::debug_dir()?;
    let session_log_path = debug_dir.join(SESSION_LOG_FILE_NAME);

    if session_log_path.exists() {
        let last_session_path = debug_dir.join(LAST_SESSION_LOG_FILE_NAME);
        if let Err(error) = fs::rename(&session_log_path, &last_session_path) {
            eprintln!("Failed to keep the previous session log: {error}");
        }
    }

    let _ = SESSION_LOG_PATH.set(session_log_path);

    let timestamp = log_timestamp();
    append_session_log_line(&format!("[{timestamp}] INFO: SESSION START"));
    for header_line in SystemInfo::collect().summary().lines() {
        append_session_log_line(&format!("[{timestamp}] INFO: {header_line}"));
    }

    Ok(())
}

/// The log levels offered in Settings -> Debug, most to least severe.
pub const LOG_LEVELS: [&str; 5] = ["error", "warn", "info", "debug", "trace"];

/// Installs the `log` backend at `info`; `apply_log_level` switches to the
/// saved level once the settings are loaded.
pub fn initialize_app_logger() -> Result<(), String> {
    log::set_logger(&LOGGER).map_err(|error| error.to_string())?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

/// Applies a saved level; an unknown one leaves the current level.
pub fn apply_log_level(level: &str) {
    match parse_log_level(level) {
        Some(filter) => log::set_max_level(filter),
        None => log::warn!("Unknown log level '{}' in settings; keeping info", level),
    }
}

pub fn parse_log_level(level: &str) -> Option<LevelFilter> {
    LOG_LEVELS
        .contains(&level)
        .then(|| level.parse::<LevelFilter>().ok())
        .flatten()
}

/// Records every panic in the session log (then runs the default hook), so an
/// unexpected exit always leaves its reason and location behind. Installed
/// first thing in `main`.
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|message| message.to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-text panic payload".to_string());
        let location = info
            .location()
            .map(|location| format!("{}:{}", location.file(), location.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        let thread = std::thread::current();
        append_session_log_line(&format!(
            "[{}] PANIC: thread '{}' panicked at {}: {}",
            log_timestamp(),
            thread.name().unwrap_or("unnamed"),
            location,
            message
        ));
        default_hook(info);
    }));
}

pub fn resolve_session_log_path() -> Result<PathBuf, String> {
    match SESSION_LOG_PATH.get() {
        Some(path) => Ok(path.clone()),
        None => Ok(crate::paths::debug_dir()?.join(SESSION_LOG_FILE_NAME)),
    }
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

#[cfg(test)]
mod tests {
    use super::{log_timestamp, parse_log_level};
    use log::LevelFilter;

    #[test]
    fn only_offered_log_levels_parse() {
        assert_eq!(parse_log_level("debug"), Some(LevelFilter::Debug));
        assert_eq!(parse_log_level("off"), None);
        assert_eq!(parse_log_level("loud"), None);
    }

    #[test]
    fn timestamps_are_rfc3339_with_millis_and_offset() {
        let timestamp = log_timestamp();
        assert!(chrono::DateTime::parse_from_rfc3339(&timestamp).is_ok());
        // Milliseconds and a numeric offset, e.g. 2026-11-03T19:55:39.123+11:00.
        assert_eq!(timestamp.len(), "2026-11-03T19:55:39.123+11:00".len());
        assert_eq!(&timestamp[19..20], ".");
    }
}
