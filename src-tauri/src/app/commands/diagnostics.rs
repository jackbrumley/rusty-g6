use std::process::Command;

#[tauri::command]
pub fn get_session_log_text() -> Result<String, String> {
    crate::app::session_log::get_session_log_text()
}

#[tauri::command]
pub fn open_session_log() -> Result<(), String> {
    let log_path = crate::app::session_log::resolve_session_log_path()?;

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&log_path)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&log_path)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&log_path)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn log_ui_event(message: String) {
    log::info!("[UI] {}", message);
}
