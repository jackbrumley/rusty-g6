//! Write command builders: output, SBX effects, SBX/Scout mode, mic boost.
use super::reads::build_audio_effect_read;
use super::{CommandFamily, G6CommandBuilder};
use crate::g6_spec::OutputDevice;

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
