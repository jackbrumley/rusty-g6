//! Single owner of Rusty G6's on-disk locations (App Standard section 3).
//! Nothing else calls `dirs::` directly.
//!
//! Everything Rusty G6 keeps lives under `~/.config/rusty-g6` on every
//! platform: `settings.json` and the `debug` folder with the session,
//! last-session and update logs. The folder name predates the reverse-DNS app
//! ID and stays as it is. Releases up to 1.0.12 kept the Windows folder in
//! `%APPDATA%\rusty-g6`; `migrate_legacy_location` moves it once at start-up.

use std::fs;
use std::path::{Path, PathBuf};

const APP_DIR_NAME: &str = "rusty-g6";

/// The config root, not created.
///
/// Linux: `$XDG_CONFIG_HOME/rusty-g6` (or `~/.config/rusty-g6`).
/// Windows: `~/.config/rusty-g6` (`%USERPROFILE%\.config\rusty-g6`), the same
/// place as on Linux rather than `%APPDATA%`.
pub fn config_root() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    let base = dirs::config_dir().ok_or("Could not resolve the config directory")?;
    #[cfg(not(target_os = "linux"))]
    let base = dirs::home_dir()
        .ok_or("Could not resolve the home directory")?
        .join(".config");
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

/// The Windows config root of releases up to 1.0.12, `%APPDATA%\rusty-g6`.
/// Linux never had a different location.
fn legacy_config_root() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|dir| dir.join(APP_DIR_NAME))
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// What the one-time move from the legacy config root did, for the session
/// log (written once it is open).
#[derive(Debug, PartialEq, Eq)]
pub struct MigrationReport {
    pub level: log::Level,
    pub message: String,
}

impl MigrationReport {
    fn info(message: String) -> Self {
        Self {
            level: log::Level::Info,
            message,
        }
    }

    fn warn(message: String) -> Self {
        Self {
            level: log::Level::Warn,
            message,
        }
    }
}

/// One-time move of `%APPDATA%\rusty-g6` to `~/.config/rusty-g6` on
/// Windows. Runs before the session log opens, because opening it creates
/// the new folder. `None` when there is nothing to migrate.
pub fn migrate_legacy_location() -> Option<MigrationReport> {
    let legacy_root = legacy_config_root()?;
    if !legacy_root.exists() {
        return None;
    }
    match config_root() {
        Ok(new_root) => Some(migrate_config_root(&legacy_root, &new_root)),
        Err(error) => Some(MigrationReport::warn(format!(
            "Config found at {} but the new location is unavailable ({error}); legacy folder left in place",
            legacy_root.display()
        ))),
    }
}

/// Moves `legacy_root` to `new_root`: a rename, or a copy then delete when the
/// rename fails (for example across volumes). Never touches an existing
/// `new_root`, and removes the legacy folder only once its contents are in
/// place.
fn migrate_config_root(legacy_root: &Path, new_root: &Path) -> MigrationReport {
    if new_root.exists() {
        return MigrationReport::warn(format!(
            "Both the legacy config folder {} and the new {} exist; using the new one and leaving the legacy folder untouched",
            legacy_root.display(),
            new_root.display()
        ));
    }

    // fs::rename needs the destination's parent; a fresh Windows profile has
    // no `.config` folder yet.
    if let Some(parent) = new_root.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            return MigrationReport::warn(format!(
                "Failed to create {} to migrate the config folder ({error}); legacy folder {} left in place",
                parent.display(),
                legacy_root.display()
            ));
        }
    }

    let rename_error = match fs::rename(legacy_root, new_root) {
        Ok(()) => {
            return MigrationReport::info(format!(
                "Moved the config folder from {} to {}",
                legacy_root.display(),
                new_root.display()
            ));
        }
        Err(error) => error,
    };

    if let Err(copy_error) = copy_tree(legacy_root, new_root) {
        // A partial copy must not shadow the legacy folder on the next start.
        let cleanup = match fs::remove_dir_all(new_root) {
            Ok(()) => String::new(),
            Err(error) => format!("; removing the partial copy also failed: {error}"),
        };
        return MigrationReport::warn(format!(
            "FAILED to migrate the config folder from {} to {} (rename: {rename_error}; copy: {copy_error}{cleanup}); legacy folder left in place",
            legacy_root.display(),
            new_root.display()
        ));
    }

    match fs::remove_dir_all(legacy_root) {
        Ok(()) => MigrationReport::info(format!(
            "Copied the config folder from {} to {} (rename failed: {rename_error}) and removed the legacy folder",
            legacy_root.display(),
            new_root.display()
        )),
        Err(remove_error) => MigrationReport::warn(format!(
            "Copied the config folder from {} to {} (rename failed: {rename_error}) but could not remove the legacy folder: {remove_error}",
            legacy_root.display(),
            new_root.display()
        )),
    }
}

fn copy_tree(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEST_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "rusty-g6-paths-{name}-{}-{}",
            std::process::id(),
            TEST_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    fn write_legacy_config(legacy_root: &Path) {
        fs::create_dir_all(legacy_root.join("debug")).unwrap();
        fs::write(
            legacy_root.join("settings.json"),
            b"{\"log_level\":\"debug\"}",
        )
        .unwrap();
        fs::write(legacy_root.join("debug").join("session.log"), b"old log").unwrap();
    }

    #[test]
    fn config_root_is_the_app_folder_in_the_config_directory() {
        let root = config_root().unwrap();
        assert!(root.ends_with(APP_DIR_NAME));
        #[cfg(not(target_os = "linux"))]
        assert_eq!(
            root,
            dirs::home_dir().unwrap().join(".config").join(APP_DIR_NAME)
        );
    }

    #[test]
    fn settings_file_is_settings_json_in_the_config_root() {
        assert_eq!(
            settings_file().unwrap(),
            config_root().unwrap().join("settings.json")
        );
    }

    #[test]
    fn migration_moves_the_legacy_folder_into_a_missing_parent() {
        let base = test_directory("move");
        let legacy_root = base.join("AppData").join(APP_DIR_NAME);
        let new_root = base.join(".config").join(APP_DIR_NAME);
        write_legacy_config(&legacy_root);

        let report = migrate_config_root(&legacy_root, &new_root);

        assert_eq!(report.level, log::Level::Info);
        assert!(!legacy_root.exists());
        assert_eq!(
            fs::read(new_root.join("settings.json")).unwrap(),
            b"{\"log_level\":\"debug\"}"
        );
        assert_eq!(
            fs::read(new_root.join("debug").join("session.log")).unwrap(),
            b"old log"
        );
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn migration_never_overwrites_an_existing_new_folder() {
        let base = test_directory("both");
        let legacy_root = base.join("AppData").join(APP_DIR_NAME);
        let new_root = base.join(".config").join(APP_DIR_NAME);
        write_legacy_config(&legacy_root);
        fs::create_dir_all(&new_root).unwrap();
        fs::write(new_root.join("settings.json"), b"new").unwrap();

        let report = migrate_config_root(&legacy_root, &new_root);

        assert_eq!(report.level, log::Level::Warn);
        assert_eq!(fs::read(new_root.join("settings.json")).unwrap(), b"new");
        assert!(legacy_root.join("settings.json").is_file());
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn copy_tree_copies_nested_files_and_keeps_the_source() {
        let base = test_directory("copy");
        let source = base.join("source");
        let destination = base.join("destination");
        write_legacy_config(&source);

        copy_tree(&source, &destination).unwrap();

        assert!(destination.join("settings.json").is_file());
        assert_eq!(
            fs::read(destination.join("debug").join("session.log")).unwrap(),
            b"old log"
        );
        assert!(source.join("settings.json").is_file());
        fs::remove_dir_all(&base).unwrap();
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn only_windows_has_a_legacy_location() {
        assert_eq!(migrate_legacy_location(), None);
    }
}
