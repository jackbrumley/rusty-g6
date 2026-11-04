//! What a bug report needs about this machine (App Standard sections 3-4):
//! written as the session log header and shown in Settings -> Debug.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub app_name: String,
    pub app_version: String,
    pub build_profile: String,
    pub executable: String,
    pub os: String,
    pub distribution: String,
    pub architecture: String,
    pub desktop: String,
    pub session_type: String,
}

impl SystemInfo {
    pub fn collect() -> Self {
        Self {
            app_name: "Rusty G6".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            // A dev build and an installed package can share a version.
            build_profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
            .to_string(),
            executable: std::env::current_exe()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|error| format!("unknown ({error})")),
            os: std::env::consts::OS.to_string(),
            distribution: distribution_name(),
            architecture: std::env::consts::ARCH.to_string(),
            desktop: env_or_unknown("XDG_CURRENT_DESKTOP"),
            session_type: session_type(),
        }
    }

    /// One `key=value` line per field, for the log header and the copy button.
    pub fn summary(&self) -> String {
        format!(
            "app={} {}\nbuild={}\nexecutable={}\nos={}\ndistribution={}\narchitecture={}\ndesktop={}\nsession_type={}",
            self.app_name,
            self.app_version,
            self.build_profile,
            self.executable,
            self.os,
            self.distribution,
            self.architecture,
            self.desktop,
            self.session_type
        )
    }
}

fn env_or_unknown(name: &str) -> String {
    std::env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Wayland or X11, from the session's environment (Windows has neither).
fn session_type() -> String {
    if cfg!(target_os = "windows") {
        return "windows".to_string();
    }
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if !session_type.trim().is_empty() {
            return session_type;
        }
    }
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        "wayland".to_string()
    } else if std::env::var_os("DISPLAY").is_some() {
        "x11".to_string()
    } else {
        "unknown".to_string()
    }
}

/// The distribution's PRETTY_NAME on Linux.
#[cfg(target_os = "linux")]
fn distribution_name() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .map(|value| value.trim_matches('"').to_string())
            })
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// The Windows edition and build, from the registry.
#[cfg(target_os = "windows")]
fn distribution_name() -> String {
    let key = r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    let product = read_registry_value(key, "ProductName");
    let build = read_registry_value(key, "CurrentBuildNumber");
    match (product, build) {
        (Some(product), Some(build)) => format!("{product} (build {build})"),
        (Some(product), None) => product,
        _ => "unknown".to_string(),
    }
}

#[cfg(target_os = "windows")]
fn read_registry_value(key: &str, name: &str) -> Option<String> {
    use std::os::windows::process::CommandExt;
    // CREATE_NO_WINDOW: no console flashes up in the windowed app.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = std::process::Command::new("reg")
        .args(["query", key, "/v", name])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        (parts.next()? == name).then(|| {
            let _kind = parts.next();
            parts.collect::<Vec<_>>().join(" ")
        })
    })
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn distribution_name() -> String {
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::SystemInfo;

    #[test]
    fn summary_names_every_field() {
        let summary = SystemInfo::collect().summary();
        for key in [
            "app=Rusty G6 ",
            "build=",
            "executable=",
            "os=",
            "distribution=",
            "architecture=",
            "desktop=",
            "session_type=",
        ] {
            assert!(summary.contains(key), "missing {key} in {summary}");
        }
    }
}
