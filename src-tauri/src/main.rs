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
    /// Set by the in-app updater when it relaunches this version because the
    /// install failed
    #[arg(long, hide = true)]
    update_failed: bool,
}

fn main() {
    let cli = Cli::parse();
    rusty_g6_lib::run(rusty_g6_lib::LaunchOptions {
        update_failed: cli.update_failed,
    })
}
