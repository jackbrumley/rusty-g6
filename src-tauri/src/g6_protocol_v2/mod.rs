/// USB Protocol V2 - Unified Abstraction Layer
/// Packet framing and the command builder live here; parsing, reads and
/// writes each have their own module.
pub mod equalizer;
pub mod events;
pub mod parser;
pub mod reads;
pub mod writes;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
