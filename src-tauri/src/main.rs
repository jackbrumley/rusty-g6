// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "rusty-g6",
    version,
    about = "Control panel for the Sound BlasterX G6"
)]
struct Cli {
    /// Start hidden in the system tray
    #[arg(short, long)]
    start_hidden: bool,

    /// Set by the in-app updater when it relaunches this version because the
    /// install failed
    #[arg(long, hide = true)]
    update_failed: bool,
}

fn main() {
    let cli = Cli::parse();
    rusty_g6_lib::run(
        rusty_g6_lib::LaunchOptions {
            start_hidden: cli.start_hidden,
            update_failed: cli.update_failed,
        },
        launch_shows_window,
    )
}

/// Whether a launch should show the main window: every launch does, except
/// autostart's `--start-hidden`. Arguments that fail to parse still show it.
fn launch_shows_window(args: &[String]) -> bool {
    Cli::try_parse_from(args)
        .map(|cli| !cli.start_hidden)
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::launch_shows_window;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn plain_launch_shows_window() {
        assert!(launch_shows_window(&args(&["rusty-g6"])));
    }

    #[test]
    fn start_hidden_launch_keeps_window_hidden() {
        assert!(!launch_shows_window(&args(&["rusty-g6", "--start-hidden"])));
        assert!(!launch_shows_window(&args(&["rusty-g6", "-s"])));
    }

    #[test]
    fn unparseable_launch_shows_window() {
        assert!(launch_shows_window(&args(&["rusty-g6", "--unknown-flag"])));
    }
}
