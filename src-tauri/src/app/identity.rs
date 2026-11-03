//! The app's identity, per the Release and Update Standard's "App Identity"
//! section: one reverse-DNS ID used as the Tauri `identifier`, the Wayland
//! `app_id` / GTK prgname and the launcher name (`<APP_ID>.desktop`).

/// Must equal the Tauri `identifier`. Never ends in `.desktop` or `.app`.
pub const APP_ID: &str = "info.brumley.rustyg6";

/// Sets the GTK prgname (which GTK uses as the Wayland `app_id`) so desktop
/// environments match the window to `info.brumley.rustyg6.desktop`. Must run
/// before any GTK use, i.e. at the very start of `run`.
pub fn configure_app_identity() {
    #[cfg(target_os = "linux")]
    {
        gtk::glib::set_prgname(Some(APP_ID));
        gtk::glib::set_application_name("Rusty G6");
    }
}

#[cfg(test)]
mod tests {
    use super::APP_ID;

    #[test]
    fn app_id_matches_tauri_identifier() {
        let tauri_config: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).unwrap();
        assert_eq!(tauri_config["identifier"], APP_ID);
    }

    #[test]
    fn launcher_is_named_after_app_id() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let launcher = manifest_dir.join(format!("packaging/{APP_ID}.desktop"));
        let contents = std::fs::read_to_string(&launcher).unwrap();
        assert!(contents.contains(&format!("StartupWMClass={APP_ID}")));
    }

    #[test]
    fn app_id_follows_the_naming_rules() {
        assert!(!APP_ID.ends_with(".desktop") && !APP_ID.ends_with(".app"));
        assert!(APP_ID.split('.').all(|segment| !segment.is_empty()
            && segment
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())));
    }
}
