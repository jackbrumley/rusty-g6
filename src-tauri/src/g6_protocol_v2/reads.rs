//! Read (query) command builders.
use super::{CommandFamily, G6CommandBuilder};

// ============================================================================
// READ COMMANDS
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

    commands.extend(super::equalizer::build_eq_read_commands());

    commands
}

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
}
