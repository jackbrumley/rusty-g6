//! App settings that the backend needs (the UI-only preferences live in the
//! webview's localStorage). Stored as `~/.config/rusty-g6/settings.json`.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// Closing the window parks it in the tray instead of quitting.
    pub close_to_tray: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            close_to_tray: true,
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    crate::paths::settings_file().ok()
}

/// Loads the settings, falling back to defaults when the file is missing or
/// unreadable. A bad file is left untouched until the next save.
pub fn load() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text).unwrap_or_else(|error| {
            log::warn!(
                "Could not parse {}: {}; using default settings",
                path.display(),
                error
            );
            AppSettings::default()
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
        Err(error) => {
            log::warn!(
                "Could not read {}: {}; using default settings",
                path.display(),
                error
            );
            AppSettings::default()
        }
    }
}

fn parse(text: &str) -> Result<AppSettings, serde_json::Error> {
    serde_json::from_str(text)
}

/// Writes the settings atomically (temporary file, then rename).
pub fn save(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or("Could not find the config directory")?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    let temp_path = path.with_extension("json.tmp");
    std::fs::write(&temp_path, text).map_err(|error| error.to_string())?;
    std::fs::rename(&temp_path, &path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_to_tray_defaults_on() {
        assert!(AppSettings::default().close_to_tray);
        assert!(parse("{}").unwrap().close_to_tray);
    }

    #[test]
    fn settings_round_trip() {
        let settings = AppSettings {
            close_to_tray: false,
        };
        let text = serde_json::to_string(&settings).unwrap();
        assert_eq!(parse(&text).unwrap(), settings);
    }
}
