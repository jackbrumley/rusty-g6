# USB Protocol Implementation

This document reflects the protocol behavior currently implemented in Rusty G6.

## Sources and Scope

Rusty G6 protocol work is based on:

- reverse engineering from [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli)
- the hardware-verified protocol reference and USB captures of
  [linuxblaster_control](https://github.com/RizeCrime/linuxblaster_control)
  (graphic EQ, multi-feature status reports)
- additional packet-capture validation from Rusty G6 development
- current implementation in:
  - `src-tauri/src/g6_protocol_v2/` (framing, `reads.rs`, `writes.rs`,
    `equalizer.rs`, `parser.rs`, `events.rs`)
  - `src-tauri/src/g6_device/` (connection, transport, setters, state read,
    event listener)
  - `src-tauri/src/app/commands/audio.rs` and `equalizer.rs`

For a broader command catalog/reference, see `rusty-g6-usb-spec.md`.

---

## Device Baseline

- USB Vendor ID: `041e`
- USB Product ID: `3256`
- Control interface: HID Interface 4
- Packet payload size used by command builders: 64 bytes

Important: commands are written with report id prefix `0x00` before the 64-byte payload.

---

## Implemented in Rusty G6

### Output Routing (`0x2c`)

- Set output (`Speakers`/`Headphones`)
- Commit output
- Toggle output helper

Typical sequence:

```text
5a 2c 05 00 [OUTPUT]
5a 2c 01 01
```

Output values:

- `0x02` = Speakers
- `0x04` = Headphones

### Audio Effects

Implemented controls:

- Surround
- Crystalizer
- Bass (toggle command + local value state)
- Smart Volume
- Dialog Plus

Core read format used by builders:

```text
5a 11 03 01 96 [FEATURE]
```

Core write format used for most effect toggles/values:

```text
5a 12 07 01 96 [FEATURE] [FLOAT]
```

### Graphic EQ (family `0x96`)

Implemented and exposed in the Output page.

- `0x09` EQ toggle (f32 `0.0` / `1.0`)
- `0x0a` preamp, -6 to +6 dB
- `0x0b`-`0x14` bands 31, 62, 125, 250, 500 Hz and 1, 2, 4, 8, 16 kHz,
  -12 to +12 dB

Gains are f32 dB values in steps of 0.5 dB, written with the core write
format above and read back with the core read format.

### Status Reports (`0x11`)

The G6 reports DSP values as `5a 11 [len] [count] 00` followed by `count`
blocks of `[family] [feature] [f32 LE]`, with `len = 2 + count * 6`. A single
value has `count = 1` (`len = 0x08`); on an output switch the G6 pushes the
new output's whole profile, up to eight blocks per packet. Only family `0x96`
(playback) is mapped to settings: families `0x95` (recording) and `0x97`
(decoder) reuse the same feature numbers for other settings.

### Gaming Modes (`0x26`)

Implemented:

- SBX mode set
- Scout mode set
- mode read query helper

Set operations use data + commit style builder flow.

### Microphone Boost (`0x3c`)

Implemented and exposed in UI/commands.

Allowed values:

- `0`, `10`, `20`, `30` dB

Builder uses:

- data command (`0x3c 04 ...`)
- commit command (`0x3c 02 01`)

---

## Read and Sync Behavior

Rusty G6 performs device synchronization via:

1. explicit read command set in `build_read_all_state_commands()`
2. event listener updates in `g6_device/listener.rs`; both apply events
   through `DeviceEvent::apply_to`

Events parsed include output changes, gaming mode state, audio effect toggles/values, digital filter reports, audio config reports, and microphone boost events.

Note: Rusty G6 keeps runtime device state in memory. This is not currently persisted as a dedicated JSON settings file in the protocol layer.

---

## Feature IDs Used

Audio effect features used by implementation:

- `0x00` Surround Toggle
- `0x01` Surround Value
- `0x02` Dialog Plus Toggle
- `0x03` Dialog Plus Value
- `0x04` Smart Volume Toggle
- `0x05` Smart Volume Value
- `0x06` Smart Volume Preset (read path)
- `0x07` Crystalizer Toggle
- `0x08` Crystalizer Value
- `0x09` EQ Toggle
- `0x0a` EQ Preamp
- `0x0b`-`0x14` EQ Bands
- `0x18` Bass Toggle
- `0x19` Bass Value

Gaming feature IDs:

- `0x01` SBX mode
- `0x02` Scout mode

---

## Not Fully Productized

- Digital filter: reports are parsed, but there is no write builder or
  command yet.
- Extended profile/preset workflows are not implemented as first-class user flows.

---

## References

- [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli)
- [Upstream USB protocol notes](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)
- [linuxblaster_control USB protocol reference](https://github.com/RizeCrime/linuxblaster_control/blob/main/UsbProtocol.md)
- [Rusty G6 reference spec](rusty-g6-usb-spec.md)

If this documentation is used elsewhere, please credit both Nils Skowasch's original reverse-engineering work and the Rusty G6 project.
