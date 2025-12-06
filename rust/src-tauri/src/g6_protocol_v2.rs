/// USB Protocol V2 - Unified Abstraction Layer
/// This module provides a cleaner, more maintainable protocol implementation
/// Built alongside the original g6_protocol.rs to allow gradual migration
use crate::g6_spec::{EffectState, FirmwareInfo, OutputDevice};

// ============================================================================
// PROTOCOL CONSTANTS
// ============================================================================

const PREFIX: u8 = 0x5a;
const PAYLOAD_SIZE: usize = 64;

// ============================================================================
// COMMAND FAMILIES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandFamily {
    Identification = 0x05, // Device info queries
    FirmwareQuery = 0x07,  // Firmware version (discovered ASCII mode)
    HardwareStatus = 0x10, // Hardware state
    AudioControl = 0x11,   // Primary protocol - Read/Write SBX & EQ
    DataControl = 0x12,    // Audio effect data commands
    BatchControl = 0x15,   // Multiple params simultaneously
    Processing = 0x20,     // Audio processing engine
    Gaming = 0x26,         // Scout/SBX Mode switches
    Routing = 0x2c,        // Output switching
    DeviceConfig = 0x30,   // General device settings
    SystemConfig = 0x3a,   // LEDs, system parameters
}

impl CommandFamily {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// OPERATION TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Identification, // 0x01 - Device identification
    AsciiQuery,     // 0x01 0x02 - ASCII response mode
    Read,           // 0x03 - Read current state
    Write,          // 0x07 - Write data
    Report,         // 0x08 - Device report
    Query,          // 0x0a - Query configuration
    BinaryQuery,    // 0x10 - Binary response mode
}

// ============================================================================
// INTERMEDIATE TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum IntermediateType {
    Equalizer = 0x0195, // EQ bands
    Audio = 0x0196,     // SBX audio effects
}

impl IntermediateType {
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

// ============================================================================
// RESPONSE FORMAT
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedResponse {
    Ascii(String),
    Float(f32),
    Boolean(bool),
    Binary(Vec<u8>),
    OutputDevice(OutputDevice),
    FirmwareInfo(FirmwareInfo),
    EffectState { enabled: EffectState, value: f32 },
}

// ============================================================================
// DEBUG INFORMATION
// ============================================================================

#[derive(Debug, Clone)]
pub struct ProtocolDebugInfo {
    pub command_hex: String,
    pub response_hex: String,
    pub command_description: String,
    pub response_description: String,
    pub parsed_result: String,
    pub parsing_attempts: Vec<String>,
    pub success: bool,
    pub timestamp: u64,
}

impl ProtocolDebugInfo {
    pub fn new() -> Self {
        Self {
            command_hex: String::new(),
            response_hex: String::new(),
            command_description: String::new(),
            response_description: String::new(),
            parsed_result: String::new(),
            parsing_attempts: Vec::new(),
            success: false,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn to_readable_text(&self) -> String {
        let status = if self.success { "✓" } else { "✗" };
        let mut output = String::new();

        output.push_str(&format!("[{}] {}\n", status, self.command_description));
        output.push_str(&format!("Command:  {}\n", self.command_hex));
        output.push_str(&format!("Response: {}\n", self.response_hex));
        output.push_str(&format!("Result:   {}\n", self.parsed_result));

        if !self.parsing_attempts.is_empty() {
            output.push_str("Attempts:\n");
            for attempt in &self.parsing_attempts {
                output.push_str(&format!("  - {}\n", attempt));
            }
        }

        output.push('\n');
        output
    }

    pub fn to_json(&self) -> String {
        // Simple JSON serialization
        format!(
            r#"{{"command_hex":"{}","response_hex":"{}","command_desc":"{}","response_desc":"{}","result":"{}","success":{},"timestamp":{}}}"#,
            self.command_hex,
            self.response_hex,
            self.command_description,
            self.response_description,
            self.parsed_result.replace('"', "\\\""),
            self.success,
            self.timestamp
        )
    }
}

// ============================================================================
// COMMAND BUILDER
// ============================================================================

#[derive(Debug)]
pub struct G6CommandBuilder {
    family: CommandFamily,
    operation_bytes: Vec<u8>,
    intermediate: Option<u16>,
    feature: Option<u8>,
    value: Option<Vec<u8>>,
}

impl G6CommandBuilder {
    /// Create a new command builder for the specified family
    pub fn new(family: CommandFamily) -> Self {
        Self {
            family,
            operation_bytes: Vec::new(),
            intermediate: None,
            feature: None,
            value: None,
        }
    }

    /// Set operation type (single byte or multi-byte)
    pub fn operation(mut self, bytes: &[u8]) -> Self {
        self.operation_bytes = bytes.to_vec();
        self
    }

    /// Set intermediate type (for audio/EQ operations)
    pub fn intermediate(mut self, intermediate: IntermediateType) -> Self {
        self.intermediate = Some(intermediate.as_u16());
        self
    }

    /// Set feature ID
    pub fn feature(mut self, feature: u8) -> Self {
        self.feature = Some(feature);
        self
    }

    /// Set value bytes (little-endian)
    pub fn value(mut self, value: &[u8]) -> Self {
        self.value = Some(value.to_vec());
        self
    }

    /// Set float value (converted to IEEE 754 LE)
    pub fn float_value(mut self, value: f32) -> Self {
        self.value = Some(value.to_le_bytes().to_vec());
        self
    }

    /// Build the final 64-byte command
    pub fn build(self) -> Vec<u8> {
        let mut command = Vec::with_capacity(PAYLOAD_SIZE);

        // Prefix byte
        command.push(PREFIX);

        // Command family
        command.push(self.family.as_u8());

        // Operation bytes
        command.extend_from_slice(&self.operation_bytes);

        // Intermediate (big-endian)
        if let Some(intermediate) = self.intermediate {
            command.extend_from_slice(&intermediate.to_be_bytes());
        }

        // Feature
        if let Some(feature) = self.feature {
            command.push(feature);
        }

        // Value
        if let Some(ref value) = self.value {
            command.extend_from_slice(value);
        }

        // Padding to 64 bytes
        command.resize(PAYLOAD_SIZE, 0x00);

        command
    }

    /// Build with automatic debug description
    pub fn build_with_debug(self) -> (Vec<u8>, String) {
        let desc = format!(
            "{:?} command (family 0x{:02x})",
            self.family,
            self.family.as_u8()
        );
        let cmd = self.build();
        (cmd, desc)
    }
}

// ============================================================================
// RESPONSE PARSER
// ============================================================================

pub struct G6ResponseParser;

impl G6ResponseParser {
    /// Parse a response and return the parsed data with debug info
    pub fn parse(response: &[u8]) -> (Result<ParsedResponse, String>, ProtocolDebugInfo) {
        let mut debug = ProtocolDebugInfo::new();
        debug.response_hex = Self::bytes_to_hex(response);

        // Check minimum length
        if response.len() < 3 {
            debug.parsed_result = "Error: Response too short".to_string();
            debug.success = false;
            return (Err("Response too short".to_string()), debug);
        }

        // Detect response type from header
        let result = match (response[1], response[2]) {
            (0x07, 0x10) => {
                debug.response_description = "Firmware query response (ASCII)".to_string();
                Self::parse_ascii_firmware(response, &mut debug)
            }
            (0x11, 0x08) => {
                debug.response_description = "Audio effect report".to_string();
                Self::parse_audio_effect(response, &mut debug)
            }
            (0x2c, 0x0a) => {
                debug.response_description = "Output configuration report".to_string();
                Self::parse_output_config(response, &mut debug)
            }
            _ => {
                debug.response_description = format!(
                    "Unknown response type: {:02x} {:02x}",
                    response[1], response[2]
                );
                Self::parse_generic(response, &mut debug)
            }
        };

        debug.success = result.is_ok();
        if let Ok(ref parsed) = result {
            debug.parsed_result = format!("{:?}", parsed);
        } else if let Err(ref err) = result {
            debug.parsed_result = format!("Error: {}", err);
        }

        (result, debug)
    }

    /// Parse ASCII firmware version response
    fn parse_ascii_firmware(
        response: &[u8],
        debug: &mut ProtocolDebugInfo,
    ) -> Result<ParsedResponse, String> {
        // Response format: 5a 07 10 [ASCII_STRING] 00
        if response.len() < 4 {
            return Err("Response too short for firmware".to_string());
        }

        // Find null terminator
        let mut version_end = 3;
        for i in 3..response.len() {
            if response[i] == 0 {
                version_end = i;
                break;
            }
        }

        if version_end > 3 {
            let version_bytes = &response[3..version_end];
            let version = String::from_utf8_lossy(version_bytes)
                .to_string()
                .trim()
                .to_string();

            debug
                .parsing_attempts
                .push(format!("ASCII extraction: \"{}\"", version));

            if !version.is_empty() {
                return Ok(ParsedResponse::FirmwareInfo(FirmwareInfo {
                    version: version.clone(),
                    build: Some(version),
                }));
            }
        }

        Err("Could not extract firmware version".to_string())
    }

    /// Parse audio effect response (IEEE 754 float)
    fn parse_audio_effect(
        response: &[u8],
        debug: &mut ProtocolDebugInfo,
    ) -> Result<ParsedResponse, String> {
        if response.len() < 11 {
            return Err("Response too short for audio effect".to_string());
        }

        // Value at bytes 7-10 (little-endian float)
        let value_bytes = &response[7..11];
        let float_value = f32::from_le_bytes([
            value_bytes[0],
            value_bytes[1],
            value_bytes[2],
            value_bytes[3],
        ]);

        debug
            .parsing_attempts
            .push(format!("IEEE 754 LE: {:.6}", float_value));

        let enabled = if float_value > 0.0001 {
            EffectState::Enabled
        } else {
            EffectState::Disabled
        };

        Ok(ParsedResponse::EffectState {
            enabled,
            value: float_value,
        })
    }

    /// Parse output configuration response
    fn parse_output_config(
        response: &[u8],
        debug: &mut ProtocolDebugInfo,
    ) -> Result<ParsedResponse, String> {
        if response.len() < 10 {
            return Err("Response too short for output config".to_string());
        }

        // Check index 9 first (most reliable)
        if response.len() > 9 {
            match response[9] {
                0x04 => {
                    debug
                        .parsing_attempts
                        .push("Index 9 = 0x04 → Headphones".to_string());
                    return Ok(ParsedResponse::OutputDevice(OutputDevice::Headphones));
                }
                0x02 => {
                    debug
                        .parsing_attempts
                        .push("Index 9 = 0x02 → Speakers".to_string());
                    return Ok(ParsedResponse::OutputDevice(OutputDevice::Speakers));
                }
                _ => {}
            }
        }

        // Fallback: scan indices 4-9
        for i in 4..10 {
            match response[i] {
                0x04 => {
                    debug
                        .parsing_attempts
                        .push(format!("Index {} = 0x04 → Headphones", i));
                    return Ok(ParsedResponse::OutputDevice(OutputDevice::Headphones));
                }
                0x02 => {
                    debug
                        .parsing_attempts
                        .push(format!("Index {} = 0x02 → Speakers", i));
                    return Ok(ParsedResponse::OutputDevice(OutputDevice::Speakers));
                }
                _ => continue,
            }
        }

        Err("Could not determine output device".to_string())
    }

    /// Generic parser for unknown response types
    fn parse_generic(
        response: &[u8],
        debug: &mut ProtocolDebugInfo,
    ) -> Result<ParsedResponse, String> {
        debug
            .parsing_attempts
            .push("Returning raw binary data".to_string());
        Ok(ParsedResponse::Binary(response.to_vec()))
    }

    /// Convert bytes to hex string for debugging
    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes
            .iter()
            .take(20) // First 20 bytes
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Build firmware query command (ASCII mode)
pub fn build_firmware_query_ascii() -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::FirmwareQuery)
        .operation(&[0x01, 0x02]) // ASCII mode
        .build()
}

/// Build firmware query command (binary mode)
pub fn build_firmware_query_binary() -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::FirmwareQuery)
        .operation(&[0x10]) // Binary mode
        .build()
}

/// Build audio effect read command
pub fn build_audio_effect_read(feature: u8) -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::AudioControl)
        .operation(&[0x03, 0x01]) // Read mode
        .intermediate(IntermediateType::Audio)
        .feature(feature)
        .build()
}

/// Build output config read command
pub fn build_output_config_read() -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::Routing)
        .operation(&[0x0a, 0x02, 0x82, 0x02])
        .build()
}

// ============================================================================
// ALL READ COMMANDS (Phase 2)
// ============================================================================

/// Build read command for surround effect
pub fn build_read_surround() -> Vec<u8> {
    build_audio_effect_read(0x00)
}

/// Build read command for surround value
pub fn build_read_surround_value() -> Vec<u8> {
    build_audio_effect_read(0x01)
}

/// Build read command for dialog plus
pub fn build_read_dialog_plus() -> Vec<u8> {
    build_audio_effect_read(0x02)
}

/// Build read command for dialog plus value
pub fn build_read_dialog_plus_value() -> Vec<u8> {
    build_audio_effect_read(0x03)
}

/// Build read command for smart volume
pub fn build_read_smart_volume() -> Vec<u8> {
    build_audio_effect_read(0x04)
}

/// Build read command for smart volume value
pub fn build_read_smart_volume_value() -> Vec<u8> {
    build_audio_effect_read(0x05)
}

/// Build read command for smart volume preset
pub fn build_read_smart_volume_preset() -> Vec<u8> {
    build_audio_effect_read(0x06)
}

/// Build read command for crystalizer
pub fn build_read_crystalizer() -> Vec<u8> {
    build_audio_effect_read(0x07)
}

/// Build read command for crystalizer value
pub fn build_read_crystalizer_value() -> Vec<u8> {
    build_audio_effect_read(0x08)
}

/// Build read command for bass
pub fn build_read_bass() -> Vec<u8> {
    build_audio_effect_read(0x18)
}

/// Build read command for bass value
pub fn build_read_bass_value() -> Vec<u8> {
    build_audio_effect_read(0x19)
}

/// Build read command for extended audio parameter
pub fn build_read_extended_param(param_id: u8) -> Vec<u8> {
    build_audio_effect_read(param_id)
}

/// Build read command for equalizer band
pub fn build_read_equalizer_band(band: u8) -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::AudioControl)
        .operation(&[0x03, 0x01])
        .intermediate(IntermediateType::Equalizer)
        .feature(band)
        .build()
}

/// Build all read commands for complete device state
pub fn build_read_all_state_commands() -> Vec<Vec<u8>> {
    let mut commands = Vec::new();

    // Firmware queries
    commands.push(build_firmware_query_ascii());

    // Output configuration
    commands.push(build_output_config_read());

    // All audio effects (0x00 to 0x1D)
    for feature in 0x00..=0x1D {
        commands.push(build_audio_effect_read(feature));
    }

    // All equalizer bands (0x00 to 0x1B)
    for band in 0x00..=0x1B {
        commands.push(build_read_equalizer_band(band));
    }

    commands
}

// ============================================================================
// WRITE COMMANDS - OUTPUT SWITCHING (Phase 4 - Simple Version)
// ============================================================================

/// Build command to set output device (routing command)
/// Command format: 5a 2c 05 00 [OUTPUT_VALUE] 00 00...
/// OUTPUT_VALUE: 0x02 = Speakers, 0x04 = Headphones
pub fn build_set_output(device: OutputDevice) -> Vec<u8> {
    let output_value = match device {
        OutputDevice::Headphones => 0x04,
        OutputDevice::Speakers => 0x02,
    };

    G6CommandBuilder::new(CommandFamily::Routing)
        .operation(&[0x05, 0x00, output_value])
        .build()
}

/// Build command to commit output change
/// Command format: 5a 2c 01 01 00 00...
pub fn build_commit_output() -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::Routing)
        .operation(&[0x01, 0x01])
        .build()
}

/// Build toggle output command (2-command sequence)
/// This is the minimal version - just routing + commit
pub fn build_toggle_output_simple(current: OutputDevice) -> Vec<Vec<u8>> {
    let target = match current {
        OutputDevice::Headphones => OutputDevice::Speakers,
        OutputDevice::Speakers => OutputDevice::Headphones,
    };

    vec![build_set_output(target), build_commit_output()]
}

// ============================================================================
// WRITE COMMANDS - AUDIO EFFECTS (Phase 6)
// ============================================================================

// Bass Effect Feature IDs
const FEATURE_BASS_TOGGLE: u8 = 0x18;
const FEATURE_BASS_VALUE: u8 = 0x19;

// Gaming Mode Feature IDs (0x26 family)
const FEATURE_SBX_MODE: u8 = 0x01;
const FEATURE_SCOUT_MODE: u8 = 0x02;

/// Build command to set bass toggle (on/off)
/// Command format: DATA (0x12) + COMMIT (0x11)
pub fn build_set_bass_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        // DATA command - Write the toggle value
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01]) // Write operation
            .intermediate(IntermediateType::Audio)
            .feature(FEATURE_BASS_TOGGLE)
            .float_value(value)
            .build(),
        // COMMIT command - Confirm the change
        G6CommandBuilder::new(CommandFamily::AudioControl)
            .operation(&[0x03, 0x01]) // Commit operation
            .intermediate(IntermediateType::Audio)
            .feature(FEATURE_BASS_TOGGLE)
            .build(),
    ]
}

/// Build command to set bass value (0-100)
/// Command format: DATA (0x12) + COMMIT (0x11)
pub fn build_set_bass_value(value: u8) -> Vec<Vec<u8>> {
    // Convert 0-100 to 0.0-1.0 float
    let float_value = (value as f32) / 100.0;

    vec![
        // DATA command - Write the slider value
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01]) // Write operation
            .intermediate(IntermediateType::Audio)
            .feature(FEATURE_BASS_VALUE)
            .float_value(float_value)
            .build(),
        // COMMIT command - Confirm the change
        G6CommandBuilder::new(CommandFamily::AudioControl)
            .operation(&[0x03, 0x01]) // Commit operation
            .intermediate(IntermediateType::Audio)
            .feature(FEATURE_BASS_VALUE)
            .build(),
    ]
}

// ============================================================================
// WRITE COMMANDS - GAMING MODES (Phase 7)
// ============================================================================

/// Build command to set SBX Mode (Master Switch)
/// Command format: DATA (0x26 05 07) + COMMIT (0x26 03 08)
pub fn build_set_sbx_mode(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 0x01 } else { 0x00 };

    vec![
        // DATA command - 5a 26 05 07 01 00 [VALUE] 00 00...
        G6CommandBuilder::new(CommandFamily::Gaming)
            .operation(&[0x05, 0x07, FEATURE_SBX_MODE, 0x00, value, 0x00, 0x00])
            .build(),
        // COMMIT command - 5a 26 03 08 ff ff 00 00 00...
        G6CommandBuilder::new(CommandFamily::Gaming)
            .operation(&[0x03, 0x08, 0xff, 0xff, 0x00, 0x00, 0x00])
            .build(),
    ]
}

/// Build command to set Scout Mode
/// Command format: DATA (0x26 05 07) + COMMIT (0x26 03 08)
pub fn build_set_scout_mode(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 0x01 } else { 0x00 };

    vec![
        // DATA command - 5a 26 05 07 02 00 [VALUE] 00 00...
        G6CommandBuilder::new(CommandFamily::Gaming)
            .operation(&[0x05, 0x07, FEATURE_SCOUT_MODE, 0x00, value, 0x00, 0x00])
            .build(),
        // COMMIT command - 5a 26 03 08 ff ff 00 00 00...
        G6CommandBuilder::new(CommandFamily::Gaming)
            .operation(&[0x03, 0x08, 0xff, 0xff, 0x00, 0x00, 0x00])
            .build(),
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firmware_query_ascii() {
        let cmd = build_firmware_query_ascii();
        assert_eq!(cmd.len(), 64);
        assert_eq!(cmd[0], 0x5a);
        assert_eq!(cmd[1], 0x07);
        assert_eq!(cmd[2], 0x01);
        assert_eq!(cmd[3], 0x02);
    }

    #[test]
    fn test_command_builder() {
        let cmd = G6CommandBuilder::new(CommandFamily::AudioControl)
            .operation(&[0x03, 0x01])
            .intermediate(IntermediateType::Audio)
            .feature(0x00)
            .build();

        assert_eq!(cmd.len(), 64);
        assert_eq!(cmd[0], 0x5a);
        assert_eq!(cmd[1], 0x11);
    }

    #[test]
    fn test_parse_firmware_response() {
        let response = b"Z\x07\x102.1.250903.1324\x00\x00\x00\x00\x00";
        let (result, _debug) = G6ResponseParser::parse(response);
        assert!(result.is_ok());
    }
}
