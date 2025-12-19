# USB Protocol Implementation

## Authoritative Sources

This documentation is based on multiple sources:

1. **Original reverse engineering by Nils Skowasch**:

   - **[USB Specification (usb-spec.txt)](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-spec.txt)** - Complete hex command reference
   - **[USB Protocol Documentation (usb-protocol.md)](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)** - General USB protocol information
   - **[Original CLI Project](https://github.com/nils-skowasch/soundblaster-x-g6-cli)** - Python implementation

2. **Our own USB packet capture analysis**:
   - Used Wireshark + USBPcap to capture actual traffic from Creative's official software
   - Discovered correct command formats that differ from initial reverse engineering
   - Validated event broadcast system for device state synchronization

**Credit**: Protocol foundation by Nils Skowasch. Command format corrections via packet capture analysis by Rusty G6 team.

---

## Device Information

- **USB Vendor ID**: `041e` (Creative Technology Ltd)
- **USB Product ID**: `3256` (Sound Blaster X G6)
- **Interface**: Interface 4 (HID)
- **Firmware Version Tested**: 2.1.250903.1324

### Critical Note: Interface Selection

The G6 has 4 USB interfaces (2 Audio, 2 HID). We **must** use Interface 4 - the other interfaces ignore commands!

---

## What Rusty G6 Implements

### ✅ Currently Implemented

#### Output Control

- **Toggle Output** (Speakers ↔ Headphones)
- **Set Output** (Direct selection)

#### Audio Effects (All Working!)

- **Surround Sound** - Toggle + Slider (0-100)
- **Crystalizer** - Toggle + Slider (0-100)
- **Bass Boost** - Toggle + Slider (0-100)
- **Smart Volume** - Toggle + Slider (0-100)
- **Dialog Plus** - Toggle + Slider (0-100)

#### Gaming Modes

- **SBX Mode** - Master on/off switch for all effects
- **Scout Mode** - Enhances footstep/environmental sounds

#### Device State

- **Event Listener** - Real-time monitoring of device state changes
- **State Synchronization** - Detects changes from official software
- **Settings Persistence** - JSON config file

### ❌ Not Yet Implemented

- Equalizer controls
- Different surround modes (7.1, 5.1, etc.)
- Digital filter settings
- Audio input controls
- Profile/preset switching (beyond basic save/load)

---

## Critical Discovery: Correct Command Format

### ⚠️ IMPORTANT: Previous Documentation Was Incorrect

Through USB packet capture analysis, we discovered the **actual** command format used by Creative's official software differs from initial reverse engineering:

### Audio Effect Commands (CORRECTED)

**Format**: `DATA (0x12 0x07) + READ (0x11 0x03)`

```
Write Command:  5a 12 07 01 96 [FEATURE] [VALUE_BYTES] 00 00...
Read Command:   5a 11 03 01 96 [FEATURE] 00 00...
```

**Structure**:

- `5a` - Prefix (all commands)
- `12` - **Command Family: DataControl** (NOT 0x11 AudioControl!)
- `07` - **Operation: Write** (NOT 0x08 Report!)
- `01 96` - Intermediate bytes (Audio type 0x96)
- `[FEATURE]` - Effect identifier (0x00, 0x07, 0x18, etc.)
- `[VALUE_BYTES]` - 4-byte little-endian IEEE 754 float
- Padding to 64 bytes

**Why This Matters**:

- Using the old format (0x11 0x08) sends commands but device doesn't acknowledge them
- The device broadcasts events in 0x11 0x08 format, which confused initial reverse engineering
- We were trying to send what we observed the device broadcasting, not what it expects

### Output Switching Commands

Output switching uses a different format:

```
Set Command:    5a 2c 05 00 [OUTPUT_VALUE] 00 00...
Commit Command: 5a 2c 01 01 00 00...
```

Where `OUTPUT_VALUE`:

- `0x02` = Speakers
- `0x04` = Headphones

---

## Implementation Notes

### Rust Code Structure

Our implementation uses Protocol V2 (cleaner architecture):

1. **`g6_spec.rs`** - Data structures and type definitions
2. **`g6_protocol_v2.rs`** - USB command builders (CORRECTED FORMATS)
3. **`g6_device.rs`** - Device manager, event listener, state management

### Command Families (From Packet Capture)

```rust
enum CommandFamily {
    Unknown02 = 0x02,      // Appears before some commands (purpose unclear)
    Identification = 0x05,
    FirmwareQuery = 0x07,
    HardwareStatus = 0x10,
    AudioControl = 0x11,   // Device REPORTS/EVENTS (RX only)
    DataControl = 0x12,    // WRITE commands (TX)
    BatchControl = 0x15,
    Processing = 0x20,
    Gaming = 0x26,         // SBX/Scout Mode
    Routing = 0x2c,        // Output switching
    DeviceConfig = 0x30,
    SystemConfig = 0x3a,
    AudioConfig = 0x3c,
    DigitalFilter = 0x6c,
}
```

### Critical Implementation Detail

Before sending any command, we **must prepend `0x00` as the report_id**:

```rust
let mut data_with_report_id = vec![0x00];
data_with_report_id.extend_from_slice(&command);
device.write(&data_with_report_id)?;
```

Without this, the first byte of payload gets interpreted as report_id and is cut off!

### Toggle Values

```rust
const VALUE_ENABLED: f32 = 1.0;
const VALUE_DISABLED: f32 = 0.0;

// Converted to IEEE 754 little-endian:
// 1.0f = 0x3f800000
// 0.0f = 0x00000000
```

### Slider Values

Slider values (0-100) are converted to IEEE 754 floating point (0.0-1.0):

```rust
let float_val = (value as f32) / 100.0;
let bytes = float_val.to_le_bytes();
```

### Feature Hex Codes

```rust
// Audio Effect Features
const FEATURE_SURROUND_TOGGLE: u8 = 0x00;
const FEATURE_SURROUND_VALUE: u8 = 0x01;
const FEATURE_DIALOG_PLUS_TOGGLE: u8 = 0x02;
const FEATURE_DIALOG_PLUS_VALUE: u8 = 0x03;
const FEATURE_SMART_VOLUME_TOGGLE: u8 = 0x04;
const FEATURE_SMART_VOLUME_VALUE: u8 = 0x05;
const FEATURE_SMART_VOLUME_PRESET: u8 = 0x06;
const FEATURE_CRYSTALIZER_TOGGLE: u8 = 0x07;
const FEATURE_CRYSTALIZER_VALUE: u8 = 0x08;
const FEATURE_BASS_TOGGLE: u8 = 0x18;
const FEATURE_BASS_VALUE: u8 = 0x19;

// Gaming Mode Features (0x26 family)
const FEATURE_SBX_MODE: u8 = 0x01;
const FEATURE_SCOUT_MODE: u8 = 0x02;
```

---

## Device State & Event System

### ✅ NEW: Device State CAN Be Read!

Contrary to earlier documentation, the G6 **does broadcast device state**:

1. **Event Listener Thread**: Continuously monitors USB endpoint for device broadcasts
2. **Event Format**: Device sends `0x11 0x08` packets when state changes
3. **Synchronization**: Works even when official software makes changes

### Event Broadcast Format

```
Device Event: 5a 11 08 01 00 96 [FEATURE] 00 [VALUE] 00 00...
```

This is why we initially thought `0x11 0x08` was the command format - we were seeing device responses, not commands!

### Event Parser

```rust
pub enum DeviceEvent {
    // Output
    OutputChanged(OutputDevice),

    // Gaming modes
    SbxModeChanged(EffectState),
    ScoutModeChanged(ScoutModeState),

    // Audio effect toggles
    SurroundToggled(EffectState),
    CrystalizerToggled(EffectState),
    BassToggled(EffectState),
    SmartVolumeToggled(EffectState),
    DialogPlusToggled(EffectState),

    // Audio effect values
    SurroundValueChanged(u8),
    CrystalizerValueChanged(u8),
    BassValueChanged(u8),
    SmartVolumeValueChanged(u8),
    DialogPlusValueChanged(u8),

    // Other
    DigitalFilterChanged(DigitalFilter),
    AudioConfigChanged(AudioConfig),
}
```

### State Management Approach

1. **On Startup**: Read complete device state
2. **On Event**: Update internal state from broadcasts
3. **On Change**: Send command, wait for event confirmation
4. **Persistence**: Save to JSON config file

This provides:

- ✅ Real-time synchronization with official software
- ✅ Accurate state regardless of how device was configured
- ✅ Persistence across app restarts
- ✅ Foundation for preset management

---

## Example: Setting Bass (CORRECTED)

```rust
// Build bass toggle command (NEW FORMAT)
pub fn build_set_bass_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        // DATA command - 5a 12 07 01 96 18 [FLOAT]
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_BASS_TOGGLE])
            .float_value(value)
            .build(),
        // READ command - 5a 11 03 01 96 18
        build_audio_effect_read(FEATURE_BASS_TOGGLE),
    ]
}

// Send to device
device_manager.send_commands(commands)?;

// Event listener will detect device broadcast:
// 5a 11 08 01 00 96 18 00 00 80 3f... (bass enabled)
```

---

## Debugging Tools

### Enhanced Packet Logging

We added microsecond-precision timestamps to all USB traffic:

```
[TX] 5a1207019618...                    | Data Control
[RX-ACK] 5a020a11830...                 | Unknown Command/Report (Type 02)
[RX-EVENT @1766144208055.157µs] 5a11... | Audio Control
```

This helped identify:

- Command/response patterns
- Event broadcast timing
- Acknowledgment vs state broadcast packets

### USB Packet Capture Process

1. **Tool**: Wireshark + USBPcap (Windows) or usbmon (Linux)
2. **Method**:
   - Run official Creative software
   - Capture USB traffic while changing settings
   - Filter for Interface 4, HID protocol
   - Analyze SET_REPORT requests (outgoing commands)
3. **Result**: Discovered actual command formats vs device broadcasts

---

## Testing Methodology

When implementing protocol changes:

1. **Capture First**: Use Wireshark to see what official software actually sends
2. **Implement**: Code the exact byte sequences observed
3. **Verify**: Test that device responds as expected
4. **Listen**: Confirm event broadcasts match expected state
5. **Cross-test**: Change setting in official software, verify our app detects it

---

## Future Protocol Work

Areas for potential expansion:

- **Equalizer Control** - Commands documented but need testing
- **Digital Filter Settings** - Read working, write format needs discovery
- **Profile Management** - Enhanced preset switching
- **Additional Surround Modes** - 5.1, 7.1 configurations
- **Microphone Effects** - Noise reduction, voice modulation

All of these would require additional packet capture analysis.

---

## References

### Original Research

- [Nils Skowasch's USB Spec](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-spec.txt)
- [Nils Skowasch's Protocol Docs](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)
- [Python CLI Implementation](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/g6_cli.py)

### Our Discoveries

- USB packet captures (see `packet-capture/` directory)
- Protocol V2 implementation (`rust/src-tauri/src/g6_protocol_v2.rs`)
- Event parser implementation (`rust/src-tauri/src/g6_device.rs`)

---

## Acknowledgments

- **Nils Skowasch** - Original USB protocol reverse engineering
- **Rusty G6 Team** - Packet capture analysis, command format corrections, event system discovery

**Remember**: If you use this documentation, please credit both projects!
