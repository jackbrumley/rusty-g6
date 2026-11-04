//! Single owner of Rusty G6's on-disk locations (App Standard section 3).
//! Nothing else calls `dirs::` directly.
//!
//! Everything Rusty G6 keeps lives under `~/.config/rusty-g6`
//! (`%APPDATA%\rusty-g6` on Windows): `settings.json` and the `debug` folder
//! with the session, last-session and update logs. The folder name predates
//! the reverse-DNS app ID and stays as it is.

use std::fs;
use std::path::PathBuf;

const APP_DIR_NAME: &str = "rusty-g6";

/// `~/.config/rusty-g6`, not created.
pub fn config_root() -> Result<PathBuf, String> {
    let base = dirs::config_dir().ok_or("Could not resolve the config directory")?;
    Ok(base.join(APP_DIR_NAME))
}

/// `~/.config/rusty-g6/settings.json`, not created.
pub fn settings_file() -> Result<PathBuf, String> {
    Ok(config_root()?.join("settings.json"))
}

/// `~/.config/rusty-g6/debug`, created: session, last-session and update logs.
pub fn debug_dir() -> Result<PathBuf, String> {
    let dir = config_root()?.join("debug");
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed creating directory '{}': {error}", dir.display()))?;
    Ok(dir)
}
