/// USB Protocol implementation for SoundBlaster X G6
/// Based on reverse engineering from soundblaster-x-g6-cli project
use crate::g6_spec::{
    EffectState, EqualizerBand, EqualizerConfig, ExtendedAudioParams, FirmwareInfo, G6Settings,
    OutputDevice, SmartVolumePreset,
};

// Protocol constants
const PREFIX: u8 = 0x5a;
const REQUEST_DATA: u16 = 0x1207;
const REQUEST_COMMIT: u16 = 0x1103;
const REQUEST_READ: u16 = 0x1103; // Corrected to match Spec (Same as Commit)
const INTERMEDIATE_AUDIO: u16 = 0x0196; // Audio effects
const INTERMEDIATE_EQ: u16 = 0x0195; // Equalizer
const PAYLOAD_SIZE: usize = 64;

// Feature hex codes (for toggles)
const FEATURE_SURROUND: u8 = 0x00;
const FEATURE_SBX_MODE: u8 = 0x01; // In 0x26 protocol
const FEATURE_SCOUT_MODE: u8 = 0x02; // In 0x26 protocol
const FEATURE_CRYSTALIZER: u8 = 0x07;
const FEATURE_BASS: u8 = 0x18;
const FEATURE_SMART_VOLUME: u8 = 0x04;
const FEATURE_DIALOG_PLUS: u8 = 0x02;

// Feature hex codes (for sliders = feature + 1)
const FEATURE_SURROUND_SLIDER: u8 = 0x01;
const FEATURE_CRYSTALIZER_SLIDER: u8 = 0x08;
const FEATURE_BASS_SLIDER: u8 = 0x19;
const FEATURE_SMART_VOLUME_SLIDER: u8 = 0x05;
const FEATURE_SMART_VOLUME_SPECIAL: u8 = 0x06;
const FEATURE_DIALOG_PLUS_SLIDER: u8 = 0x03;

// Toggle values
const VALUE_ENABLED: u32 = 0x3f800000; // 1.0f as little-endian u32 (bytes: 00 00 80 3f)
const VALUE_DISABLED: u32 = 0x00000000;

/// Build a 64-byte USB HID command
fn build_command(request_type: u16, feature: u8, value: u32) -> Vec<u8> {
    let mut command = Vec::with_capacity(PAYLOAD_SIZE);

    // Prefix (1 byte)
    command.push(PREFIX);

    // Request type (2 bytes, big-endian)
    command.extend_from_slice(&request_type.to_be_bytes());

    // Intermediate (2 bytes, big-endian)
    command.extend_from_slice(&INTERMEDIATE_AUDIO.to_be_bytes());

    // Feature (1 byte)
    command.push(feature);

    // Value (4 bytes, little-endian as per USB spec)
    command.extend_from_slice(&value.to_le_bytes());

    // Padding to 64 bytes
    command.resize(PAYLOAD_SIZE, 0x00);

    command
}

/// Build a 64-byte USB HID read command
fn build_read_command(request_type: u16, intermediate: u16, feature: u8) -> Vec<u8> {
    let mut command = Vec::with_capacity(PAYLOAD_SIZE);

    // Prefix (1 byte)
    command.push(PREFIX);

    // Request type (2 bytes, big-endian)
    command.extend_from_slice(&request_type.to_be_bytes());

    // Intermediate (2 bytes, big-endian)
    command.extend_from_slice(&intermediate.to_be_bytes());

    // Feature (1 byte)
    command.push(feature);

    // Padding to 64 bytes with zeros
    command.resize(PAYLOAD_SIZE, 0x00);

    command
}

/// Build command pair (DATA + COMMIT) for a toggle operation
pub fn build_toggle_commands(feature: u8, enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled {
        VALUE_ENABLED
    } else {
        VALUE_DISABLED
    };

    vec![
        build_command(REQUEST_DATA, feature, value),
        build_command(REQUEST_COMMIT, feature, 0),
    ]
}

/// Build command pair (DATA + COMMIT) for a slider operation
pub fn build_slider_commands(feature: u8, value: u8) -> Vec<Vec<u8>> {
    // Convert 0-100 value to the G6's expected format
    let hex_value = value_to_hex(value);

    vec![
        build_command(REQUEST_DATA, feature, hex_value),
        build_command(REQUEST_COMMIT, feature, 0),
    ]
}

/// Convert a 0-100 value to the G6's hex representation
/// The G6 expects IEEE 754 floating point values from 0.0 to 1.0
fn value_to_hex(value: u8) -> u32 {
    // Convert 0-100 to 0.0-1.0 range
    let float_val = (value as f32) / 100.0;

    // Convert to IEEE 754 little-endian bytes and then to u32
    let bytes = float_val.to_le_bytes();
    u32::from_le_bytes(bytes)
}

/// Build commands for surround effect
pub fn build_surround_toggle(enabled: bool) -> Vec<Vec<u8>> {
    build_toggle_commands(FEATURE_SURROUND, enabled)
}

pub fn build_surround_slider(value: u8) -> Vec<Vec<u8>> {
    build_slider_commands(FEATURE_SURROUND_SLIDER, value)
}

/// Build commands for crystalizer effect
pub fn build_crystalizer_toggle(enabled: bool) -> Vec<Vec<u8>> {
    build_toggle_commands(FEATURE_CRYSTALIZER, enabled)
}

pub fn build_crystalizer_slider(value: u8) -> Vec<Vec<u8>> {
    build_slider_commands(FEATURE_CRYSTALIZER_SLIDER, value)
}

/// Build commands for bass effect
pub fn build_bass_toggle(enabled: bool) -> Vec<Vec<u8>> {
    build_toggle_commands(FEATURE_BASS, enabled)
}

pub fn build_bass_slider(value: u8) -> Vec<Vec<u8>> {
    build_slider_commands(FEATURE_BASS_SLIDER, value)
}

/// Build commands for smart volume effect
pub fn build_smart_volume_toggle(enabled: bool) -> Vec<Vec<u8>> {
    build_toggle_commands(FEATURE_SMART_VOLUME, enabled)
}

pub fn build_smart_volume_slider(value: u8) -> Vec<Vec<u8>> {
    build_slider_commands(FEATURE_SMART_VOLUME_SLIDER, value)
}

/// Build commands for dialog plus effect  
pub fn build_dialog_plus_toggle(enabled: bool) -> Vec<Vec<u8>> {
    build_toggle_commands(FEATURE_DIALOG_PLUS, enabled)
}

pub fn build_dialog_plus_slider(value: u8) -> Vec<Vec<u8>> {
    build_slider_commands(FEATURE_DIALOG_PLUS_SLIDER, value)
}

/// Parse hex string to bytes
fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// Build commands for SBX Mode (Master Switch)
pub fn build_sbx_mode_enable() -> Vec<Vec<u8>> {
    // Verified via packet capture:
    // DATA:   5a 26 05 07 01 00 01 00 00...
    // COMMIT: 5a 26 03 08 ff ff 00 00 00...

    let mut data = Vec::with_capacity(PAYLOAD_SIZE);
    data.push(PREFIX);
    data.extend_from_slice(&[0x26, 0x05, 0x07, FEATURE_SBX_MODE, 0x00, 0x01, 0x00, 0x00]);
    data.resize(PAYLOAD_SIZE, 0x00);

    let mut commit = Vec::with_capacity(PAYLOAD_SIZE);
    commit.push(PREFIX);
    commit.extend_from_slice(&[0x26, 0x03, 0x08, 0xff, 0xff, 0x00, 0x00, 0x00]);
    commit.resize(PAYLOAD_SIZE, 0x00);

    vec![data, commit]
}

pub fn build_sbx_mode_disable() -> Vec<Vec<u8>> {
    // Verified via packet capture:
    // DATA:   5a 26 05 07 01 00 00 00 00...
    // COMMIT: 5a 26 03 08 ff ff 00 00 00...

    let mut data = Vec::with_capacity(PAYLOAD_SIZE);
    data.push(PREFIX);
    data.extend_from_slice(&[0x26, 0x05, 0x07, FEATURE_SBX_MODE, 0x00, 0x00, 0x00, 0x00]);
    data.resize(PAYLOAD_SIZE, 0x00);

    let mut commit = Vec::with_capacity(PAYLOAD_SIZE);
    commit.push(PREFIX);
    commit.extend_from_slice(&[0x26, 0x03, 0x08, 0xff, 0xff, 0x00, 0x00, 0x00]);
    commit.resize(PAYLOAD_SIZE, 0x00);

    vec![data, commit]
}

/// Build commands for Scout Mode
pub fn build_scout_mode_enable() -> Vec<Vec<u8>> {
    // Inferred from SBX Mode pattern:
    // Uses FEATURE_SCOUT_MODE (0x02) instead of FEATURE_SBX_MODE (0x01)

    let mut data = Vec::with_capacity(PAYLOAD_SIZE);
    data.push(PREFIX);
    data.extend_from_slice(&[0x26, 0x05, 0x07, FEATURE_SCOUT_MODE, 0x00, 0x01, 0x00, 0x00]);
    data.resize(PAYLOAD_SIZE, 0x00);

    let mut commit = Vec::with_capacity(PAYLOAD_SIZE);
    commit.push(PREFIX);
    commit.extend_from_slice(&[0x26, 0x03, 0x08, 0xff, 0xff, 0x00, 0x00, 0x00]);
    commit.resize(PAYLOAD_SIZE, 0x00);

    vec![data, commit]
}

pub fn build_scout_mode_disable() -> Vec<Vec<u8>> {
    let mut data = Vec::with_capacity(PAYLOAD_SIZE);
    data.push(PREFIX);
    data.extend_from_slice(&[0x26, 0x05, 0x07, FEATURE_SCOUT_MODE, 0x00, 0x00, 0x00, 0x00]);
    data.resize(PAYLOAD_SIZE, 0x00);

    let mut commit = Vec::with_capacity(PAYLOAD_SIZE);
    commit.push(PREFIX);
    commit.extend_from_slice(&[0x26, 0x03, 0x08, 0xff, 0xff, 0x00, 0x00, 0x00]);
    commit.resize(PAYLOAD_SIZE, 0x00);

    vec![data, commit]
}

/// Build commands for switching to headphones
pub fn build_output_headphones() -> Vec<Vec<u8>> {
    vec![
        hex_to_bytes("5a2c0500040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a2c0101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    ]
}

/// Build commands for switching to speakers
pub fn build_output_speakers() -> Vec<Vec<u8>> {
    vec![
        hex_to_bytes("5a2c0500020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a2c0101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301961400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a120701960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
        hex_to_bytes("5a110301960900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"),
    ]
}

/// Build commands for toggling output
pub fn build_output_toggle(current: OutputDevice) -> Vec<Vec<u8>> {
    match current {
        OutputDevice::Headphones => build_output_speakers(),
        OutputDevice::Speakers => build_output_headphones(),
    }
}

// === DEVICE STATE READING FUNCTIONS ===

/// Build command to read audio effect state
pub fn build_read_audio_effect(feature: u8) -> Vec<u8> {
    build_read_command(REQUEST_READ, INTERMEDIATE_AUDIO, feature)
}

/// Build command to read equalizer band
pub fn build_read_equalizer_band(band: u8) -> Vec<u8> {
    build_read_command(REQUEST_READ, INTERMEDIATE_EQ, band)
}

/// Build command to read firmware version (Candidate 1: Identification)
pub fn build_read_firmware_v1() -> Vec<u8> {
    let mut command = vec![PREFIX, 0x05, 0x01];
    command.resize(PAYLOAD_SIZE, 0x00);
    command
}

/// Build command to read firmware version (Candidate 2: Original undocumented)
pub fn build_read_firmware_v2() -> Vec<u8> {
    let mut command = vec![PREFIX, 0x07, 0x10];
    command.resize(PAYLOAD_SIZE, 0x00);
    command
}

/// Build command to read current output configuration
pub fn build_read_output_config() -> Vec<u8> {
    // Based on analysis: 5a2c0a... reads output routing configuration
    let mut command = vec![PREFIX, 0x2c, 0x0a, 0x02, 0x82, 0x02];
    command.resize(PAYLOAD_SIZE, 0x00);
    command
}

/// Build commands for comprehensive device state reading
pub fn build_read_all_state_commands() -> Vec<Vec<u8>> {
    let mut commands = Vec::new();

    // Read firmware version (Try multiple candidates)
    commands.push(build_read_firmware_v1());
    commands.push(build_read_firmware_v2());

    // Read output configuration
    commands.push(build_read_output_config());

    // Read all audio effects (0x00 to 0x1D)
    for feature in 0x00..=0x1D {
        commands.push(build_read_audio_effect(feature));
    }

    // Read equalizer bands (0x00 to 0x1B)
    for band in 0x00..=0x1B {
        commands.push(build_read_equalizer_band(band));
    }

    commands
}

// === RESPONSE PARSING FUNCTIONS ===

/// Parse IEEE 754 little-endian float from 4 bytes
fn parse_ieee754_le(bytes: &[u8]) -> Result<f32, &'static str> {
    if bytes.len() < 4 {
        return Err("Not enough bytes for IEEE 754 float");
    }

    let float_bytes = [bytes[0], bytes[1], bytes[2], bytes[3]];
    Ok(f32::from_le_bytes(float_bytes))
}

/// Parse firmware version from response
pub fn parse_firmware_response(response: &[u8]) -> Result<FirmwareInfo, String> {
    if response.len() < 10 {
        return Err("Response too short for firmware info".to_string());
    }

    // Look for ASCII content starting from different positions
    // The version might start after different header lengths
    let search_positions = [3, 4, 5, 6, 7, 8, 9, 10];

    for start_pos in search_positions {
        if start_pos >= response.len() {
            continue;
        }

        // Look for continuous ASCII sequence starting at this position
        let mut version_start = None;
        let mut consecutive_ascii = 0;

        for i in start_pos..response.len() {
            if response[i] >= 32 && response[i] <= 126 {
                // Printable ASCII
                if version_start.is_none() {
                    version_start = Some(i);
                }
                consecutive_ascii += 1;
            } else if response[i] == 0 {
                // Null terminator - end of string
                break;
            } else {
                // Non-ASCII byte - reset
                version_start = None;
                consecutive_ascii = 0;
            }

            // If we found at least 2 consecutive ASCII chars (e.g. "V2"), that helps
            if consecutive_ascii >= 2 {
                if let Some(start) = version_start {
                    // Find the end of the ASCII sequence
                    let mut version_end = i + 1;
                    for j in (i + 1)..response.len() {
                        if response[j] >= 32 && response[j] <= 126 {
                            version_end = j + 1;
                        } else {
                            break;
                        }
                    }

                    let version_bytes = &response[start..version_end];
                    let version = String::from_utf8_lossy(version_bytes)
                        .to_string()
                        .trim()
                        .to_string();

                    // We accept length >= 2 to catch "V2" which is what G6 reports
                    if !version.is_empty() && version.len() >= 2 {
                        // Validate it looks somewhat like a version (has digits or 'V')
                        if version.contains(|c: char| c.is_numeric() || c == 'V' || c == 'v') {
                            return Ok(FirmwareInfo {
                                version: version.clone(),
                                build: if version.contains('.') {
                                    Some(version)
                                } else {
                                    None
                                },
                            });
                        }
                    }
                }
            }
        }
    }

    Err("Could not find valid firmware version in response".to_string())
}

/// Parse audio effect response to extract current value and state
pub fn parse_audio_effect_response(response: &[u8]) -> Result<(EffectState, f32), String> {
    if response.len() < 11 {
        return Err("Response too short for audio effect".to_string());
    }

    // Audio effect values start at byte 7 (after Type and Feature)
    // [5a, 11, 08, 01, 00, 96, <Feature>, <Val0>, <Val1>, <Val2>, <Val3>]
    let value_bytes = &response[7..11];
    let float_value = parse_ieee754_le(value_bytes)
        .map_err(|e| format!("Failed to parse IEEE 754 value: {}", e))?;

    // Determine if effect is enabled based on value
    let state = if float_value > 0.0001 {
        // Account for floating point precision
        EffectState::Enabled
    } else {
        EffectState::Disabled
    };

    Ok((state, float_value))
}

/// Parse equalizer band response
pub fn parse_equalizer_response(response: &[u8]) -> Result<EqualizerBand, String> {
    if response.len() < 15 {
        return Err("Response too short for equalizer band".to_string());
    }

    // Frequency typically at bytes 7-11, gain at bytes 11-15
    // Assuming similar shift: [5a 11 08 01 00 95 <Band> <F0> <F1> <F2> <F3> <G0> <G1> <G2> <G3>]
    let freq_bytes = &response[7..11];
    let gain_bytes = &response[11..15];

    let frequency =
        parse_ieee754_le(freq_bytes).map_err(|e| format!("Failed to parse frequency: {}", e))?;
    let gain = parse_ieee754_le(gain_bytes).map_err(|e| format!("Failed to parse gain: {}", e))?;

    Ok(EqualizerBand { frequency, gain })
}

/// Parse output configuration response
pub fn parse_output_config_response(response: &[u8]) -> Result<OutputDevice, String> {
    if response.len() < 10 {
        return Err("Response too short for output config".to_string());
    }

    // Look for output type indicator in the response
    // Based on command analysis: 0x02 = Speakers, 0x04 = Headphones
    // The actual output value is at index 9 in the response
    // Format: 5a 2c 0a 02 82 02 00 00 00 [OUTPUT_VALUE]
    if response.len() > 9 {
        match response[9] {
            0x04 => return Ok(OutputDevice::Headphones),
            0x02 => return Ok(OutputDevice::Speakers),
            _ => {} // Fall through to scanning if index 9 doesn't have expected value
        }
    }

    // Fallback: scan from index 4 onwards (skip header bytes)
    for i in 4..10 {
        match response[i] {
            0x04 => return Ok(OutputDevice::Headphones),
            0x02 => return Ok(OutputDevice::Speakers),
            _ => continue,
        }
    }

    Err("Could not determine output device from response".to_string())
}

/// Convert IEEE 754 float (0.0-1.0) to percentage (0-100)
pub fn float_to_percentage(value: f32) -> u8 {
    (value * 100.0).round().clamp(0.0, 100.0) as u8
}

/// Describe a packet for logging purposes
pub fn describe_packet(packet: &[u8]) -> String {
    if packet.len() < 2 {
        return "Invalid Packet (< 2 bytes)".to_string();
    }

    if packet[0] != PREFIX {
        return "Unknown Protocol".to_string();
    }

    // Helper to get feature name
    let get_feature_name = |f: u8| -> &str {
        match f {
            FEATURE_SURROUND => "Surround",
            FEATURE_CRYSTALIZER => "Crystalizer",
            FEATURE_BASS => "Bass",
            FEATURE_SMART_VOLUME => "Smart Vol",
            FEATURE_DIALOG_PLUS => "Dialog+",
            FEATURE_SURROUND_SLIDER => "Surround (Slider)",
            FEATURE_CRYSTALIZER_SLIDER => "Crystalizer (Slider)",
            FEATURE_BASS_SLIDER => "Bass (Slider)",
            FEATURE_SMART_VOLUME_SLIDER => "Smart Vol (Slider)",
            FEATURE_SMART_VOLUME_SPECIAL => "Smart Vol (Preset)",
            FEATURE_DIALOG_PLUS_SLIDER => "Dialog+ (Slider)",
            0x0a..=0x14 | 0x1a..=0x1d => "Extended Param",
            _ => "Unknown Feature",
        }
    };

    // Helper to format value
    let format_value = |p: &[u8], f: u8| -> String {
        if p.len() < 4 {
            return "Invalid Value".to_string();
        }
        if let Ok(val) = parse_ieee754_le(p) {
            if val.abs() < 0.001 {
                "Disabled (0.0)".to_string()
            } else if (val - 1.0).abs() < 0.001 {
                "Enabled (1.0)".to_string()
            } else if (val - 0.5).abs() < 0.001
                && (f == FEATURE_SMART_VOLUME
                    || f == FEATURE_SMART_VOLUME_SPECIAL
                    || f == FEATURE_SMART_VOLUME_SLIDER)
            {
                "Night Mode (0.5)".to_string()
            } else {
                format!("{:.2} ({})", val, float_to_percentage(val))
            }
        } else {
            "Hex Value".to_string()
        }
    };

    // Analyze Packet based on Pattern
    // Standard Control Protocol: 5a 12 07 <Feature> <Value...>
    if packet.len() >= 8 && packet[1] == 0x12 && packet[2] == 0x07 {
        let feature = packet[3];
        let value_str = format_value(&packet[4..8], feature);
        return format!("Set {}: {}", get_feature_name(feature), value_str);
    }

    // Standard Read/Commit Protocol: 5a 11 03 <Intermediate> <Feature>
    if packet.len() >= 5 && packet[1] == 0x11 && packet[2] == 0x03 {
        let feature = packet[4];
        return format!("Commit/Read {}", get_feature_name(feature));
    }

    // Audio Effect Response/Report: 5a 11 08 01 00 96 <Feature> <Value...>
    // OR Packet Report: 5a 11 08 ...
    if packet.len() >= 11 && packet[1] == 0x11 && packet[2] == 0x08 {
        // Check for Audio Effect (0x96)
        if packet.len() >= 7 && packet[5] == 0x96 {
            let feature = packet[6];
            let value_str = format_value(&packet[7..11], feature);
            return format!("Report {}: {}", get_feature_name(feature), value_str);
        }
    }

    // SBX/Scout Mode Protocol: 5a 26 ...
    if packet[1] == 0x26 {
        // Set/Report
        if packet.len() >= 8 && packet[2] == 0x05 && packet[3] == 0x07 {
            let feature = packet[4];
            let enabled = packet[6] == 0x01;
            let mode = match feature {
                FEATURE_SBX_MODE => "SBX Mode",
                FEATURE_SCOUT_MODE => "Scout Mode",
                _ => "Unknown 0x26 Feature",
            };
            let state = if enabled { "Enabled" } else { "Disabled" };
            return format!("Set/Report {}: {}", mode, state);
        }
        // Commit
        if packet.len() >= 4 && packet[2] == 0x03 && packet[3] == 0x08 {
            return "Commit SBX/Scout Mode".to_string();
        }
    }

    // Output Switching: 5a 2c ...
    if packet[1] == 0x2c {
        if packet.len() >= 6 && packet[2] == 0x05 {
            // Packet might be 5a 2c 05 01 04 ... (Scan for 02/04)
            for i in 3..10 {
                match packet[i] {
                    0x02 => return "Set Output: Speakers".to_string(),
                    0x04 => return "Set Output: Headphones".to_string(),
                    _ => continue,
                }
            }
            return "Set Output: Unknown".to_string();
        }
        if packet.len() >= 4 && packet[2] == 0x01 && packet[3] == 0x01 {
            return "Commit Output Change".to_string();
        }
        if packet.len() >= 6 && packet[2] == 0x0a {
            // Report format: 5a 2c 0a 02 82 02 00 00 00 [OUTPUT_VALUE]
            // The actual output value is at index 9
            if packet.len() > 9 {
                match packet[9] {
                    0x04 => return "Report Output: Headphones".to_string(),
                    0x02 => return "Report Output: Speakers".to_string(),
                    _ => {} // Fall through to scanning
                }
            }
            // Fallback: scan from index 4 onwards (skip header bytes)
            for i in 4..10 {
                match packet[i] {
                    0x04 => return "Report Output: Headphones".to_string(),
                    0x02 => return "Report Output: Speakers".to_string(),
                    _ => continue,
                }
            }
            return "Report Output: Unknown".to_string();
        }
    }

    // Firmware/Init
    if packet.len() >= 6 && packet[1] == 0x05 && packet[2] == 0x01 {
        return "Read Firmware V1".to_string();
    }
    if packet.len() >= 6 && packet[1] == 0x07 && packet[2] == 0x10 {
        return "Read Firmware V2".to_string();
    }

    // Default fallback
    format!("Unknown Command/Report (Type {:02x})", packet[1])
}

/// Parse complete device state from multiple responses
pub fn parse_device_state_responses(responses: &[Vec<u8>]) -> Result<G6Settings, String> {
    let mut settings = G6Settings::default();
    settings.is_connected = true;
    settings.last_read_time = Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    if responses.is_empty() {
        return Err("No responses provided".to_string());
    }

    // Parse firmware info (Try first two responses)
    if let Ok(firmware) = parse_firmware_response(&responses[0]) {
        settings.firmware_info = Some(firmware);
    } else if responses.len() > 1 {
        if let Ok(firmware) = parse_firmware_response(&responses[1]) {
            settings.firmware_info = Some(firmware);
        }
    }

    // Parse output configuration (It's now at index 2 because we added a candidate)
    if responses.len() > 2 {
        if let Ok(output_device) = parse_output_config_response(&responses[2]) {
            settings.output = output_device;
        }
    }

    // Parse audio effects (responses start from index 3, features 0x00-0x1D)
    let mut extended_params = ExtendedAudioParams::default();

    let effect_start_idx = 3;
    if responses.len() > effect_start_idx {
        for (i, response) in responses[effect_start_idx..].iter().enumerate().take(30) {
            if let Ok((state, value)) = parse_audio_effect_response(response) {
                let percentage = float_to_percentage(value);

                match i as u8 {
                    0x00 => settings.surround_enabled = state,
                    0x01 => settings.surround_value = percentage,
                    0x02 => settings.dialog_plus_enabled = state,
                    0x03 => settings.dialog_plus_value = percentage,
                    0x04 => settings.smart_volume_enabled = state,
                    0x05 => settings.smart_volume_value = percentage,
                    0x06 => {
                        // Smart Volume Preset (0.5 = Night, 1.0 = Loud)
                        if value > 0.9 {
                            settings.smart_volume_preset = Some(SmartVolumePreset::Loud);
                        } else if value > 0.4 {
                            settings.smart_volume_preset = Some(SmartVolumePreset::Night);
                        } else {
                            settings.smart_volume_preset = None;
                        }
                    }
                    0x07 => settings.crystalizer_enabled = state,
                    0x08 => settings.crystalizer_value = percentage,
                    0x18 => settings.bass_enabled = state,
                    0x19 => settings.bass_value = percentage,
                    0x0a => extended_params.param_0x0a = Some(value),
                    0x0b => extended_params.param_0x0b = Some(value),
                    0x0c => extended_params.param_0x0c = Some(value),
                    0x0d => extended_params.param_0x0d = Some(value),
                    0x0e => extended_params.param_0x0e = Some(value),
                    0x0f => extended_params.param_0x0f = Some(value),
                    0x10 => extended_params.param_0x10 = Some(value),
                    0x11 => extended_params.param_0x11 = Some(value),
                    0x12 => extended_params.param_0x12 = Some(value),
                    0x13 => extended_params.param_0x13 = Some(value),
                    0x14 => extended_params.param_0x14 = Some(value),
                    0x1a => extended_params.param_0x1a = Some(value),
                    0x1b => extended_params.param_0x1b = Some(value),
                    0x1c => extended_params.param_0x1c = Some(value),
                    0x1d => extended_params.param_0x1d = Some(value),
                    _ => {} // Skip unknown features
                }
            }
        }
    }

    settings.extended_params = Some(extended_params);

    // Parse equalizer bands (remaining responses)
    let eq_start = 3 + 30; // After 2 firmware candidates + output config + 30 audio effects
    if responses.len() > eq_start {
        let mut eq_config = EqualizerConfig::default();
        let mut bands = Vec::new();

        for response in responses[eq_start..].iter().take(28) {
            // 0x00-0x1B = 28 bands
            if let Ok(band) = parse_equalizer_response(response) {
                bands.push(band);
            }
        }

        if !bands.is_empty() {
            eq_config.enabled = if bands.iter().any(|b| b.gain.abs() > 0.001) {
                EffectState::Enabled
            } else {
                EffectState::Disabled
            };
            eq_config.bands = bands;
            settings.equalizer = Some(eq_config);
        }
    }

    Ok(settings)
}
