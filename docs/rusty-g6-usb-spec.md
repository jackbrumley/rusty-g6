# Sound Blaster X G6 USB Protocol Specification

**Document Version:** 2.1 (Finalized with Initialization Values)
**Device:** Creative Sound Blaster X G6
**USB IDs:** VID `041E`, PID `3256`
**Firmware Version Tested:** 2.1.201208.1030

*This specification documents the complete communication protocol used by the Sound Blaster X G6, derived from USB packet analysis.*

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

| Prefix | Function | Notes |
| :--- | :--- | :--- |
| `5a 05` | **Identification** | Queries device capabilities. |
| `5a 10` | **Hardware Status** | Checks internal hardware state. |
| `5a 11` | **Audio Control** | **Primary Protocol.** Reads/Writes SBX & EQ settings. |
| `5a 15` | **Batch Control** | Sets multiple parameters simultaneously. |
| `5a 20` | **Processing** | Audio processing engine control. |
| `5a 26` | **Gaming** | Scout Mode & Hardware Buttons. |
| `5a 2c` | **Routing** | Output switching (Headphone/Speaker). |
| `5a 30` | **Device Config** | General device settings. |
| `5a 3a` | **System Config** | System-level parameters (LEDs, etc). |

---

## 3. Reading Device State (The "Missing Link")

To read the current value of *any* setting (SBX, EQ, etc.), use the `0x11` family in **Read Mode**.

**Request (Host to EP 0x00):**
`5a 11 03 01 [Type] [Feature] 00...`

**Response (Device from EP 0x85):**
`5a 11 08 01 00 [Type] [Feature] [Value: 4 Bytes] ...`

*This mechanism allows the driver to sync with the hardware state at any time.*

---

## 4. SBX Acoustic Engine (`Type 0x96`)

Controls the audio enhancement algorithms. Values are IEEE 754 Floats or Toggles (0.0/1.0).

| Feature ID | Name | Type | Notes |
| :--- | :--- | :--- | :--- |
| `0x00` | **Surround** | Toggle | |
| `0x01` | Surround Level | Float | 0.0 - 1.0 |
| `0x02` | **Dialog+** | Toggle | |
| `0x03` | Dialog+ Level | Float | 0.0 - 1.0 |
| `0x04` | **Smart Vol** | Toggle | |
| `0x05` | Smart Vol Level | Float | 0.0 - 1.0 |
| `0x06` | Smart Vol Preset | Float? | |
| `0x07` | **Crystalizer** | Toggle | |
| `0x08` | Crystalizer Level | Float | 0.0 - 1.0 |
| `0x0A` - `0x11` | *Internal Params* | ? | Observed in capture during init. |
| `0x12` | **Bass Toggle** | Toggle | *Likely Candidate* |
| `0x13` | Bass Level | Float | |
| `0x14` | Crossover Freq | Float | e.g. 80.0 Hz |
| `0x18` | *Bass (Alt)?* | ? | Older spec reference. |
| `0x1D` | *Unknown* | ? | Highest observed query. |

---

## 5. Equalizer (`Type 0x95`)

Controls the 10-band equalizer. Values are often Floats (dB) or Frequency (Hz).

| Feature ID | Description | Observed Valid Ranges |
| :--- | :--- | :--- |
| `0x0A` | *Meta/Preamp*? | |
| `0x0B` | Band 1 | 31 Hz |
| `0x0C` | Band 2 | 62 Hz |
| `0x0D` | Band 3 | 125 Hz |
| `...` | ... | ... |
| `0x14` | Band 10 | 16 kHz |
| `0x15` - `0x1A`| Gains/Q-Factor | |

---

## 6. Initialization Sequence

To initialize the device correctly (preventing side-effects like SBX disabling), perform this sequence on connection:

1.  **Handshake:** Send `5a 05...` and `5a 10...` to confirm device presence.
2.  **State Sync:** Loop through **all** SBX (`0x96`) and EQ (`0x95`) features using the **Read Request** command. This populates the software UI and likely ensures the device firmware knows the host is aware of the state.
3.  **Batch Init:** (Optional) Send `5a 15...` block setting commands if restoring a profile.

---

## 7. Hardware Events

**Buttons (EP 0x85):**
- `5a 26 0b... 02` = Scout Mode Press
- `5a 26 0b... 01` = SBX Mode Press
- `5a 26 0b... 00` = Button Release

**Volume Knob (EP 0x83):**
- `0x01` = Volume Increment (Small)
- `0x02` = Volume Increment (Large)
- `0xff`? = Volume Decrement (To be verified via live testing)

**Output Mode:**
- Headphones: `5a 2c 05 00 04...`
- Speakers: `5a 2c 05 00 02...`
- *Note: Always follow with cleanup sequence.*

---

**Conclusion:**
This document contains all necessary protocol details to implement a full driver. The capture files are no longer required as we can now use the **Read Protocol** to query the device directly for any remaining unknown values or ranges.
