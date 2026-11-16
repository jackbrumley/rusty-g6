# USB Protocol Implementation

This document describes the G6 protocol as Rusty G6 implements it. For a
broader command catalogue, see [rusty-g6-usb-spec.md](rusty-g6-usb-spec.md).

## Sources

- Reverse engineering from
  [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli).
- The hardware-verified protocol reference and USB captures of
  [linuxblaster_control](https://github.com/RizeCrime/linuxblaster_control)
  (graphic EQ, DAC filter, mic boost read, multi-feature status reports).
- The USB captures of
  [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli)
  (`payloads/raw`: mic processing, decoder, lighting, Direct mode).
- Packet captures and session logs from Rusty G6 development.

## Where It Lives

- `src-tauri/src/g6_protocol/`: the bytes.
  - `mod.rs`: packet framing and the DSP parameter read/write.
  - `effects.rs`, `equalizer.rs`, `dac_filter.rs`, `firmware.rs`: one area
    each.
  - `reads.rs`: state queries and the full state read.
  - `writes.rs`: output, SBX/Scout and mic boost writes.
  - `events.rs`: parses every packet the G6 sends; `DeviceEvent::apply_to`
    is the one mapping from a report to the cached settings.
- `src-tauri/src/g6_device/`: the HID connection, sending and answers
  (`transport.rs`), setters, the state read and the event listener.
- `src-tauri/src/app/commands/device_settings.rs`: the Tauri commands.

## Transport

- USB `041e:3256`, HID interface 4.
- Every packet is 64 bytes: `5a [family] [payload...]`, zero padded. hidapi
  writes get a leading report ID `00`.
- The G6 answers on its interrupt IN endpoint: an acknowledgement
  (`5a 02 ...`) and, for settings, a status report with the new value.

Every write is followed by a read of the same setting, and every answer is
parsed and applied to the cached settings. So the app shows what the device
took, never what was asked for. A write the G6 does not answer is an error.

## DSP Parameters (`0x11` read, `0x12` write)

```text
read:  5a 11 03 01 [family] [id]
write: 5a 12 07 01 [family] [id] [f32 LE]
```

Rusty G6 uses the playback family `0x96`:

| Id | Setting | Value |
|---|---|---|
| `00` / `01` | Surround on / level | 0.0 or 1.0 / 0.0-1.0 |
| `02` / `03` | Dialog+ on / level | |
| `04` / `05` | Smart Volume on / level | |
| `07` / `08` | Crystalizer on / level | |
| `18` / `19` | Bass on / level | |
| `09` | EQ on | 0.0 or 1.0 |
| `0a` | EQ preamp | -6 to +6 dB, step 0.5 |
| `0b`-`14` | EQ bands 31 Hz-16 kHz | -12 to +12 dB, step 0.5 |

Levels are shown as 0-100.

And the recording family `0x95` (mic processing, `g6_protocol/microphone.rs`):

| Id | Setting | Value | Source |
|---|---|---|---|
| `04` / `05` | Noise reduction on / level | 0.0 or 1.0 / 0.0-0.5 (shown 0-100) | captured |
| `00` | Echo cancellation | 0.0 or 1.0 | captured |
| `2c` | Mic smart volume | 0.0 or 1.0 | captured |
| `13` | Mic EQ on | 0.0 or 1.0 | captured |
| `0a` | Voice morph on (`g6_protocol/voice_morph.rs`) | 0.0 or 1.0 | decoded, read on a G6 |
| `0b`-`0d` | Voice morph formants 1-3 | 200-800, 800-1800, 1600-2750 Hz | decoded, read on a G6 |
| `0e`-`12` | Pitch, envelope, pitch variability, quiver, contour | 0.25-4, 0.5-2, -1-5, 0-0.2, 0-0.4 | decoded, read on a G6 |

The G6 has no voice morph preset command: a preset is its eight values,
written one by one (values from Creative's G6 `PresetConfig.json`).

And the decoder family `0x97` (`g6_protocol/decoder.rs`): `02` dynamic range,
2.0 Normal, 1.0 Full, 3.0 Night (captured).

### Status Reports

The G6 reports DSP values as `5a 11 [len] [count] 00` followed by `count`
blocks of `[family] [id] [f32 LE]`, with `len = 2 + count * 6`. A single value
has `count = 1` (`len = 0x08`). On an output switch the G6 pushes the new
output's profile, up to eight blocks per packet. Only family `0x96` is mapped:
families `0x95` (recording) and `0x97` (decoder) reuse the same ids for other
settings.

## Other Families

| Family | Write | Read | Answer |
|---|---|---|---|
| `0x2c` output | `5a 2c 05 00 [02 speakers / 04 headphones]` | `5a 2c 01 01` | `5a 2c 05 01 [output]` |
| `0x26` SBX / Scout | `5a 26 05 07 [01 SBX / 02 Scout] 00 [0/1] 00` | `5a 26 03 08 ff ff` | `5a 26 0b 08 ff ff [bitmask]`: bit 0 SBX, bit 1 Scout |
| `0x3c` mic boost | `5a 3c 04 00 00 02 [dB]` (0, 10, 20, 30) | `5a 3c 02 01 00` | `5a 3c 04 01 00 02 [dB]` |
| `0x6c` DAC filter | `5a 6c 03 00 [filter] 00` | `5a 6c 01 01` | `5a 6c 03 01 [filter] 00` |
| `0x3a` lighting | on/off `5a 3a 02 06 [0/1]`; colour `5a 3a 06 04 00 03 01 00 01` then `5a 3a 09 0a 00 03 01 01 ff [B G R]` | `5a 3a 01 07`; `5a 3a 06 0b 01 00 01 01 01` | `5a 3a 02 07 [0/1]`; `5a 3a 09 0b 01 00 01 01 [A B G R]` |
| `0x07` firmware | | `5a 07 01 02` | `5a 07 [len] [ASCII version]` |

DAC filters: `01` fast roll-off minimum phase, `02` slow roll-off minimum
phase, `03` NOS, `04` fast roll-off linear phase, `05` slow roll-off linear
phase. The G6 lists all five in its filter list (`5a 6c 01 02` answers
`5a 6c 0e 02 05 85 00 01 00 02 00 03 00 04 00 05 00`); the NOS write itself
is not captured.

## Not Implemented Yet

The plan, with the decoded but uncaptured commands, is in
[TODO.md](TODO.md).

## References

- [soundblaster-x-g6-cli](https://github.com/nils-skowasch/soundblaster-x-g6-cli)
- [Upstream USB protocol notes](https://github.com/nils-skowasch/soundblaster-x-g6-cli/blob/main/doc/usb-protocol.md)
- [linuxblaster_control USB protocol reference](https://github.com/RizeCrime/linuxblaster_control/blob/main/UsbProtocol.md)
- [Rusty G6 reference spec](rusty-g6-usb-spec.md)

If this documentation is used elsewhere, please credit Nils Skowasch's
original reverse-engineering work, linuxblaster_control and the Rusty G6
project.
