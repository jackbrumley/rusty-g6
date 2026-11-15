/// SoundBlaster X G6 USB Device Specifications
/// Based on reverse engineering from soundblaster-x-g6-cli project
/// https://github.com/nils-skowasch/soundblaster-x-g6-cli
use serde::{Deserialize, Serialize};

// USB Device Identifiers
pub const USB_VENDOR_ID: u16 = 0x041e; // Creative Technology Ltd
pub const USB_PRODUCT_ID: u16 = 0x3256; // Sound Blaster X G6

/// Audio output types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OutputDevice {
    Speakers,
    Headphones,
}

/// Sound effect enable/disable state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EffectState {
    Enabled,
    Disabled,
}

/// Smart Volume special presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SmartVolumePreset {
    Night,
    Loud,
}

/// Scout Mode state (read-only for now)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ScoutModeState {
    Enabled,
    Disabled,
}

/// Digital Filter Types (DAC filter characteristics)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
// The variants are the DAC's filter names (fast/slow roll-off, minimum/linear
// phase), and they are serialized into the settings the UI receives.
#[allow(clippy::enum_variant_names)]
pub enum DigitalFilter {
    FastRollOffMinimumPhase, // 0x01
    SlowRollOffMinimumPhase, // 0x02
    FastRollOffLinearPhase,  // 0x04
    SlowRollOffLinearPhase,  // 0x05
}

/// Audio Configuration (0x3c family - purpose unclear, may be Windows notification)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AudioConfig {
    Unknown(u8), // Placeholder until we understand the meaning
}

/// Device firmware information (read-only)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FirmwareInfo {
    pub version: String,
    pub build: Option<String>,
}

/// EQ preamp range in dB (device range dump `0x15`: -6 to +6).
pub const EQ_PREAMP_LIMIT_DB: f32 = 6.0;

/// EQ band gain range in dB (device range dump `0x15`: -12 to +12).
pub const EQ_BAND_LIMIT_DB: f32 = 12.0;

/// The device's EQ gain step in dB.
pub const EQ_GAIN_STEP_DB: f32 = 0.5;

/// Graphic equalizer state of the active output. The G6 keeps one per
/// output (speakers and headphones).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EqualizerSettings {
    pub enabled: EffectState,
    pub preamp_db: f32,
    /// Gains in dB of the ten bands: 31, 62, 125, 250, 500 Hz and
    /// 1, 2, 4, 8, 16 kHz.
    pub band_gains_db: [f32; 10],
}

impl Default for EqualizerSettings {
    fn default() -> Self {
        Self {
            enabled: EffectState::Disabled,
            preamp_db: 0.0,
            band_gains_db: [0.0; 10],
        }
    }
}

/// Checks an EQ gain against its range (`limit_db` either way) and snaps it
/// to the device's 0.5 dB step.
pub fn validate_eq_gain(gain_db: f32, limit_db: f32) -> anyhow::Result<f32> {
    if !gain_db.is_finite() || gain_db.abs() > limit_db {
        return Err(anyhow::anyhow!(
            "EQ gain must be between -{limit_db} and +{limit_db} dB, got {gain_db}"
        ));
    }
    Ok((gain_db / EQ_GAIN_STEP_DB).round() * EQ_GAIN_STEP_DB)
}

/// G6 device settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G6Settings {
    // Controllable settings (read-write)
    pub output: OutputDevice,

    pub surround_enabled: EffectState,
    pub surround_value: u8, // 0-100

    pub crystalizer_enabled: EffectState,
    pub crystalizer_value: u8, // 0-100

    pub bass_enabled: EffectState,
    pub bass_value: u8, // 0-100

    pub smart_volume_enabled: EffectState,
    pub smart_volume_value: u8, // 0-100
    pub smart_volume_preset: Option<SmartVolumePreset>,

    pub dialog_plus_enabled: EffectState,
    pub dialog_plus_value: u8, // 0-100

    // Microphone boost (0, 10, 20, or 30 dB)
    pub microphone_boost: u8, // 0, 10, 20, or 30

    // Global SBX processing switch
    pub sbx_enabled: EffectState,

    // Digital filter setting
    pub digital_filter: Option<DigitalFilter>,

    // Audio configuration (0x3c family)
    pub audio_config: Option<AudioConfig>,

    // Read-only device information
    pub firmware_info: Option<FirmwareInfo>,
    pub scout_mode: ScoutModeState,
    pub equalizer: EqualizerSettings,

    // Device connection state
    pub is_connected: bool,
    pub last_read_time: Option<u64>, // Unix timestamp
}

impl Default for G6Settings {
    fn default() -> Self {
        Self {
            output: OutputDevice::Headphones,
            surround_enabled: EffectState::Disabled,
            surround_value: 50,
            crystalizer_enabled: EffectState::Disabled,
            crystalizer_value: 50,
            bass_enabled: EffectState::Disabled,
            bass_value: 50,
            smart_volume_enabled: EffectState::Disabled,
            smart_volume_value: 50,
            smart_volume_preset: None,
            dialog_plus_enabled: EffectState::Disabled,
            dialog_plus_value: 50,
            microphone_boost: 0,
            sbx_enabled: EffectState::Disabled,
            digital_filter: None,
            audio_config: None,
            firmware_info: None,
            scout_mode: ScoutModeState::Disabled,
            equalizer: EqualizerSettings::default(),
            is_connected: false,
            last_read_time: None,
        }
    }
}

/// Validate that a value is within 0-100 range
pub fn validate_effect_value(value: u8) -> anyhow::Result<u8> {
    if value <= 100 {
        Ok(value)
    } else {
        Err(anyhow::anyhow!(
            "Value must be between 0 and 100, got {}",
            value
        ))
    }
}

/// Protocol console message for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConsoleMessage {
    pub timestamp: u64,
    pub level: String, // "command", "response", "error", "info"
    pub text: String,
    pub details: Option<String>,
}

impl ProtocolConsoleMessage {
    pub fn new(level: &str, text: String, details: Option<String>) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level: level.to_string(),
            text,
            details,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_gain_snaps_to_half_db() {
        assert_eq!(validate_eq_gain(2.3, EQ_BAND_LIMIT_DB).unwrap(), 2.5);
        assert_eq!(validate_eq_gain(-12.0, EQ_BAND_LIMIT_DB).unwrap(), -12.0);
    }

    #[test]
    fn eq_gain_outside_range_is_rejected() {
        assert!(validate_eq_gain(6.5, EQ_PREAMP_LIMIT_DB).is_err());
        assert!(validate_eq_gain(f32::NAN, EQ_BAND_LIMIT_DB).is_err());
    }
}
