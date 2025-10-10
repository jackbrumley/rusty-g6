# Rusty G6

**Cross-platform GUI for SoundBlaster X G6 control**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey)](https://github.com/jackbrumley/rusty-g6)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8DB)](https://tauri.app/)

*A modern, user-friendly graphical interface for controlling your SoundBlaster X G6 audio device across all platforms*

---

## 🎯 Overview

Rusty G6 is a cross-platform GUI application that brings full device control to the [SoundBlaster X G6](https://de.creative.com/p/sound-blaster/sound-blasterx-g6). Creative provides excellent software for Windows, but offers **absolutely no support for Linux or macOS**. While the device functions as a basic USB audio interface on these platforms, all advanced features (surround sound, bass boost, output switching, etc.) are completely inaccessible without third-party tools.

The only existing solution is the community-created [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli) command-line tool. Rusty G6 builds on that pioneering work to provide the first graphical interface for controlling the G6 outside of Windows.

**Why Rusty G6?**
- 🐧 **Linux Support** - The FIRST GUI tool for controlling G6 features on Linux
- 🌐 **Cross-Platform** - Works on Linux, Windows, and macOS with a consistent experience
- 🎨 **User-Friendly** - No command-line knowledge required - just click and adjust
- 🔓 **Open Source** - Free software built on community reverse engineering efforts
- ⚡ **Lightweight** - Fast, native performance with minimal resource usage

## ✨ Features

Rusty G6 provides full control over your SoundBlaster X G6 device settings:

### Audio Output
- **Output Toggle** - Quickly switch between Speakers and Headphones
- **Output Selection** - Directly set the desired output device

### Sound Effects & Processing
- **Surround Sound** - Enable/disable and adjust intensity (0-100)
- **Crystalizer** - Enable/disable and control clarity enhancement (0-100)
- **Bass Boost** - Enable/disable and adjust bass levels (0-100)
- **Smart Volume** - Enable/disable and set volume normalization (0-100)
  - Special presets: Night mode and Loud mode
- **Dialog Plus** - Enable/disable and enhance voice clarity (0-100)

### User Experience
- **Real-time Visual Feedback** - See current settings at a glance
- **Intuitive Controls** - Sliders and toggles for easy adjustments
- **Settings Persistence** - Your preferences are saved automatically
- **Live Updates** - Changes apply immediately to your device

## ⚠️ Important Disclaimer

This software communicates directly with USB hardware. While it has been developed with care and is based on reverse-engineered USB protocols from the [soundblaster-x-g6-cli project](https://github.com/nils-skowasch/soundblaster-x-g6-cli), **USE THIS SOFTWARE AT YOUR OWN RISK**. 

The authors are not responsible for any damages to your system or device. It is recommended to:
- Ensure your device firmware is up to date
- Test settings gradually rather than making extreme changes
- Keep the Windows software available for firmware updates

## 🔧 Firmware Compatibility

This software is designed and tested with SoundBlaster X G6 devices running:
**Firmware version: 2.1.201208.1030**

While it may work with other firmware versions, compatibility is not guaranteed. You can update your firmware using:
- **Windows**: [SoundBlaster Command](https://support.creative.com/Products/ProductDetails.aspx?prodID=21383&prodName=Sound%20Blaster)
- **Linux**: Windows VM with USB passthrough (QEMU/KVM with virt-manager)

## 🚀 Installation

### Prerequisites

#### All Platforms
- SoundBlaster X G6 device
- USB connection to your computer

#### Linux
Install libusb development libraries:
```bash
# Debian/Ubuntu
sudo apt-get install libusb-1.0-0-dev

# Fedora/RHEL
sudo dnf install libusb1-devel

# Arch
sudo pacman -S libusb
```

Create a udev rule to allow user access to the device. Create `/etc/udev/rules.d/50-soundblaster-x-g6.rules` with:
```
SUBSYSTEM=="usb", ATTRS{idVendor}=="041e", ATTRS{idProduct}=="3256", TAG+="uaccess"
```

Apply the rule:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### Windows
No additional setup required. The application will handle USB communication natively.

#### macOS
May require granting USB device access permissions when first running the application.

### Download & Install

Ready-to-use binaries will be available for all supported platforms:

**[📥 Download Latest Release](https://github.com/jackbrumley/rusty-g6/releases/latest)**

- **Linux**: `.deb` package, `.AppImage`, or standalone binary
- **Windows**: `.msi` installer or standalone `.exe`
- **macOS**: `.dmg` disk image

### Building from Source

See [BUILD.md](docs/BUILD.md) for detailed build instructions.

## 💻 Technology Stack

Rusty G6 is built with modern, performant technologies:

- **[Tauri](https://tauri.app/)** - Secure, fast, and lightweight desktop framework
- **[Rust](https://www.rust-lang.org/)** - Systems programming for USB communication and backend logic
- **[React](https://reactjs.org/)** - Modern UI framework for the frontend
- **[rusb/hidapi](https://github.com/libusb/libusb)** - USB Human Interface Device communication

## 🗺️ Roadmap

### Current Status
- ⏳ **Early Development** - Planning and initial implementation phase

### Planned Features

#### Phase 1: Core Functionality (MVP)
- [ ] Basic USB device detection and connection
- [ ] Output toggle (Speakers/Headphones)
- [ ] Simple GUI with basic controls
- [ ] Settings persistence

#### Phase 2: Full Feature Parity
- [ ] All audio effect controls (Surround, Crystalizer, Bass, etc.)
- [ ] Real-time status display
- [ ] Preset management
- [ ] Settings import/export

#### Phase 3: Enhanced Experience
- [ ] System tray integration
- [ ] Keyboard shortcuts
- [ ] Multiple device support
- [ ] Advanced preset system
- [ ] Device firmware information display

#### Phase 4: Polish & Distribution
- [ ] Comprehensive user documentation
- [ ] Platform-specific installers
- [ ] Auto-update functionality
- [ ] Flatpak/Snap/AUR packaging for Linux

## 🎨 Screenshots

*Screenshots will be added as the UI is developed*

## 📚 Technical Details

Rusty G6 uses the USB HID protocol to communicate with the SoundBlaster X G6. The USB protocol implementation is based on reverse engineering work documented in the [soundblaster-x-g6-cli project](https://github.com/nils-skowasch/soundblaster-x-g6-cli).

Key technical specifications:
- **USB Vendor ID**: `041e` (Creative Technology Ltd)
- **USB Product ID**: `3256` (Sound Blaster X G6)
- **Interface Class**: HID (Human Interface Device)
- **Communication Method**: USB Interrupt transfers

For detailed USB protocol information, see the original CLI project's [USB specification documentation](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-spec.txt).

## 🤝 Contributing

Contributions are welcome! Whether it's:

- 🐛 Bug reports and fixes
- ✨ Feature requests and implementations
- 📚 Documentation improvements
- 🎨 UI/UX enhancements
- 🌍 Translations and localization

Please feel free to open issues or submit pull requests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli)** by Nils Skowasch - For the USB protocol reverse engineering work and CLI implementation that made this project possible
- **Creative Technology Ltd** - For the excellent SoundBlaster X G6 hardware
- **Tauri Team** - For the amazing cross-platform framework
- **Rust Community** - For the robust ecosystem and excellent USB libraries

## 📖 Additional Resources

- [SoundBlaster X G6 Official Page](https://de.creative.com/p/sound-blaster/sound-blasterx-g6)
- [Original CLI Project](https://github.com/nils-skowasch/soundblaster-x-g6-cli)
- [USB Protocol Documentation](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)

---

<div align="center">

**Made with ❤️ for the SoundBlaster community**

[Report Bug](https://github.com/jackbrumley/rusty-g6/issues) • [Request Feature](https://github.com/jackbrumley/rusty-g6/issues)

</div>
