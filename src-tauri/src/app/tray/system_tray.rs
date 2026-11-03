//! Windows tray, built on Tauri's tray icon.

use super::{handle_menu_item, show_main_window, TrayMenuItem, TrayOutputState};
use tauri::image::Image;
use tauri::menu::{IsMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

/// The output menu item, kept so it can be relabeled.
struct OutputMenuItem(MenuItem<tauri::Wry>);

/// Tauri menu events identify items by string id.
fn menu_item_id(item: TrayMenuItem) -> &'static str {
    match item {
        TrayMenuItem::Open => "open",
        TrayMenuItem::SwitchOutput => "switch-output",
        TrayMenuItem::Quit => "quit",
    }
}

fn menu_item_from_id(id: &str) -> Option<TrayMenuItem> {
    TrayMenuItem::ALL
        .into_iter()
        .find(|item| menu_item_id(*item) == id)
}

pub fn build(app: &tauri::App<tauri::Wry>, icon: Image<'static>) -> Result<(), tauri::Error> {
    let initial = TrayOutputState::Disconnected;
    let items = TrayMenuItem::ALL
        .into_iter()
        .map(|item| {
            MenuItem::with_id(
                app,
                menu_item_id(item),
                item.label(initial),
                item.enabled(initial),
                None::<&str>,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let item_refs: Vec<&dyn IsMenuItem<tauri::Wry>> = items
        .iter()
        .map(|item| item as &dyn IsMenuItem<tauri::Wry>)
        .collect();
    let menu = Menu::with_items(app, &item_refs)?;

    if let Some(output_item) = TrayMenuItem::ALL
        .iter()
        .position(|item| *item == TrayMenuItem::SwitchOutput)
        .map(|index| items[index].clone())
    {
        app.manage(OutputMenuItem(output_item));
    }

    TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .tooltip("Rusty G6")
        .show_menu_on_left_click(false)
        .icon(icon)
        .on_menu_event(|app_handle, event| {
            if let Some(item) = menu_item_from_id(event.id.as_ref()) {
                handle_menu_item(app_handle, item);
            }
        })
        .on_tray_icon_event(|tray, event| {
            // Click fires on both press and release; act once, on release.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Relabels the output item.
pub fn set_output_state(app_handle: &tauri::AppHandle, output: TrayOutputState) {
    let Some(item) = app_handle.try_state::<OutputMenuItem>() else {
        return;
    };
    let item = &item.0;
    let label = TrayMenuItem::SwitchOutput.label(output);
    if let Err(error) = item.set_text(label) {
        log::warn!("Failed to relabel the tray output item: {}", error);
    }
    if let Err(error) = item.set_enabled(TrayMenuItem::SwitchOutput.enabled(output)) {
        log::warn!("Failed to update the tray output item: {}", error);
    }
}

#[cfg(test)]
mod tests {
    use super::{menu_item_from_id, menu_item_id, TrayMenuItem};

    #[test]
    fn menu_item_ids_round_trip() {
        for item in TrayMenuItem::ALL {
            assert_eq!(menu_item_from_id(menu_item_id(item)), Some(item));
        }
        assert_eq!(menu_item_from_id("unknown"), None);
    }
}
