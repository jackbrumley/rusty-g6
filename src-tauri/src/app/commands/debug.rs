use crate::app::protocol_console::{clear_messages, get_messages, push_console_message};
use crate::app::state::AppState;
use crate::g6_protocol_v2::{
    build_firmware_query_ascii, build_toggle_output_simple, G6ResponseParser,
};
use crate::g6_spec::ProtocolConsoleMessage;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_protocol_console_messages() -> Vec<ProtocolConsoleMessage> {
    get_messages()
}

#[tauri::command]
pub fn clear_protocol_console() -> Result<String, String> {
    clear_messages();
    Ok("Console cleared".to_string())
}

#[tauri::command]
pub fn test_protocol_v2(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();

    if !manager.is_connected() {
        return Err("Device not connected".to_string());
    }

    eprintln!("=== TESTING PROTOCOL V2 ===");
    let command = build_firmware_query_ascii();

    let cmd_hex: String = command
        .iter()
        .take(20)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    let cmd_msg = ProtocolConsoleMessage::new(
        "command",
        "Firmware Query (ASCII Mode) - Protocol V2".to_string(),
        Some(format!("Command bytes: {}", cmd_hex)),
    );
    push_console_message(&app, cmd_msg);

    eprintln!("Command (V2): {}", cmd_hex);

    let response = match manager.send_raw_command(&command) {
        Ok(resp) => resp,
        Err(e) => {
            let err_msg = ProtocolConsoleMessage::new(
                "error",
                format!("Failed to send command: {}", e),
                None,
            );
            push_console_message(&app, err_msg);
            return Err(format!("Failed to send command: {}", e));
        }
    };

    let resp_hex: String = response
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    let resp_ascii: String = response
        .iter()
        .filter(|&&b| b >= 0x20 && b <= 0x7E)
        .map(|&b| b as char)
        .collect();

    let resp_msg = ProtocolConsoleMessage::new(
        "response",
        "Device response received".to_string(),
        Some(format!("Hex: {}\nASCII: {}", resp_hex, resp_ascii)),
    );
    push_console_message(&app, resp_msg);

    let (parsed, debug_info) = G6ResponseParser::parse(&response);
    let parse_level = if parsed.is_ok() { "info" } else { "error" };
    let parse_text = match &parsed {
        Ok(p) => format!("✅ Parse SUCCESS: {:?}", p),
        Err(e) => format!("❌ Parse FAILED: {}", e),
    };

    let parse_msg =
        ProtocolConsoleMessage::new(parse_level, parse_text, Some(debug_info.to_readable_text()));
    push_console_message(&app, parse_msg);

    match parsed {
        Ok(_) => Ok("Protocol V2 test complete! Check console for full details.".to_string()),
        Err(e) => Ok(format!(
            "Test sent but parsing failed: {}. Check console for details.",
            e
        )),
    }
}

#[tauri::command]
pub fn test_output_toggle_v2(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();

    if !manager.is_connected() {
        return Err("Device not connected".to_string());
    }

    let current_output = manager.get_settings().output;

    let start_msg = ProtocolConsoleMessage::new(
        "command",
        format!("🔄 Testing V2 Output Toggle from {:?}", current_output),
        Some("Simple 2-command version (routing + commit)".to_string()),
    );
    push_console_message(&app, start_msg);

    let commands = build_toggle_output_simple(current_output);

    let info_msg = ProtocolConsoleMessage::new(
        "info",
        format!("Built {} commands (V2 minimal approach)", commands.len()),
        None,
    );
    push_console_message(&app, info_msg);

    for (i, cmd) in commands.iter().enumerate() {
        let cmd_hex: String = cmd
            .iter()
            .take(20)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let cmd_desc = if i == 0 {
            "Step 1: Set output routing"
        } else {
            "Step 2: Commit change"
        };

        let cmd_msg = ProtocolConsoleMessage::new(
            "command",
            cmd_desc.to_string(),
            Some(format!("Hex: {}", cmd_hex)),
        );
        push_console_message(&app, cmd_msg);

        let response = match manager.send_raw_command(cmd) {
            Ok(resp) => resp,
            Err(e) => {
                let err_msg = ProtocolConsoleMessage::new(
                    "error",
                    format!("❌ Step {} failed: {}", i + 1, e),
                    None,
                );
                push_console_message(&app, err_msg);
                return Err(format!("Failed at step {}: {}", i + 1, e));
            }
        };

        let resp_hex: String = response
            .iter()
            .take(20)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let resp_msg = ProtocolConsoleMessage::new(
            "response",
            format!("Response {}/{}", i + 1, commands.len()),
            Some(format!("Hex: {}", resp_hex)),
        );
        push_console_message(&app, resp_msg);
    }

    let success_msg = ProtocolConsoleMessage::new(
        "info",
        "✅ V2 output toggle test complete!".to_string(),
        Some("Check if output actually switched. Try Read State to verify.".to_string()),
    );
    push_console_message(&app, success_msg);

    Ok(format!(
        "V2 toggle test sent ({} commands). Check Protocol Console for details.",
        commands.len()
    ))
}
