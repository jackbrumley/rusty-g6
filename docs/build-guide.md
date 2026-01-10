# Build Guide

## Development (Run with Hot Reload)

**Use this for development** - opens the app and restarts automatically when you change files:

```bash
node dev.js
```

This runs `cargo tauri dev` which:
- Opens the application window
- Watches for file changes
- Automatically rebuilds and restarts when you save changes
- Press Ctrl+C to stop

## Production Build

**Use this to create an executable** - builds the final app but doesn't run it:

```bash
node build.js              # Release build (optimized, smaller)
node build.js --dev        # Debug build (faster to build, larger file)
node build.js --clean      # Clean build (removes cache first)
```

This runs `cargo tauri build` which creates standalone executables in:
- `src/src-tauri/target/release/rusty-g6` (release)
- `src/src-tauri/target/debug/rusty-g6` (debug)

## Prerequisites

- Node.js and npm
- Rust and Cargo
- Tauri CLI (`cargo install tauri-cli`)
- Linux: webkit2gtk, gtk3, libudev-dev, and other system dependencies (scripts will check)

## USB Permissions (Linux)

The application automatically prompts for USB permissions on launch if they are missing. You do not need to run any scripts manually.

If you prefer to set them up ahead of time, you can run:
```bash
./setup-usb-permissions.sh
```

## Troubleshooting

### Permission Denied Error

If you see `Permission denied` when accessing `/dev/hidraw*`:
The app should automatically prompt to fix this. If it doesn't, you can manually run:
`./setup-usb-permissions.sh`

### App Doesn't Find G6

Make sure your Sound BlasterX G6 is plugged in via USB before starting the app.
