//! App settings that the backend needs (the UI-only preferences live in the
//! webview's localStorage). Stored as `~/.config/rusty-g6/settings.json`.

use crate::app::json_store;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Every field falls back to its `Default` value when missing, so new
/// fields need no migration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Closing the window parks it in the tray instead of quitting.
    pub close_to_tray: bool,
    /// One of `session_log::LOG_LEVELS`, chosen in Settings -> Debug.
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_to_tray: true,
            log_level: "info".to_string(),
        }
    }
}

pub fn settings_path() -> Result<PathBuf, String> {
    crate::paths::settings_file()
}

/// Loads the settings: defaults when the file is missing, or when it could
/// not be parsed (`json_store` moves it aside and tells the user). A file
/// that exists but cannot be read is an error; start-up checks for that
/// first with `json_store::ensure_readable` and exits.
pub fn load() -> Result<AppSettings, String> {
    Ok(json_store::read_json::<AppSettings>(&settings_path()?)?.unwrap_or_default())
}

/// Writes the settings atomically (temp file, fsync, rename).
pub fn save(settings: &AppSettings) -> Result<(), String> {
    json_store::write_json(&settings_path()?, settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<AppSettings, serde_json::Error> {
        serde_json::from_str(text)
    }

    #[test]
    fn missing_fields_take_their_defaults() {
        assert!(AppSettings::default().close_to_tray);
        let existing = parse(r#"{"close_to_tray": false}"#).unwrap();
        assert!(!existing.close_to_tray);
        assert_eq!(existing.log_level, "info");
        assert_eq!(parse("{}").unwrap(), AppSettings::default());
    }

    #[test]
    fn settings_round_trip() {
        let settings = AppSettings {
            close_to_tray: false,
            log_level: "debug".to_string(),
        };
        let text = serde_json::to_string(&settings).unwrap();
        assert_eq!(parse(&text).unwrap(), settings);
    }
}
