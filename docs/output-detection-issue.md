# Output Detection Issue - Investigation Notes

## Problem Statement

The G6 device toggle commands work correctly, but initial state detection is inconsistent. When the device is physically outputting to Headphones, our software sometimes reads and displays "Speakers" (and vice versa).

## Resolution (2025-12-18)

**Issue Fixed**: The initial state detection logic was using an incorrect read command that returned static configuration data instead of the live device state.

### Root Cause Analysis

1. **Incorrect Read Command**:

   - Old Command: `5a 2c 01 02` (Routing Query)
   - Response: `5a 2c 0a 02 ...`
   - Issue: This response contains static configuration data (likely defaults) and does not reflect real-time physical switches or recent software toggles. It always returned `02` (Speakers) at the checked index in many cases.

2. **Correct Read Command Discovered**:

   - New Command: `5a 2c 01 01` (Output State Query)
   - Response: `5a 2c 05 01 [VALUE] ...`
   - Behavior: This command triggers the device to report its actual current output state using the same event format as asynchronous notifications.

3. **Parsing Logic Update**:
   - The new response format (`0x05`) places the value at index 4.
   - The old response format (`0x0a`) placed values at index 5.
   - We removed the legacy fallback logic that scanned multiple indices, as it was prone to false positives and picking up static data.

### Implementation Details

- **Protocol V2 Updated**:
  - `build_output_config_read()` now sends `5a 2c 01 01`.
  - `G6ResponseParser` now handles `0x2c 0x05` responses.
  - `parse_output_config` explicitly checks index 4 for `0x04` (Headphones) or `0x02` (Speakers).
  - Legacy parsing logic and heuristic scanning loops were removed to ensure reliability.

### Output Verification

- **Headphones**: Value `0x04` at index 4.
- **Speakers**: Value `0x02` at index 4.

## Status

**Fixed**. The software now correctly detects the initial output state on startup by querying the live state directly.

---

**Last Updated**: 2025-12-18
**Status**: Resolved
