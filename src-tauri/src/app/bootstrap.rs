use crate::app::state::AppState;
use tauri::Manager;

pub fn setup(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();

    let open_item =
        tauri::menu::MenuItem::with_id(app, "open", "Open Rusty G6", true, None::<&str>)?;
    let quit_item = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = tauri::menu::Menu::with_items(app, &[&open_item, &quit_item])?;

    let tray_result = tauri::tray::TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(|app_handle, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app_handle.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app);

    *state.tray_available.lock().unwrap() = tray_result.is_ok();
    if let Err(error) = tray_result {
        eprintln!(
            "Tray initialization failed, using taskbar minimize fallback: {}",
            error
        );
    }

    Ok(())
}
