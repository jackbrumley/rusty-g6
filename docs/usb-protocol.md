# USB Protocol Implementation

## Authoritative Source

This documentation is based on the reverse engineering work by **Nils Skowasch**:

- **[USB Specification (usb-spec.txt)](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-spec.txt)** - Complete hex command reference
- **[USB Protocol Documentation (usb-protocol.md)](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)** - General USB protocol information
- **[Original CLI Project](https://github.com/nils-skowasch/soundblaster-x-g6-cli)** - Python implementation and additional documentation

**All credit for the protocol reverse engineering goes to Nils Skowasch.** Rusty G6 implements these protocols in Rust for a cross-platform GUI application.

---

## Device Information

- **USB Vendor ID**: `041e` (Creative Technology Ltd)
- **USB Product ID**: `3256` (Sound Blaster X G6)
- **Interface**: Interface 4 (HID)
- **Firmware Version Tested**: 2.1.201208.1030

### Critical Note: Interface Selection

The G6 has 4 USB interfaces (2 Audio, 2 HID). We **must** use Interface 4 - the other interfaces ignore commands!

---

## What Rusty G6 Implements

### ✅ Currently Implemented

#### Output Control
- **Toggle Output** (Speakers ↔ Headphones)
- **Set Output** (Direct selection)

#### Audio Effects
- **Surround Sound** - Toggle + Slider (0-100)
- **Crystalizer** - Toggle + Slider (0-100)
- **Bass Boost** - Toggle + Slider (0-100)
- **Smart Volume** - Toggle + Slider (0-100)
  - Special presets: Night mode, Loud mode
- **Dialog Plus** - Toggle + Slider (0-100)

#### Configuration
- **Settings Persistence** - JSON config file
- **Auto-apply on Connect** - Restores saved settings

### ❌ Not Yet Implemented

- Reading device state (USB protocol doesn't support this)
- Equalizer controls
- Scout Mode
- Audio input controls (beyond microphone setup)
- Profile/preset switching
- Different surround modes (7.1, 5.1, etc.)

---

## Implementation Notes

### Rust Code Structure

Our implementation is organized into three main modules:

1. **`g6_spec.rs`** - Data structures and type definitions
2. **`g6_protocol.rs`** - USB command builders
3. **`g6_device.rs`** - Device manager and state management

### Command Protocol

All commands follow the pattern:
```
DATA command (0x1207) → COMMIT command (0x1103)
```

Each command is 64 bytes with structure:
```
[PREFIX][REQUEST_TYPE][INTERMEDIATE][FEATURE][VALUE][PADDING...]
```

Where:
- `PREFIX`: Always `0x5a`
- `REQUEST_TYPE`: `0x1207` (DATA) or `0x1103` (COMMIT)
- `INTERMEDIATE`: Always `0x0196`
- `FEATURE`: Effect identifier (0x00 = Surround, 0x07 = Crystalizer, etc.)
- `VALUE`: 4-byte little-endian value (toggle or slider)
- `PADDING`: Zeros to reach 64 bytes

### Critical Implementation Detail

Before sending any command, we **must prepend `0x00` as the report_id**:

```rust
let mut data_with_report_id = vec![0x00];
data_with_report_id.extend_from_slice(&command);
device.write(&data_with_report_id)?;
```

Without this, the first byte of the payload is interpreted as the report_id and gets cut off!

### Toggle Values

```rust
const VALUE_ENABLED: u32 = 0x3f800000;  // 1.0f as little-endian
const VALUE_DISABLED: u32 = 0x00000000; // 0.0f
```

### Slider Values

Slider values (0-100) are converted to IEEE 754 floating point (0.0-1.0):

```rust
fn value_to_hex(value: u8) -> u32 {
    let float_val = (value as f32) / 100.0;
    u32::from_le_bytes(float_val.to_le_bytes())
}
```

### Feature Hex Codes

```rust
// Toggles
const FEATURE_SURROUND: u8 = 0x00;
const FEATURE_CRYSTALIZER: u8 = 0x07;
const FEATURE_BASS: u8 = 0x18;
const FEATURE_SMART_VOLUME: u8 = 0x04;
const FEATURE_DIALOG_PLUS: u8 = 0x02;

// Sliders (feature + 1)
const FEATURE_SURROUND_SLIDER: u8 = 0x01;
const FEATURE_CRYSTALIZER_SLIDER: u8 = 0x08;
const FEATURE_BASS_SLIDER: u8 = 0x19;
const FEATURE_SMART_VOLUME_SLIDER: u8 = 0x05;
const FEATURE_DIALOG_PLUS_SLIDER: u8 = 0x03;

// Smart Volume special
const FEATURE_SMART_VOLUME_SPECIAL: u8 = 0x06;
```

---

## State Management

### The Read Problem

The G6's USB protocol **does not support reading device state**. This was confirmed by:
1. No read commands documented in Nils's reverse engineering
2. The original CLI tool uses a temp file to track output state
3. Testing confirms only write operations work

### Our Solution: Config Persistence

We maintain device state in a JSON config file:
- **Location**: `~/.config/rusty-g6/g6_settings.json` (Linux)
- **Format**: JSON serialization of all settings
- **Behavior**: 
  - Save after every setting change
  - Load and apply all settings on device connection
  - Ensures device always matches app state

This approach:
- ✅ Works around the no-read limitation
- ✅ Persists settings across app restarts
- ✅ Allows settings to survive device disconnection
- ✅ Provides a foundation for preset management

---

## Example: Setting Surround Sound

```rust
// 1. Build toggle commands
let toggle_commands = g6_protocol::build_surround_toggle(true);
// Returns: [DATA command with 0x3f800000, COMMIT command]

// 2. Build slider commands  
let slider_commands = g6_protocol::build_surround_slider(75);
// Returns: [DATA command with 75% as float, COMMIT command]

// 3. Send to device
device_manager.send_commands(toggle_commands)?;
device_manager.send_commands(slider_commands)?;

// 4. Update state and save
settings.surround_enabled = EffectState::Enabled;
settings.surround_value = 75;
device_manager.save_settings_to_disk()?;
```

---

## Output Switching

Output switching is more complex - it requires sending ~30 commands to configure various features. These are sent as pre-captured hex sequences from Nils's work:

```rust
pub fn build_output_headphones() -> Vec<Vec<u8>> {
    vec![
        hex_to_bytes("5a2c0500040000..."), // Command 1
        hex_to_bytes("5a2c010100..."),     // Command 2
        // ... 28 more commands
    ]
}
```

We don't fully understand what all these commands do, but they work reliably.

---

## Testing

When testing protocol implementation:

1. **Monitor with `eprintln!`** - We log all USB communication
2. **Check device behavior** - Verify audio changes match commands
3. **Test persistence** - Restart app, verify settings restore
4. **Cross-reference** - Compare our commands with Nils's CLI tool output

---

## Future Protocol Work

Areas for potential expansion:

- **Equalizer Control** - Commands documented but not implemented
- **Scout Mode** - Enhances footstep sounds in games
- **Profile Management** - More sophisticated preset switching
- **Additional Surround Modes** - 5.1, 7.1 configurations
- **Microphone Effects** - Noise reduction, voice modulation

All of these would require referencing Nils's documentation and potentially additional reverse engineering.

---

## References

- [Original USB Spec](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-spec.txt)
- [USB Protocol Docs](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)
- [Python Implementation](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/g6_cli.py)

---

**Remember**: All protocol knowledge comes from Nils Skowasch's reverse engineering work. If you use this documentation, please credit both Rusty G6 **and** the original soundblaster-x-g6-cli project!
