use crate::g6_spec::ProtocolConsoleMessage;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

static PROTOCOL_CONSOLE: Mutex<Vec<ProtocolConsoleMessage>> = Mutex::new(Vec::new());

pub fn push_console_message(app: &AppHandle, msg: ProtocolConsoleMessage) {
    PROTOCOL_CONSOLE.lock().unwrap().push(msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &msg) {
        eprintln!("Failed to emit console update: {}", e);
    }
}

pub fn log_to_console(app: &AppHandle, level: &str, text: String, details: Option<String>) {
    push_console_message(app, ProtocolConsoleMessage::new(level, text, details));
}

pub fn get_messages() -> Vec<ProtocolConsoleMessage> {
    PROTOCOL_CONSOLE.lock().unwrap().clone()
}

pub fn clear_messages() {
    PROTOCOL_CONSOLE.lock().unwrap().clear();
}
