import { invoke } from "@tauri-apps/api/core";

/// Write a UI event to the session log. A failed log write is reported in
/// the webview console only: logging must never interrupt the user.
export async function logUiEvent(message: string): Promise<void> {
  try {
    await invoke("log_ui_event", { message });
  } catch (error) {
    console.error("Failed to write a UI event to the session log:", error);
  }
}
