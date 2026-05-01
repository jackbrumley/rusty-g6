<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Rusty G6 Logo" width="96" height="96" />
</p>

<h1 align="center">Rusty G6 - Sound BlasterX G6 GUI for Linux and Windows</h1>

<p align="center">Simple desktop interface for Sound BlasterX G6 controls, no command line needed.</p>

<p align="center">
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-f1c40f" alt="MIT License" /></a>
  <a href="https://github.com/jackbrumley/rusty-g6"><img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-4f46e5" alt="Platform" /></a>
  <a href="https://tauri.app/"><img src="https://img.shields.io/badge/Built%20With-Tauri-14b8a6" alt="Built with Tauri" /></a>
</p>

<p align="center">
  <a href="https://github.com/jackbrumley/rusty-g6/releases/latest"><img src="https://img.shields.io/badge/Download-Latest%20Release-ef4444?style=for-the-badge" alt="Download Latest Release" /></a>
</p>

Rusty G6 is a Sound BlasterX G6 GUI app for Linux and Windows that lets you control G6 features without command-line tools.

It exists because there is no official Linux client for the G6, and advanced controls are hard to access outside Creative's Windows software. Rusty G6 provides a cleaner desktop experience for Linux users and is also available on Windows.

If you searched for "Sound Blaster G6 GUI Linux" or "Sound Blaster G6 interface on Linux," this is the app you want.

## Install

Download the latest release here:

- **[Download Latest Release](https://github.com/jackbrumley/rusty-g6/releases/latest)**

## What You Can Do

- Tweak your Sound BlasterX G6 audio settings from a simple desktop UI
- Switch output between speakers and headphones
- Adjust SBX effects like Surround, Crystalizer, Bass, Smart Volume, and Dialog Plus
- Use the app on Linux and Windows with the same straightforward controls

## Who This Is For

- Linux users who want a graphical interface for Sound BlasterX G6 settings
- People who do not want to use terminal commands to manage their DAC settings
- Anyone searching for an easy Sound Blaster G6 control panel on desktop

## Screenshots

![Rusty G6 main dashboard on Linux](docs/screenshots/screenshot-1.png)
![Sound BlasterX G6 output and profile controls](docs/screenshots/screenshot-2.png)
![SBX effects controls in Rusty G6 desktop app](docs/screenshots/screenshot-3.png)

### Linux

Linux packages include dependency and USB permission setup.

- `.deb` (Debian/Ubuntu/Mint)
- `.rpm` (Fedora/RHEL/CentOS)
- `.AppImage` (portable Linux)

Install by opening the package from the release page with your distro's package installer.

### Windows

Download and run the `.msi` installer from the latest release.

## FAQ

### Is there a GUI for Sound BlasterX G6 on Linux?

Yes. Rusty G6 is a desktop GUI for Linux that gives you direct control over Sound BlasterX G6 settings.

### How do I control Sound Blaster G6 settings on Linux?

Install Rusty G6 from the latest release and use the app to switch outputs and adjust SBX settings with a normal graphical interface.

## Why This Project Exists

There is no official Linux client for the G6. On Linux, the device works as a basic audio interface, but advanced controls are not easily available.

Rusty G6 builds on the reverse-engineering work from [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli) and provides a user-friendly desktop interface.

## Build From Source

```bash
git clone https://github.com/jackbrumley/rusty-g6
cd rusty-g6
npm install
npm run tauri:build
```

For more detail, see [docs/build-guide.md](docs/build-guide.md).

## Technical Notes

- USB Vendor ID: `041e` (Creative Technology Ltd)
- USB Product ID: `3256` (Sound BlasterX G6)
- Protocol: USB HID with interrupt transfers

Documentation:

- [USB Protocol Implementation](docs/usb-protocol.md)
- [Reference USB Specification](docs/rusty-g6-usb-spec.md)

## Disclaimer

This software communicates directly with USB hardware using reverse-engineered protocols. Use at your own risk.

## Credits

Special thanks to **[Nils Skowasch](https://github.com/nils-skowasch)** and the [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli) project for the protocol reverse engineering foundation.

## License

MIT - see [LICENSE](LICENSE).
