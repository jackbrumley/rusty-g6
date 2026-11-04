/// USB Protocol V2 - Unified Abstraction Layer
/// This module provides a cleaner, more maintainable protocol implementation
/// Built alongside the original g6_protocol.rs to allow gradual migration
use crate::g6_spec::{EffectState, FirmwareInfo, OutputDevice};

mod events;
pub use events::{DeviceEvent, G6EventParser};

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
    FirmwareQuery = 0x07, // Firmware version (discovered ASCII mode)
    AudioControl = 0x11,  // Primary protocol - Read/Write SBX & EQ
    DataControl = 0x12,   // Audio effect data commands
    Gaming = 0x26,        // Scout/SBX Mode switches
    Routing = 0x2c,       // Output switching
    AudioConfig = 0x3c,   // Audio configuration (microphone boost)
}

// The other families seen in captures (0x02, 0x05, 0x10, 0x15, 0x20, 0x30,
// 0x3a, 0x6c) are only named in `describe_packet`: nothing sends them.

impl CommandFamily {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

// ============================================================================
// RESPONSE FORMAT
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedResponse {
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
}

// ============================================================================
// COMMAND BUILDER
// ============================================================================

#[derive(Debug)]
pub struct G6CommandBuilder {
    family: CommandFamily,
    operation_bytes: Vec<u8>,
    feature: Option<u8>,
    value: Option<Vec<u8>>,
}

impl G6CommandBuilder {
    /// Create a new command builder for the specified family
    pub fn new(family: CommandFamily) -> Self {
        Self {
            family,
            operation_bytes: Vec::new(),
            feature: None,
            value: None,
        }
    }

    /// Set operation type (single byte or multi-byte)
    pub fn operation(mut self, bytes: &[u8]) -> Self {
        self.operation_bytes = bytes.to_vec();
        self
    }

    /// Set feature ID
    pub fn feature(mut self, feature: u8) -> Self {
        self.feature = Some(feature);
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
            (0x2c, 0x05) => {
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

        // Find null terminator (none: no version)
        let version_end = response[3..]
            .iter()
            .position(|&byte| byte == 0)
            .map_or(3, |offset| 3 + offset);

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
    /// Handles the 0x2c 0x05 response format (Live State)
    /// Format: 5a 2c 05 01 [VALUE] 00... where VALUE is at index 4
    fn parse_output_config(
        response: &[u8],
        debug: &mut ProtocolDebugInfo,
    ) -> Result<ParsedResponse, String> {
        if response.len() < 5 {
            return Err("Response too short for output config".to_string());
        }

        // Log full packet for debugging
        let full_hex: String = response
            .iter()
            .take(response.len().min(20))
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        debug
            .parsing_attempts
            .push(format!("Full packet (first 20 bytes): {}", full_hex));

        if response[1] != 0x2c || response[2] != 0x05 {
            return Err("Invalid packet header for output config".to_string());
        }

        debug.parsing_attempts.push(format!(
            "Read response format (0x2c 0x05), index 4 = 0x{:02x}",
            response[4]
        ));

        match response[4] {
            0x04 => {
                debug
                    .parsing_attempts
                    .push("Read response: Index 4 = 0x04 → Headphones".to_string());
                Ok(ParsedResponse::OutputDevice(OutputDevice::Headphones))
            }
            0x02 => {
                debug
                    .parsing_attempts
                    .push("Read response: Index 4 = 0x02 → Speakers".to_string());
                Ok(ParsedResponse::OutputDevice(OutputDevice::Speakers))
            }
            val => {
                debug
                    .parsing_attempts
                    .push(format!("Unexpected value at index 4: 0x{:02x}", val));
                Err(format!("Unknown output device value: 0x{:02x}", val))
            }
        }
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
// PACKET DESCRIPTION (for logging)
// ============================================================================

/// Describe a packet for logging purposes
pub fn describe_packet(packet: &[u8]) -> String {
    if packet.len() < 2 {
        return "Invalid packet (too short)".to_string();
    }

    let family = packet[1];

    match family {
        0x05 => "Identification".to_string(),
        0x07 => "Firmware Query".to_string(),
        0x10 => "Hardware Status".to_string(),
        0x11 => "Audio Control".to_string(),
        0x12 => "Data Control".to_string(),
        0x15 => "Batch Control".to_string(),
        0x20 => "Processing".to_string(),
        0x26 => "Gaming Mode (SBX/Scout)".to_string(),
        0x2c => "Routing (Output)".to_string(),
        0x30 => "Device Config".to_string(),
        0x3a => "System Config".to_string(),
        0x3c => "Audio Config".to_string(),
        0x6c => "Digital Filter".to_string(),
        _ => format!("Unknown Command/Report (Type {:02x})", family),
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

/// Build audio effect read command
pub fn build_audio_effect_read(feature: u8) -> Vec<u8> {
    // Corrected based on packet capture: 5a 11 03 01 96 [FEATURE]
    // 0x96 is the Audio Type byte. We include it in operation to avoid
    // G6CommandBuilder's 2-byte intermediate insertion.
    G6CommandBuilder::new(CommandFamily::AudioControl)
        .operation(&[0x03, 0x01, 0x96])
        .feature(feature)
        .build()
}

/// Build gaming mode (Scout/SBX) read command
/// Corrected based on packet capture: 5a 26 03 08 ff ff ...
pub fn build_gaming_mode_read() -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::Gaming)
        .operation(&[0x03, 0x08, 0xff, 0xff])
        .build()
}

/// Build output config read command
/// Uses 0x01 0x01 which returns the actual output state event
/// Device responds with event format: 5a 2c 05 01 [OUTPUT_VALUE] 00 00...
pub fn build_output_config_read() -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::Routing)
        .operation(&[0x01, 0x01])
        .build()
}

/// Build all read commands for complete device state
/// OPTIMIZATION: Only queries commands that actually work!
/// Audio effect reads fail (device responds with wrong packet type),
/// so we skip them and rely on the event listener to update state.
pub fn build_read_all_state_commands() -> Vec<Vec<u8>> {
    let mut commands = Vec::new();

    // ✅ Firmware query - WORKS
    commands.push(build_firmware_query_ascii());

    // ✅ Output configuration - WORKS
    commands.push(build_output_config_read());

    // EXPERIMENTAL: Gaming Mode (Scout/SBX)
    commands.push(build_gaming_mode_read());

    // EXPERIMENTAL: Audio effects (0x00-0x1D) with new 0x01 0x01 op code
    // Iterate common features
    let features = [
        0x00, // Surround Toggle
        0x01, // Surround Value
        0x02, // Dialog+ Toggle
        0x03, // Dialog+ Value
        0x04, // Smart Vol Toggle
        0x05, // Smart Vol Value
        0x06, // Smart Vol Preset
        0x07, // Crystalizer Toggle
        0x08, // Crystalizer Value
        0x18, // Bass Toggle
        0x19, // Bass Value
    ];

    for feature in features.iter() {
        commands.push(build_audio_effect_read(*feature));
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

// Audio Effect Feature IDs
const FEATURE_SURROUND_TOGGLE: u8 = 0x00;
const FEATURE_SURROUND_VALUE: u8 = 0x01;
const FEATURE_DIALOG_PLUS_TOGGLE: u8 = 0x02;
const FEATURE_DIALOG_PLUS_VALUE: u8 = 0x03;
const FEATURE_SMART_VOLUME_TOGGLE: u8 = 0x04;
const FEATURE_SMART_VOLUME_VALUE: u8 = 0x05;
const FEATURE_CRYSTALIZER_TOGGLE: u8 = 0x07;
const FEATURE_CRYSTALIZER_VALUE: u8 = 0x08;
const FEATURE_BASS_TOGGLE: u8 = 0x18;

// Gaming Mode Feature IDs (0x26 family)
const FEATURE_SBX_MODE: u8 = 0x01;
const FEATURE_SCOUT_MODE: u8 = 0x02;

/// Build command to set bass toggle (on/off)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - matches official software USB capture
/// The official software uses DataControl family (0x12) with Write operation (0x07), NOT AudioControl (0x11 0x08)
pub fn build_set_bass_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        // DATA command - 5a 12 07 01 96 18 [FLOAT] ...
        // This is the ACTUAL command from USB packet capture
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_BASS_TOGGLE])
            .float_value(value)
            .build(),
        // READ command - 5a 11 03 01 96 18 ... (to verify/confirm change)
        build_audio_effect_read(FEATURE_BASS_TOGGLE),
    ]
}

/// Build command to set surround toggle (on/off)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_surround_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_SURROUND_TOGGLE])
            .float_value(value)
            .build(),
        build_audio_effect_read(FEATURE_SURROUND_TOGGLE),
    ]
}

/// Build command to set surround value (0-100)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_surround_value(value: u8) -> Vec<Vec<u8>> {
    let float_value = (value as f32) / 100.0;

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_SURROUND_VALUE])
            .float_value(float_value)
            .build(),
        build_audio_effect_read(FEATURE_SURROUND_VALUE),
    ]
}

/// Build command to set crystalizer toggle (on/off)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_crystalizer_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_CRYSTALIZER_TOGGLE])
            .float_value(value)
            .build(),
        build_audio_effect_read(FEATURE_CRYSTALIZER_TOGGLE),
    ]
}

/// Build command to set crystalizer value (0-100)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_crystalizer_value(value: u8) -> Vec<Vec<u8>> {
    let float_value = (value as f32) / 100.0;

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_CRYSTALIZER_VALUE])
            .float_value(float_value)
            .build(),
        build_audio_effect_read(FEATURE_CRYSTALIZER_VALUE),
    ]
}

/// Build command to set smart volume toggle (on/off)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_smart_volume_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_SMART_VOLUME_TOGGLE])
            .float_value(value)
            .build(),
        build_audio_effect_read(FEATURE_SMART_VOLUME_TOGGLE),
    ]
}

/// Build command to set smart volume value (0-100)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_smart_volume_value(value: u8) -> Vec<Vec<u8>> {
    let float_value = (value as f32) / 100.0;

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_SMART_VOLUME_VALUE])
            .float_value(float_value)
            .build(),
        build_audio_effect_read(FEATURE_SMART_VOLUME_VALUE),
    ]
}

/// Build command to set dialog plus toggle (on/off)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_dialog_plus_toggle(enabled: bool) -> Vec<Vec<u8>> {
    let value = if enabled { 1.0f32 } else { 0.0f32 };

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_DIALOG_PLUS_TOGGLE])
            .float_value(value)
            .build(),
        build_audio_effect_read(FEATURE_DIALOG_PLUS_TOGGLE),
    ]
}

/// Build command to set dialog plus value (0-100)
/// Command format: DATA (0x12 0x07) + READ (0x11 0x03) - fixed to match bass implementation
pub fn build_set_dialog_plus_value(value: u8) -> Vec<Vec<u8>> {
    let float_value = (value as f32) / 100.0;

    vec![
        G6CommandBuilder::new(CommandFamily::DataControl)
            .operation(&[0x07, 0x01, 0x96, FEATURE_DIALOG_PLUS_VALUE])
            .float_value(float_value)
            .build(),
        build_audio_effect_read(FEATURE_DIALOG_PLUS_VALUE),
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
// WRITE COMMANDS - MICROPHONE BOOST (Phase 9 - Packet Capture Verified!)
// ============================================================================

/// Build command to set microphone boost level
/// Command format: DATA (0x3c 04) + COMMIT (0x3c 02 01)
/// Discovered from packet capture: enable-microphone-boost-up-to-30-db-max.txt
///
/// Values:
/// - 0 dB = 0x00 (no boost)
/// - 10 dB = 0x0a
/// - 20 dB = 0x14
/// - 30 dB = 0x1e (maximum)
pub fn build_set_microphone_boost(db_value: u8) -> Vec<Vec<u8>> {
    // Validate: only 0, 10, 20, or 30 dB allowed
    let validated_value = match db_value {
        0 => 0x00,
        10 => 0x0a,
        20 => 0x14,
        30 => 0x1e,
        _ => {
            // Default to nearest valid value
            if db_value < 5 {
                0x00
            } else if db_value < 15 {
                0x0a
            } else if db_value < 25 {
                0x14
            } else {
                0x1e
            }
        }
    };

    vec![
        // DATA command - 5a 3c 04 00 00 02 [DB_VALUE] 00 00...
        G6CommandBuilder::new(CommandFamily::AudioConfig)
            .operation(&[0x04, 0x00, 0x00, 0x02, validated_value])
            .build(),
        // COMMIT command - 5a 3c 02 01 00 00...
        G6CommandBuilder::new(CommandFamily::AudioConfig)
            .operation(&[0x02, 0x01])
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
            .operation(&[0x03, 0x01, 0x96])
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
