# Sound Blaster X G6 USB Protocol Specification

**Document Version:** 3.2 (Updated with Packet Capture Analysis)
**Device:** Creative Sound Blaster X G6
**USB IDs:** VID `041E`, PID `3256`
**Firmware Versions Tested:** 2.1.201208.1030, 2.1.250903.1324

_This specification documents the complete communication protocol used by the Sound Blaster X G6, derived from USB packet analysis and enhanced by discoveries made during Rusty G6 development._

**What's New in v3.2:**

- ✅ **Gaming Mode Query**: Discovered command to read Scout/SBX state (`5a 26 03 08 ff ff`).
- ✅ **Audio Effect Query**: Corrected read command format (removed erroneous intermediate bytes).
- ✅ **Operation Patterns**: Clarified usage of `01 01`, `03 01`, and `03 08`.

---

## 1. Protocol Architecture

**USB Interface:**

- **Interface 4** is the sole control interface.
- **Endpoint 0x00 (OUT):** Sends commands to the device.
- **Endpoint 0x85 (IN):** Receives responses, data, and button events.
- **Endpoint 0x83 (IN):** Receives volume knob rotation events.

**Packet Structure:**

- All packets are 64 bytes (plus 8-byte USB setup header).
- Commands initiate with a "Magic Byte" `0x5a`.

---

## 2. Protocol Families

The byte following `0x5a` determines the command family:

| Prefix  | Function            | Notes                                                 |
| :------ | :------------------ | :---------------------------------------------------- |
| `5a 05` | **Identification**  | Queries device capabilities.                          |
| `5a 10` | **Hardware Status** | Checks internal hardware state.                       |
| `5a 11` | **Audio Control**   | **Primary Protocol.** Reads/Writes SBX & EQ settings. |
| `5a 15` | **Batch Control**   | Sets multiple parameters simultaneously.              |
| `5a 20` | **Processing**      | Audio processing engine control.                      |
| `5a 26` | **Gaming**          | Scout Mode & Hardware Buttons.                        |
| `5a 2c` | **Routing**         | Output switching (Headphone/Speaker).                 |
| `5a 30` | **Device Config**   | General device settings.                              |
| `5a 3a` | **System Config**   | System-level parameters (LEDs, etc).                  |

---

## 3. Reading Device State

To read the current state of the device, you must use the appropriate command for each family.

### 3.1 Audio Control (`0x11`) - SBX & EQ

To read values like Surround Level or Crystalizer toggle:
**Command:** `5a 11 03 01 [Type] [Feature] ...`

- `03 01` = Read Feature Value Operation
- `[Type]` = `0x96` (SBX Effects) or `0x95` (Equalizer)
- `[Feature]` = Feature ID (see tables below)

**Response:** `5a 11 08 01 00 [Type] [Feature] [Value: 4 Bytes] ...`

- The value is typically an IEEE 754 Float (Little Endian).

### 3.2 Output Routing (`0x2c`)

To read Headphone vs Speaker state:
**Command:** `5a 2c 01 01 ...`

- `01 01` = Live State Query (forces an event packet)

**Response:** `5a 2c 05 01 [State] ...` (Value at index 4)

- `0x04` = Headphones
- `0x02` = Speakers

### 3.3 Gaming Mode (`0x26`)

To read Scout Mode and SBX Master Switch state:
**Command:** `5a 26 03 08 ff ff ...`

- `03 08` = Report Request?
- `ff ff` = "All" or "Query" payload

**Response:** `5a 26 0b 08 ff ff [State] ...`

- `0x00` = Both Off
- `0x01` = SBX On
- `0x02` = Scout Mode On
- `0x03` = Both On

---

## 4. SBX Acoustic Engine (`Type 0x96`)

Controls the audio enhancement algorithms.

| Feature ID | Name              | Type   | Notes     |
| :--------- | :---------------- | :----- | :-------- |
| `0x00`     | **Surround**      | Toggle |           |
| `0x01`     | Surround Level    | Float  | 0.0 - 1.0 |
| `0x02`     | **Dialog+**       | Toggle |           |
| `0x03`     | Dialog+ Level     | Float  | 0.0 - 1.0 |
| `0x04`     | **Smart Vol**     | Toggle |           |
| `0x05`     | Smart Vol Level   | Float  | 0.0 - 1.0 |
| `0x06`     | Smart Vol Preset  | Float? |           |
| `0x07`     | **Crystalizer**   | Toggle |           |
| `0x08`     | Crystalizer Level | Float  | 0.0 - 1.0 |
| `0x18`     | **Bass**          | Toggle |           |
| `0x19`     | Bass Level        | Float  |           |

---

## 5. Equalizer (`Type 0x95`)

Controls the 10-band equalizer.

| Feature ID | Description    | Observed Valid Ranges |
| :--------- | :------------- | :-------------------- |
| `0x0A`     | _Meta/Preamp_? |                       |
| `0x0B`     | Band 1         | 31 Hz                 |
| `0x0C`     | Band 2         | 62 Hz                 |
| `0x0D`     | Band 3         | 125 Hz                |
| `...`      | ...            | ...                   |
| `0x14`     | Band 10        | 16 kHz                |

---

## 6. Initialization Sequence

To synchronize software state with hardware:

1.  **Handshake:** Send `5a 05...` and `5a 10...` to confirm device presence.
2.  **Output Check:** Send `5a 2c 01 01` to determine Speaker/Headphone mode.
3.  **Gaming Mode Check:** Send `5a 26 03 08 ff ff` to check Scout/SBX buttons.
4.  **Audio Sync:** Loop through SBX (`0x96`) and EQ (`0x95`) features using `5a 11 03 01...`.

---

## 7. Formatting Notes

- **ASCII Mode**: Used for Firmware (`5a 07 01 02`).
- **Live State Query**: Used for Routing (`5a 2c 01 01`).
- **Read Value**: Used for Audio (`5a 11 03 01`).

**Conclusion:**
By mixing these different query types appropriately, full bi-directional state synchronization is achieved.
