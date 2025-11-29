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
- `rust/src-tauri/target/release/rusty-g6` (release)
- `rust/src-tauri/target/debug/rusty-g6` (debug)

## First Time Setup

### Install Dependencies
```bash
cd rust && npm install
```

### Fix USB Permissions (Linux) - REQUIRED

On Linux, you need to grant permission to access USB devices. Run this **one-time setup**:

```bash
./setup-usb-permissions.sh
```

Then unplug and replug your Sound BlasterX G6.

This creates a udev rule that allows the app (and any built executable) to access the G6 without needing sudo.

**Why?** Linux restricts USB device access for security. The udev rule grants permission specifically for the G6 (VID: 041e, PID: 3256).

## Manual Commands

### Development with Hot Reload
```bash
cd rust && cargo tauri dev
```

### Production Build
```bash
cd rust && cargo tauri build
```

## Prerequisites

- Node.js and npm
- Rust and Cargo
- Tauri CLI (`cargo install tauri-cli`)
- Linux: webkit2gtk, gtk3, libudev-dev, and other system dependencies (scripts will check)

## Troubleshooting

### Permission Denied Error

If you see `Permission denied` when accessing `/dev/hidraw*`:
1. Run `./setup-usb-permissions.sh`
2. Unplug and replug your G6
3. Try again

### App Doesn't Find G6

Make sure your Sound BlasterX G6 is plugged in via USB before starting the app.
