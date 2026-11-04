//! In-app updates, per the Release and Update Standard
//! (https://github.com/jackbrumley/jackbrumley.github.io/blob/main/RELEASE_STANDARD.md):
//! the update check reads GitHub's latest release, and installing an update
//! re-runs the same one-line installer a new user runs.

use std::process::Stdio;

use serde::{Deserialize, Serialize};

const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/jackbrumley/rusty-g6/releases/latest";
const INSTALL_SITE: &str = "https://brumley.info/rusty-g6";
const PRODUCT_NAME: &str = "Rusty G6";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    notes_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubLatestRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .build()
        .map_err(|error| format!("Failed to build update client: {error}"))?;

    let response = client
        .get(GITHUB_LATEST_RELEASE_API_URL)
        .header(reqwest::header::USER_AGENT, "rusty-g6-update-checker")
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| format!("Failed to fetch latest release: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Latest release request failed with status {}",
            response.status()
        ));
    }

    let latest_release: GitHubLatestRelease = response
        .json()
        .await
        .map_err(|error| format!("Failed to decode latest release response: {error}"))?;

    let latest_version = normalize_version(&latest_release.tag_name).ok_or_else(|| {
        format!(
            "Invalid latest release version tag: {}",
            latest_release.tag_name
        )
    })?;

    let parsed_current = parse_version(&current_version)
        .ok_or_else(|| format!("Invalid current app version: {current_version}"))?;
    let parsed_latest = parse_version(&latest_version)
        .ok_or_else(|| format!("Invalid latest app version: {latest_version}"))?;

    let update_available = parsed_latest > parsed_current;
    log::info!(
        "Update check: current={current_version} latest={latest_version} available={update_available}"
    );

    Ok(UpdateCheckResult {
        current_version,
        latest_version,
        update_available,
        notes_url: latest_release
            .body
            .filter(|body| !body.trim().is_empty())
            .map(|_| latest_release.html_url.clone()),
        release_url: latest_release.html_url,
    })
}

/// Starts the installer in a detached process, then exits so the installer
/// can replace this installation. The installer relaunches the new version;
/// if it fails, the still-installed version is relaunched with
/// `--update-failed`.
#[tauri::command]
pub async fn install_update(app_handle: tauri::AppHandle) -> Result<(), String> {
    log::info!("Starting in-app update");

    spawn_update_process()?;

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
        log::info!("Exiting {PRODUCT_NAME} for update installation");
        handle.exit(0);
    });

    Ok(())
}

/// The install command that replaces this installation in place. AppImage
/// installs must stay AppImage installs rather than switching to a package.
#[cfg(target_os = "linux")]
fn unix_update_command() -> String {
    let is_appimage = std::env::var_os("APPIMAGE").is_some()
        || std::env::current_exe()
            .map(|path| path.to_string_lossy().contains(".local/bin"))
            .unwrap_or(false);

    if is_appimage {
        format!("curl --proto '=https' --tlsv1.2 -sSf {INSTALL_SITE}/install.sh | bash -s -- --appimage")
    } else {
        format!("curl --proto '=https' --tlsv1.2 -sSf {INSTALL_SITE}/install.sh | bash")
    }
}

/// The installer's output, and what the update-failed dialog opens.
fn update_log_path() -> std::path::PathBuf {
    crate::paths::debug_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("update.log")
}

/// A fresh update log, as the updater's stdout and stderr. Emptied first, then
/// opened in append mode: install.sh also appends its own lines to this file,
/// and a plain write handle would write over them at its own offset.
fn update_log_outputs() -> Result<(Stdio, Stdio), String> {
    let log_path = update_log_path();
    std::fs::write(&log_path, b"")
        .map_err(|error| format!("Failed to reset update log file: {error}"))?;
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&log_path)
        .map_err(|error| format!("Failed to open update log file: {error}"))?;
    let log_file_err = log_file
        .try_clone()
        .map_err(|error| format!("Failed to clone update log handle: {error}"))?;
    Ok((Stdio::from(log_file), Stdio::from(log_file_err)))
}

/// What to relaunch, with `--update-failed`, when the installer fails: the
/// AppImage file itself (its mounted executable disappears on exit),
/// otherwise this executable.
fn relaunch_target() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(appimage) = std::env::var_os("APPIMAGE") {
            return Some(std::path::PathBuf::from(appimage));
        }
    }
    std::env::current_exe().ok()
}

/// Wraps `value` in single quotes for bash, which takes everything inside
/// literally (an embedded quote closes, escapes and reopens).
#[cfg(any(target_os = "linux", test))]
fn bash_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

/// Wraps `value` in single quotes for PowerShell, which escapes an embedded
/// quote by doubling it.
#[cfg(any(target_os = "windows", test))]
fn powershell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Runs the installer, and if it fails relaunches the installed version with
/// `--update-failed` so the user is told instead of the app just vanishing.
/// `pipefail` makes a failed download fail the update rather than piping an
/// empty script into bash.
#[cfg(target_os = "linux")]
fn unix_update_script() -> String {
    let relaunch = match relaunch_target() {
        Some(path) => {
            let path = bash_single_quote(&path.to_string_lossy());
            format!("if [ -x {path} ]; then {path} --update-failed >/dev/null 2>&1 & fi")
        }
        None => "echo \"[rusty-g6] No installed version found to relaunch\"".to_string(),
    };
    format!(
        r#"set -o pipefail
{install}
status=$?
if [ "$status" -ne 0 ]; then
  echo "[rusty-g6] Update failed (exit $status); relaunching the installed version"
  {relaunch}
fi
"#,
        install = unix_update_command(),
    )
}

/// Windows counterpart of `unix_update_script`. install.ps1 runs from a file
/// rather than through `iex`, so its `exit 1` on failure ends the script, not
/// this updater, and sets `$LASTEXITCODE`.
#[cfg(target_os = "windows")]
fn windows_update_script() -> String {
    let relaunch = match relaunch_target() {
        Some(path) => {
            let path = powershell_single_quote(&path.to_string_lossy());
            format!(
                "if (Test-Path -LiteralPath {path}) {{ Start-Process -FilePath {path} -ArgumentList '--update-failed' }}"
            )
        }
        None => "Write-Output '[rusty-g6] No installed version found to relaunch'".to_string(),
    };
    format!(
        r#"$installer = Join-Path $env:TEMP 'rusty-g6-update-install.ps1'
try {{
  Invoke-RestMethod -Uri '{INSTALL_SITE}/install.ps1' -OutFile $installer -ErrorAction Stop
  $LASTEXITCODE = 0
  & $installer
  $succeeded = $LASTEXITCODE -eq 0
}} catch {{
  Write-Output "[rusty-g6] ERROR: $_"
  $succeeded = $false
}}
if (-not $succeeded) {{
  Write-Output '[rusty-g6] Update failed; relaunching the installed version'
  {relaunch}
}}
"#
    )
}

#[cfg(target_os = "linux")]
fn spawn_update_process() -> Result<(), String> {
    use std::os::unix::process::CommandExt;

    let script = unix_update_script();
    log::info!("Spawning update process:\n{script}");
    let (stdout, stderr) = update_log_outputs()?;

    let mut command = std::process::Command::new("bash");
    command
        .arg("-c")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr);

    // SAFETY: setsid only detaches the child from this process's session, so
    // the updater survives Rusty G6 exiting.
    unsafe {
        command.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    command
        .spawn()
        .map_err(|error| format!("Failed to spawn updater process: {error}"))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn spawn_update_process() -> Result<(), String> {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

    let script = windows_update_script();
    log::info!("Spawning Windows update process:\n{script}");
    let (stdout, stderr) = update_log_outputs()?;

    let mut command = std::process::Command::new("powershell.exe");
    command
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

    command
        .spawn()
        .map_err(|error| format!("Failed to spawn updater process: {error}"))?;

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn spawn_update_process() -> Result<(), String> {
    Err("In-app updates are not supported on this platform".to_string())
}

/// Shown when the updater relaunched this (old) version after the installer
/// failed, so a failed update is never silent.
pub fn show_update_failed_dialog(app_handle: &tauri::AppHandle) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

    let log_path = update_log_path();
    log::warn!(
        "Relaunched after a failed update; details in {}",
        log_path.display()
    );
    app_handle
        .dialog()
        .message(format!(
            "{PRODUCT_NAME} could not install the update, so this version is still installed.\n\nThe installer's output is in:\n{}",
            log_path.display()
        ))
        .title("Update failed")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Open Update Log".to_string(),
            "Close".to_string(),
        ))
        .show(move |open_log| {
            if open_log {
                if let Err(error) =
                    tauri_plugin_opener::open_path(&log_path, None::<&str>)
                {
                    log::warn!("Failed to open the update log: {error}");
                }
            }
        });
}

fn normalize_version(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches('v');
    let parsed = parse_version(trimmed)?;
    Some(format!("{}.{}.{}", parsed.0, parsed.1, parsed.2))
}

fn parse_version(value: &str) -> Option<(u64, u64, u64)> {
    let sanitized = value.trim();
    if sanitized.is_empty() {
        return None;
    }

    let core = sanitized
        .split_once('-')
        .map_or(sanitized, |(left, _)| left);
    let mut segments = core.split('.');

    let major = segments.next()?.parse::<u64>().ok()?;
    let minor = segments.next()?.parse::<u64>().ok()?;
    let patch = segments.next()?.parse::<u64>().ok()?;

    if segments.next().is_some() {
        return None;
    }

    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::{bash_single_quote, normalize_version, powershell_single_quote};

    #[test]
    fn bash_quoting_keeps_paths_literal() {
        assert_eq!(
            bash_single_quote("/usr/bin/rusty-g6"),
            "'/usr/bin/rusty-g6'"
        );
        assert_eq!(
            bash_single_quote("/home/o'brien/Rusty G6 $HOME.AppImage"),
            r"'/home/o'\''brien/Rusty G6 $HOME.AppImage'"
        );
    }

    #[test]
    fn powershell_quoting_doubles_embedded_quotes() {
        assert_eq!(
            powershell_single_quote(r"C:\Users\o'brien\AppData\Local\rusty-g6\rusty-g6.exe"),
            r"'C:\Users\o''brien\AppData\Local\rusty-g6\rusty-g6.exe'"
        );
    }

    #[test]
    fn versions_normalize_from_tags() {
        assert_eq!(normalize_version("v1.0.7").as_deref(), Some("1.0.7"));
        assert_eq!(normalize_version("1.2"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn unix_update_script_fails_on_a_failed_download_and_relaunches() {
        let script = super::unix_update_script();
        assert!(script.starts_with("set -o pipefail\n"));
        assert!(script.contains("https://brumley.info/rusty-g6/install.sh"));
        assert!(script.contains("--update-failed"));
    }
}
