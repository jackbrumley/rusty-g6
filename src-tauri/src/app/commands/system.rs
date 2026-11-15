use log::info;
use std::process::Command;
use tauri::AppHandle;

#[tauri::command]
pub fn quit_application(app: AppHandle) {
    crate::app::tray::quit(&app);
}

#[tauri::command]
pub fn check_usb_permissions() -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        use std::path::Path;

        let rule_path = "/etc/udev/rules.d/99-soundblaster-g6.rules";
        if Path::new(rule_path).exists() {
            return Ok(true);
        }
        Ok(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(true)
    }
}

#[tauri::command]
pub async fn setup_udev_rules() -> Result<String, String> {
    info!("Setting up USB udev rules...");

    let script = r#"
# Sound BlasterX G6 - VID: 041e, PID: 3256
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
SUBSYSTEM=="usb", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", MODE="0666", TAG+="uaccess"
"#;

    let cmd = format!(
        "echo '{}' > /etc/udev/rules.d/99-soundblaster-g6.rules && udevadm control --reload-rules && udevadm trigger",
        script
    );

    let output = Command::new("pkexec")
        .args(["bash", "-c", &cmd])
        .output()
        .map_err(|e| format!("Failed to execute pkexec: {}", e))?;

    if output.status.success() {
        Ok("USB permissions set up successfully. You can now connect to the device.".to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to set up USB permissions: {}", err))
    }
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn clear_terminal(message: Option<String>) -> Result<String, String> {
    print!("\x1b[2J\x1b[H");

    info!("============================================================");
    if let Some(msg) = message {
        if !msg.trim().is_empty() {
            info!("LOG SEPARATOR: {}", msg.trim());
        } else {
            info!("LOG SEPARATOR - User requested terminal break");
        }
    } else {
        info!("LOG SEPARATOR - User requested terminal break");
    }
    info!("============================================================");

    Ok("Log separator added".to_string())
}
