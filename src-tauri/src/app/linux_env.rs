//! Linux start-up environment (App Standard section 2). Runs at the very
//! start of `main`, before Tauri, GTK or any portal call, while the process
//! is still single-threaded.

use std::ffi::OsStr;

pub fn configure_linux_session_environment() {
    // GTK prgname / application name: the Wayland app_id the desktop matches
    // to the launcher (Release Standard section 10).
    crate::app::identity::configure_app_identity();

    // WebKitGTK shows a blank window or crashes with "Could not create default
    // EGL display: EGL_BAD_PARAMETER" on Arch-based distros (CachyOS, Manjaro)
    // and some NVIDIA setups unless its DMA-BUF renderer and GPU sandbox are
    // disabled.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    std::env::set_var("WEBKIT_DISABLE_GPU_SANDBOX", "1");

    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    if should_force_wayland_backend(
        std::env::var_os("GDK_BACKEND").as_deref(),
        session_type.as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
    ) {
        // Native Wayland, rather than XWayland, on Wayland sessions.
        std::env::set_var("GDK_BACKEND", "wayland");
    }
}

/// Force GTK's native Wayland backend only on a real Wayland session with a
/// reachable display, and never over a `GDK_BACKEND` the user chose:
/// otherwise GTK fails to start where no Wayland display is reachable.
fn should_force_wayland_backend(
    gdk_backend: Option<&OsStr>,
    session_type: Option<&str>,
    wayland_display: Option<&OsStr>,
) -> bool {
    gdk_backend.is_none()
        && session_type.is_some_and(|value| value.eq_ignore_ascii_case("wayland"))
        && wayland_display.is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::should_force_wayland_backend;
    use std::ffi::OsStr;

    fn display() -> Option<&'static OsStr> {
        Some(OsStr::new("wayland-0"))
    }

    #[test]
    fn wayland_session_with_a_display_forces_wayland() {
        assert!(should_force_wayland_backend(
            None,
            Some("wayland"),
            display()
        ));
        assert!(should_force_wayland_backend(
            None,
            Some("Wayland"),
            display()
        ));
    }

    #[test]
    fn user_choice_is_kept() {
        assert!(!should_force_wayland_backend(
            Some(OsStr::new("x11")),
            Some("wayland"),
            display()
        ));
    }

    #[test]
    fn x11_or_unknown_sessions_are_left_alone() {
        assert!(!should_force_wayland_backend(None, Some("x11"), display()));
        assert!(!should_force_wayland_backend(None, None, display()));
    }

    #[test]
    fn missing_or_empty_display_is_left_alone() {
        assert!(!should_force_wayland_backend(None, Some("wayland"), None));
        assert!(!should_force_wayland_backend(
            None,
            Some("wayland"),
            Some(OsStr::new(""))
        ));
    }
}
