//! Response parsing for state reads: turns a G6 answer into a typed value,
//! with a debug trace of how it was read.
use crate::g6_spec::{EffectState, FirmwareInfo, OutputDevice};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_firmware_response() {
        let response = b"Z\x07\x102.1.250903.1324\x00\x00\x00\x00\x00";
        let (result, _debug) = G6ResponseParser::parse(response);
        assert!(result.is_ok());
    }
}
