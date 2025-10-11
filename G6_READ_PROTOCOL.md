# G6 Read Protocol - Research & Implementation

## Status: 🔨 IN PROGRESS - Phase 1 Complete

**Phase 1 DONE:** Discovered read commands via USB capture  
**Phase 2 TODO:** Actually read data from device and parse responses  
**Phase 3 TODO:** Map response data to device settings  
**Phase 4 TODO:** Integrate into app UI

## Current Progress

- [x] Capture USB traffic from Creative software
- [x] Extract and analyze HID commands
- [x] Identify 7 read commands
- [x] Implement command builders in Rust
- [x] Add read_device_state() function to g6_device.rs
- [x] Add read_full_device_state() function
- [x] Expose via Tauri commands (read_device_state, read_full_device_state)
- [x] Create Debug tab in UI with buttons to test read commands
- [ ] **TEST: Connect device and click "Read All Commands" button in Debug tab**
- [ ] **NEXT: Analyze the hex responses to figure out what each byte means**
- [ ] Parse response bytes to actual settings
- [ ] Create state parsing functions
- [ ] Auto-read on connect to populate UI with real values

## Discovered Commands

All commands use format: `0x5a [CMD] [PARAMS...]` (padded to 64 bytes)

| Command | Parameters | Notes |
|---------|-----------|-------|
| **0x05** | None | Main status query (HIGH PRIORITY) |
| 0x10 | None | Unknown |
| 0x15 | 0x01, 0x00 | Unknown |
| 0x20 | None | Unknown |
| 0x30 | None | Unknown |
| 0x39 | 0x01, 0x04 | Unknown |
| **0x3a** | 0x02, 0x09 OR 0x01, 0x07 | Parameterized query (HIGH PRIORITY) |

## Startup Sequence

Creative software sends these commands when opening:
```
1. 0x05           - Status query
2. 0x10           - Query
3. 0x20           - Query
4. 0x30           - Query
5. 0x15 (0x01)    - Query with param
6. 0x3a (0x02 09) - Parameterized
7. 0x05           - Status query (repeat)
8. 0x39 (0x01 04) - Query with param
9. 0x3a (0x01 07) - Different param
```

## Rust Implementation

Added to `rust/src-tauri/src/g6_protocol.rs`:

```rust
// Constants
const CMD_STATUS_QUERY: u8 = 0x05;
const CMD_QUERY_10: u8 = 0x10;
const CMD_QUERY_15: u8 = 0x15;
const CMD_QUERY_20: u8 = 0x20;
const CMD_QUERY_30: u8 = 0x30;
const CMD_QUERY_39: u8 = 0x39;
const CMD_QUERY_3A: u8 = 0x3a;

// Functions
build_status_query()          // 0x05
build_query_10()              // 0x10
build_query_15()              // 0x15 with params
build_query_20()              // 0x20
build_query_30()              // 0x30
build_query_39()              // 0x39 with params
build_query_3a_variant1()     // 0x3a variant 1
build_query_3a_variant2()     // 0x3a variant 2
build_full_device_read_sequence() // All commands in order
```

## Next Steps

### To Use These Commands:
```rust
// 1. Send command
let cmd = build_status_query();
device.send_feature_report(&cmd)?;

// 2. Read response
let mut response = [0u8; 64];
device.get_feature_report(&mut response)?;

// 3. Parse response (TODO - needs response analysis)
```

### Still TODO:
1. **Capture device RESPONSES** - We only have commands, not replies
2. **Parse response bytes** - Figure out what each byte means
3. **Map to settings** - Which bytes = surround enabled, bass level, etc.
4. **Integrate in app** - Call on connect to populate UI

## How to Capture Responses

In Wireshark:
1. Filter: `usb.src == "3.3.5"` (device to host)
2. Look for URB_INTERRUPT IN after each SET_REPORT
3. Copy hex data from those packets
4. Correlate with which command was sent

## Files

**Modified:**
- `rust/src-tauri/src/g6_protocol.rs` - Added read command builders

**Captured Data:**
- `packet-capture/sound-blasterx-g6-get-device-status.pcapng` - Full capture
- `packet-capture/set-report-samples.txt` - Extracted command hex
- `packet-capture/g6-device-info.csv` - Packet list

## Discovery Method

1. Opened Wireshark, started capture on USBPcap3
2. Opened Creative Control Panel software
3. Stopped capture after ~10 seconds
4. Exported SET_REPORT packets
5. Analyzed hex data to find command patterns
6. Implemented in Rust

---

**Date:** 2025-01-10  
**Duration:** ~42 minutes  
**Result:** 7 new commands discovered and implemented ✅
