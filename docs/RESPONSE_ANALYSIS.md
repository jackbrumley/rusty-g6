# G6 Device Response Analysis

## Test Results - First Capture

Successfully read responses from device! Here's what we got:

### Command Responses (Hex Format)

```
0x05_status:        5a 05 04 1f 00 00 00 00 00 00 00 00 00 00 00 00
0x10:               5a 10 08 ef 67 74 00 00 00 00 00 00 00 00 00 00
0x20:               5a 20 04 97 00 00 00 00 00 00 00 00 00 00 00 00
0x30:               5a 30 04 30 01 10 00 00 00 00 00 00 00 00 00 00
0x15:               5a 15 2c 00 03 96 0a 00 00 c0 40 00 00 c0 c0 00
0x3a (02 09):       5a 3a 03 09 00 03 00 00 00 00 00 00 00 00 00 00
0x05_status_repeat: 5a 05 04 1f 00 00 00 00 00 00 00 00 00 00 00 00
0x39 (01 04):       5a 02 0a 39 81 00 00 00 00 00 00 00 00 00 00 00
0x3a (01 07):       5a 3a 02 07 01 00 00 00 00 00 00 00 00 00 00 00
```

## Pattern Analysis

All responses start with `5a` followed by the command byte - this echoes our command format.

### Response Structure Pattern
```
Byte 0: 0x5a (prefix - same as command)
Byte 1: Command byte (echoed back)
Byte 2: Appears to be length or count (varies: 04, 08, 2c, 03, 02, 0a)
Byte 3+: Data payload
```

### Notable Patterns

**0x05 Status (appears twice, identical):**
- `5a 05 04 1f` - Consistent response
- Byte 2: `04` might mean 4 bytes of data follow
- Byte 3: `1f` (31 decimal) - Status flags?

**0x15 Response (Longest):**
- `5a 15 2c` - Byte 2 is `2c` (44 decimal) - much longer response
- Contains interesting bytes: `96 0a`, `c0 40`, `c0 c0`
- This might contain multiple settings

**0x3a Variants:**
- `0x3a (02 09)`: Response `5a 3a 03 09 00 03`
- `0x3a (01 07)`: Response `5a 3a 02 07 01`
- Different parameters give different responses!

## Next Steps to Decode

### TEST 1: Toggle Output
1. Note current response from 0x05
2. Toggle output (headphones <-> speakers) in Main tab
3. Read 0x05 again
4. **Compare**: Which byte changed?

### TEST 2: Change Bass Setting
1. Read full state
2. Change bass level (e.g., 0 -> 100)
3. Read full state again
4. **Compare**: Which bytes changed?

### TEST 3: Enable/Disable Effects
1. Read full state
2. Toggle surround ON
3. Read full state
4. **Compare**: Look for 00 -> 01 or similar flag changes

### TEST 4: Systematic Effect Testing
For each effect (Surround, Crystalizer, Bass, Smart Volume, Dialog Plus):
1. Set to OFF, value 0
2. Read full state - record responses
3. Set to ON, value 50
4. Read full state - compare
5. Set value to 100
6. Read full state - compare

## Hypothesis Testing

### Possible Byte Meanings (To Test)

**0x05 Status - Byte 3 (0x1f = 31 decimal = 0b00011111):**
- Might be bit flags for enabled/disabled states
- Bit 0: Surround?
- Bit 1: Crystalizer?
- Bit 2: Bass?
- Bit 3: Smart Volume?
- Bit 4: Dialog Plus?
- **Test**: Toggle each effect, see if bits change

**0x15 Response - Bytes 9-10 (c0 40, c0 c0):**
- These look like IEEE 754 float values!
- `40 c0 00 00` reversed = could be 6.0 or similar
- Might be effect levels (0.0 to 1.0 range)?

**0x30 - Byte 4-5 (01 10):**
- `01` might be a flag
- `10` (16 decimal) - unknown purpose

## Data Collection Template

When testing, record:
```
Setting Changed: [e.g., "Bass: OFF->ON", "Bass Level: 0->50"]
Command: [e.g., "0x05", "0x15"]
Before: [hex response]
After:  [hex response]
Diff:   [which bytes changed]
```

## Test 1: After Output Toggle

User toggled output between speakers/headphones. Here are the NEW responses:

```
0x05_status:        5a 2c 05 01 02 00 00 00 00 00 00 00 00 00 00 00
0x10:               5a 11 32 08 00 96 01 00 00 00 3f 96 03 00 00 00
0x20:               5a 11 32 08 00 96 0b 00 00 00 00 96 0c 00 00 00
0x30:               5a 11 32 08 00 96 13 00 00 00 00 96 14 00 00 00
0x15:               5a 11 32 08 00 95 00 00 00 00 00 95 04 00 00 00
0x3a (02 09):       5a 11 32 08 00 95 0f 00 00 80 3f 95 10 00 00 80
0x05_status_repeat: 5a 11 32 08 00 95 17 00 00 00 40 95 18 00 00 00
0x39 (01 04):       5a 11 32 08 00 95 1f 00 00 7a 43 95 20 00 00 fa
0x3a (01 07):       5a 02 0a 2c 00 00 02 00 00 00 00 00 00 00 00 00
```

### 🚨 MAJOR DISCOVERY

The responses are COMPLETELY different! Two possibilities:

**Theory 1: Device is in different state**
- The output toggle command might have put the device in a different mode
- Most responses now start with `5a 11 32` instead of echoing the command

**Theory 2: Async response issue**
- Maybe we're reading responses that don't match the commands we sent
- The device might be sending unsolicited data
- Timing issue with read/write

### Comparison Table

| Command | First Response | Second Response | Changed? |
|---------|---------------|-----------------|----------|
| 0x05    | `5a 05 04 1f` | `5a 2c 05 01 02` | ✅ COMPLETELY DIFFERENT |
| 0x10    | `5a 10 08 ef 67 74` | `5a 11 32 08 00 96 01` | ✅ DIFFERENT |
| 0x20    | `5a 20 04 97` | `5a 11 32 08 00 96 0b` | ✅ DIFFERENT |
| 0x15    | `5a 15 2c 00 03 96 0a` | `5a 11 32 08 00 95 00` | ✅ DIFFERENT |
| 0x3a v2 | `5a 3a 02 07 01` | `5a 02 0a 2c 00 00 02` | ✅ DIFFERENT |

### Key Observations

1. **First capture**: Responses echo command byte (byte 1)
2. **Second capture**: Most have `11 32` or `2c` at bytes 1-2
3. **Pattern break**: Something fundamental changed

### Hypothesis: Multiple Read Methods?

Maybe we need to:
1. Clear the read buffer before each read?
2. Use a different HID read method?
3. Send commands in a specific order/timing?

## Test 2: After Disconnect/Reconnect

User disconnected and reconnected device. Responses are now IDENTICAL to first test:

```
0x05_status:        5a 05 04 1f 00 00 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x10:               5a 10 08 ef 67 74 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x20:               5a 20 04 97 00 00 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x30:               5a 30 04 30 01 10 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x15:               5a 15 2c 00 03 96 0a 00 00 c0 40 00 00 c0 c0 00  ✅ SAME
0x3a (02 09):       5a 3a 03 09 00 03 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x05_status_repeat: 5a 05 04 1f 00 00 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x39 (01 04):       5a 02 0a 39 81 00 00 00 00 00 00 00 00 00 00 00  ✅ SAME
0x3a (01 07):       5a 3a 02 07 01 00 00 00 00 00 00 00 00 00 00 00  ✅ SAME
```

### ✅ CONCLUSION

**The read mechanism is working correctly!** The strange responses in Test 1 were likely because:

1. **Our output toggle command worked** - it changed device state
2. **But the device got confused** - the command sequence we send might not be the complete/correct one
3. **After reconnect** - device resets to default state, responses are consistent again

This means:
- ✅ Read commands work
- ✅ Responses are deterministic
- ⚠️ Our WRITE commands might need investigation (output toggle sent incomplete sequence?)

## Next Investigation

We should test output toggle using the OFFICIAL Creative software:
1. Use Creative app to toggle output
2. Read state with our app
3. Compare responses before/after official toggle

This will show us what a "proper" output toggle looks like in the device responses.

## Test 3: After Official Creative Software Change

User changed output from headphones to speakers using Creative Control Panel.

### BEFORE Creative software (device in clean state):
```
0x05:    5a 11 32 08 00 95 1f 00 00 7a 43 95 20 00 00 fa
0x20:    5a 2c 05 01 04 00 00 00 00 00 00 00 00 00 00 00
0x3a_v2: 5a 11 14 03 00 96 13 00 00 00 00 96 14 00 00 00
```

### AFTER Creative software changed setting:
```
0x05:    00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ❌ ALL ZEROS
0x10:    5a 2c 05 01 02 00 00 00 00 00 00 00 00 00 00 00
0x20:    5a 11 32 08 00 96 0b 00 00 00 00 96 0c 00 00 00
0x30:    5a 26 0b 08 ff ff 00 00 00 00 00 00 00 00 00 00  ❌ GARBAGE
0x15:    5a 26 0b 08 ff ff 00 00 00 00 00 00 00 00 00 00  ❌ GARBAGE
0x3a_v1: 5a 26 0b 08 ff ff 00 00 00 00 00 00 00 00 00 00  ❌ GARBAGE
```

### 🚨 CRITICAL ISSUE DISCOVERED

After Creative software changes settings, the device enters an **unstable state**:
- First read returns all zeros
- Subsequent reads return garbage/inconsistent data
- Many responses show `5a 26 0b 08 ff ff` pattern (error response?)
- Reads don't match commands sent

### ROOT CAUSE ANALYSIS

The problem is likely:

1. **Missing initialization sequence** - We might need to send specific commands before reading
2. **Device mode confusion** - Creative software might put device in different communication mode
3. **Async notifications** - Device might be sending unsolicited data we need to clear
4. **Feature Report vs Interrupt** - We might be using wrong HID transfer type

### Comparing to Python Reference

Looking back at reference code, it uses `get_feature_report()` not `read()`:
```python
# Reference code probably uses:
device.get_feature_report(report_id, size)
```

We're using:
```rust
// Our code uses:
device.read_timeout(&mut response, 1000)  // Interrupt endpoint
```

**HYPOTHESIS**: We should use `get_feature_report()` instead of `read()` for reading responses!

## Test 4: Official Creative Software Output Change

User changed output from **Speakers** to **Headphones** using official Creative Control Panel.

### BEFORE Change (Speakers):
```
0x05:    5a 05 04 1f 00 00 00 00 00 00 00 00 00 00 00 00
0x10:    5a 10 08 ef 67 74 00 00 00 00 00 00 00 00 00 00
0x20:    5a 20 04 97 00 00 00 00 00 00 00 00 00 00 00 00
0x3a_v2: 5a 3a 02 07 01 00 00 00 00 00 00 00 00 00 00 00
```

### AFTER Change (Headphones):
```
0x05:    5a 11 32 08 00 95 27 00 c0 30 43 95 28 00 c0 b0  ❌ DIFFERENT
0x10:    5a 2c 05 01 04 00 00 00 00 00 00 00 00 00 00 00  ❌ DIFFERENT
0x20:    5a 11 32 08 00 96 0b 00 00 00 00 96 0c 00 00 00  ❌ DIFFERENT
0x30:    5a 26 0b 08 ff ff 00 00 00 00 00 00 00 00 00 00  ❌ ERROR PATTERN
0x15:    5a 26 0b 08 ff ff 00 00 00 00 00 00 00 00 00 00  ❌ ERROR PATTERN
0x3a_v1: 5a 26 0b 08 ff ff 00 00 00 00 00 00 00 00 00 00  ❌ ERROR PATTERN
0x3a_v2: 5a 11 08 01 00 96 00 00 00 80 3f 00 00 00 00 00  ❌ DIFFERENT
```

### 🎯 KEY DISCOVERY

**The instability happens even with OFFICIAL Creative software!** This proves:

1. ✅ Our read implementation is correct
2. ✅ The `write()` and `read()` methods work fine
3. ❌ The device itself gets into an unstable state after ANY write operation
4. ❌ Responses don't echo commands after writes - we get async data instead

### Theory: Hardware Transition State

User reports an **audible click** (relay/solenoid) when switching outputs. This means:

1. **Physical hardware switching** - The device has a relay that physically switches audio paths
2. **Transition period** - During the click, the device is in a hardware transition state
3. **Busy/Status updates** - The device likely sends status messages during this transition
4. **Unstable reads** - We're trying to read while the device is still switching hardware

After writes, the device appears to enter **hardware transition mode**:
- Physical relay clicks (takes time to complete)
- Device sends status updates about the transition
- Pattern `5a 26 0b 08 ff ff` might be "hardware busy" or "transition in progress"
- Pattern `5a 11 32` / `5a 2c 05` might be transition status notifications
- Stops echoing our read commands because it's focused on the hardware operation

### Solution Ideas

**Option 1: Wait for Hardware Transition**
- After a write, wait for the hardware relay to complete (500ms - 1s?)
- Then drain any pending status messages
- Finally send read commands when device is stable
- This respects the physical hardware transition time

**Option 2: Read Until Stable**
- After a write, read repeatedly until we get responses that echo our commands
- This would "drain" the transition status queue
- Could combine with a delay to let hardware settle

**Option 3: Reconnect After Writes**
- Accept that writes destabilize the device
- Disconnect and reconnect to reset device state
- Then read clean responses
- Simple but slow

**Option 4: Decode Transition Status Format**
- Figure out what `5a 11 32` and `5a 2c 05` actually mean
- Parse the transition status messages
- Look for a "ready" message that indicates hardware transition complete
- Most robust but requires more reverse engineering

**BEST APPROACH:** Try Option 1 first - add a delay after writes to let the relay complete, then clear any pending messages before reading. This matches the physical reality of the hardware.

## Next Steps

The responses we got in the FIRST test (before any writes) were clean and consistent:
```
0x05: 5a 05 04 1f
0x15: 5a 15 2c 00 03 96 0a 00 00 c0 40 00 00 c0 c0 00
0x3a: 5a 3a 02 07 01
```

We should **decode these stable responses first** to understand the baseline format. Then we can tackle the async notification problem.

## Test 5: Crystalizer Toggle (DSP Only - No Relay)

User enabled Crystalizer in Creative software (pure DSP, no physical relay).

### BEFORE (Baseline):
```
0x05: 5a 05 04 1f
0x10: 5a 10 08 ef 67 74
0x20: 5a 20 04 97
0x30: 5a 30 04 30 01 10
0x15: 5a 15 2c 00 03 96 0a 00 00 c0 40 00 00 c0 c0 00
0x3a_v1: 5a 3a 03 09 00 03
0x05 (repeat): 5a 05 04 1f
0x39: 5a 02 0a 39 81
0x3a_v2: 5a 3a 02 07 01
```

### AFTER (Crystalizer Enabled):
```
0x05: 5a 02 0a 12           <- NEW/DIFFERENT
0x10: 5a 11 08 01 00 96 07  <- NEW/DIFFERENT
0x20: 5a 11 08 01 00 96 07  <- DUPLICATE of 0x10 response!
0x30: 5a 05 04 1f           <- THIS IS THE OLD 0x05 RESPONSE!
0x15: 5a 10 08 ef 67 74     <- THIS IS THE OLD 0x10 RESPONSE!
0x3a_v1: 5a 20 04 97        <- THIS IS THE OLD 0x20 RESPONSE!
0x05 (repeat): 5a 30 04 30 01 10  <- THIS IS THE OLD 0x30 RESPONSE!
0x39: 5a 15 2c 00 03 96 0a  <- THIS IS THE OLD 0x15 RESPONSE!
0x3a_v2: 5a 3a 03 09 00 03  <- THIS IS THE OLD 0x3a_v1 RESPONSE!
```

### 🎯 CRITICAL DISCOVERY: Response Buffer Shift!

**The responses are NOT random - they're SHIFTED!**

After ANY write (even DSP-only like Crystalizer), the device enters a mode where:
1. **First few responses** contain NEW data (device status updates)
2. **Later responses** are BUFFERED OLD RESPONSES from before the write!
3. **The responses are echoing our PREVIOUS read sequence!**

This pattern shows:
- Position 3 (0x30) returns what 0x05 returned before
- Position 4 (0x15) returns what 0x10 returned before  
- Position 5 (0x3a_v1) returns what 0x20 returned before
- And so on...

### What This Means

The device has a **response buffer** that gets filled with status updates when we write. When we read:
1. First reads get the NEW status messages from the buffer
2. Once buffer is empty, device starts echoing our PREVIOUS responses
3. This proves the device is caching/buffering responses

### Solution: Drain the Buffer First!

After ANY write operation, we need to:
1. Send dummy read commands to drain the status update buffer
2. Keep reading until responses stabilize (echo our commands properly)
3. THEN send our actual read commands

Pseudocode:
```rust
// After write operation
for _ in 0..10 {  // Drain up to 10 buffered messages
    let dummy = build_status_query();
    let _ = send_read_command(dummy);  // Discard response
}

// Now device should be stable
let actual_response = send_read_command(build_status_query());
```

##Status

- ✅ Read mechanism works correctly
- ✅ Confirmed: ANY write causes buffer/async mode (not just relay)
- ✅ DISCOVERED: Responses are buffered/shifted, not random
- ✅ SOLUTION: Must drain buffer after writes before reading real data
- ⏳ Implement buffer draining logic
- ⏳ Then decode the clean baseline responses
- ⏳ Full integration with auto-drain after writes
