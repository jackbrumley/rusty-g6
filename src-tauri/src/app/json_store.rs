//! Reading and writing Rusty G6's JSON settings file (App Standard
//! section 5), `settings.json`.
//!
//! - Writes are atomic: a temp file in the same directory is written,
//!   fsynced, then renamed over the original, so a crash mid-save never
//!   leaves a truncated file.
//! - A file that cannot be parsed is moved aside to
//!   `<file>.invalid-<timestamp>` and the caller starts from defaults; the user
//!   is told once with a native dialog.
//! - A file that exists but cannot be read is an error, never overwritten.
//!   At start-up `ensure_readable` turns that into an error dialog and exit.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Quarantine notices waiting for the UI, and the handle that shows them.
static PENDING_RECOVERY_NOTICES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static RECOVERY_NOTICE_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// Reads and parses `path`. `Ok(None)` when the file does not exist, or when
/// it could not be parsed and was moved aside (the caller uses defaults).
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable_message(path, &error)),
    };
    match serde_json::from_slice::<T>(&bytes) {
        Ok(value) => Ok(Some(value)),
        Err(parse_error) => {
            let backup_path = quarantine(path)?;
            let notice = format!(
                "{} could not be read ({parse_error}), so Rusty G6 moved it to {} and started from default settings.",
                path.display(),
                backup_path.display()
            );
            log::warn!(
                "SETTINGS_QUARANTINED path={} backup={} error={parse_error}",
                path.display(),
                backup_path.display()
            );
            report_recovery(notice);
            Ok(None)
        }
    }
}

/// Serializes `value` and writes it atomically.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Failed encoding {}: {error}", path.display()))?;
    write_atomically(path, text.as_bytes())
        .map_err(|error| format!("Failed writing {}: {error}", path.display()))
}

/// Fails if `path` exists but cannot be read (permissions, I/O), so start-up
/// can stop instead of later replacing the file with defaults.
pub fn ensure_readable(path: &Path) -> Result<(), String> {
    match fs::read(path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(unreadable_message(path, &error)),
    }
}

/// Shows quarantine notices from now on, and any recorded before start-up.
pub fn enable_recovery_notices(app_handle: &tauri::AppHandle) {
    let _ = RECOVERY_NOTICE_HANDLE.set(app_handle.clone());
    let pending = PENDING_RECOVERY_NOTICES
        .lock()
        .map(|mut notices| std::mem::take(&mut *notices))
        .unwrap_or_default();
    for notice in pending {
        show_recovery_notice(app_handle, notice);
    }
}

fn report_recovery(notice: String) {
    match RECOVERY_NOTICE_HANDLE.get() {
        Some(app_handle) => show_recovery_notice(app_handle, notice),
        None => {
            if let Ok(mut notices) = PENDING_RECOVERY_NOTICES.lock() {
                notices.push(notice);
            }
        }
    }
}

fn show_recovery_notice(app_handle: &tauri::AppHandle, notice: String) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
    app_handle
        .dialog()
        .message(notice)
        .title("Rusty G6 reset its settings")
        .kind(MessageDialogKind::Warning)
        .show(|_| {});
}

fn unreadable_message(path: &Path, error: &io::Error) -> String {
    format!("Could not read {}: {error}", path.display())
}

/// Moves an unparseable file to `<file>.invalid-<timestamp>` so it survives
/// for manual repair instead of being overwritten.
fn quarantine(path: &Path) -> Result<PathBuf, String> {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "settings.json".to_string());
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup_path = path.with_file_name(format!("{file_name}.invalid-{timestamp}"));
    fs::rename(path, &backup_path).map_err(|error| {
        format!(
            "{} could not be parsed, and moving it aside to {} failed: {error}",
            path.display(),
            backup_path.display()
        )
    })?;
    Ok(backup_path)
}

/// Writes a sibling temp file, fsyncs it, then renames it over the target,
/// so a crash or power loss leaves either the old file or the new one.
fn write_atomically(target_path: &Path, contents: &[u8]) -> io::Result<()> {
    let directory = target_path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
    fs::create_dir_all(directory)?;
    let file_name = target_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();
    // Unique per write, so concurrent saves never share a temp file.
    let temp_path = directory.join(format!(
        ".{file_name}.tmp-{}-{}",
        std::process::id(),
        TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    let write_result = (|| {
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        temp_file.write_all(contents)?;
        temp_file.sync_all()?;
        drop(temp_file);
        fs::rename(&temp_path, target_path)
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
        return write_result;
    }

    // Make the rename itself durable (Unix; Windows cannot open a directory
    // as a file).
    #[cfg(unix)]
    if let Ok(directory_handle) = fs::File::open(directory) {
        let _ = directory_handle.sync_all();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "rusty-g6-json-store-{name}-{}-{}",
            std::process::id(),
            TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn write_replaces_the_file_and_leaves_no_temp() {
        let directory = test_directory("write");
        let target = directory.join("settings.json");
        fs::write(&target, b"old").unwrap();

        write_json(&target, &serde_json::json!({ "value": 1 })).unwrap();

        let reread: serde_json::Value = read_json(&target).unwrap().unwrap();
        assert_eq!(reread["value"], 1);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn missing_file_reads_as_none() {
        let directory = test_directory("missing");
        let result: Option<serde_json::Value> = read_json(&directory.join("absent.json")).unwrap();
        assert!(result.is_none());
        assert!(ensure_readable(&directory.join("absent.json")).is_ok());
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn unparseable_file_is_moved_aside() {
        let directory = test_directory("invalid");
        let target = directory.join("settings.json");
        fs::write(&target, br#"{"close_to_tray": "#).unwrap();

        let result: Option<serde_json::Value> = read_json(&target).unwrap();

        assert!(result.is_none());
        assert!(!target.exists());
        let backups: Vec<String> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(backups.len(), 1);
        assert!(backups[0].starts_with("settings.json.invalid-"));
        fs::remove_dir_all(&directory).unwrap();
    }
}
