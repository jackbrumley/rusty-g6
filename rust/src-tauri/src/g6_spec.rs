/// SoundBlaster X G6 USB Device Specifications
/// Based on reverse engineering from soundblaster-x-g6-cli project
/// https://github.com/nils-skowasch/soundblaster-x-g6-cli

use serde::{Deserialize, Serialize};

// USB Device Identifiers
pub const USB_VENDOR_ID: u16 = 0x041e; // Creative Technology Ltd
pub const USB_PRODUCT_ID: u16 = 0x3256; // Sound Blaster X G6

// Device Configuration
pub const USB_INTERFACE: i32 = 0;
pub const USB_TIMEOUT_MS: u64 = 1000;

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

/// Device firmware information (read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareInfo {
    pub version: String,
    pub build: Option<String>,
}

/// Equalizer band configuration (read-only for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerBand {
    pub frequency: f32,
    pub gain: f32,
}

/// Complete equalizer configuration (read-only for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqualizerConfig {
    pub enabled: EffectState,
    pub bands: Vec<EqualizerBand>,
}

/// Extended audio effect parameters (read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedAudioParams {
    pub param_0x0a: Option<f32>,
    pub param_0x0b: Option<f32>,
    pub param_0x0c: Option<f32>,
    pub param_0x0d: Option<f32>,
    pub param_0x0e: Option<f32>,
    pub param_0x0f: Option<f32>,
    pub param_0x10: Option<f32>,
    pub param_0x11: Option<f32>,
    pub param_0x12: Option<f32>,
    pub param_0x13: Option<f32>,
    pub param_0x14: Option<f32>,
    pub param_0x1a: Option<f32>,
    pub param_0x1b: Option<f32>,
    pub param_0x1c: Option<f32>,
    pub param_0x1d: Option<f32>,
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
    
    // Read-only device information
    pub firmware_info: Option<FirmwareInfo>,
    pub scout_mode: ScoutModeState,
    pub equalizer: Option<EqualizerConfig>,
    pub extended_params: Option<ExtendedAudioParams>,
    
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
            firmware_info: None,
            scout_mode: ScoutModeState::Disabled,
            equalizer: None,
            extended_params: None,
            is_connected: false,
            last_read_time: None,
        }
    }
}

impl Default for EqualizerConfig {
    fn default() -> Self {
        Self {
            enabled: EffectState::Disabled,
            bands: vec![
                EqualizerBand { frequency: 31.0, gain: 0.0 },
                EqualizerBand { frequency: 62.0, gain: 0.0 },
                EqualizerBand { frequency: 125.0, gain: 0.0 },
                EqualizerBand { frequency: 250.0, gain: 0.0 },
                EqualizerBand { frequency: 500.0, gain: 0.0 },
                EqualizerBand { frequency: 1000.0, gain: 0.0 },
                EqualizerBand { frequency: 2000.0, gain: 0.0 },
                EqualizerBand { frequency: 4000.0, gain: 0.0 },
                EqualizerBand { frequency: 8000.0, gain: 0.0 },
                EqualizerBand { frequency: 16000.0, gain: 0.0 },
            ],
        }
    }
}

impl Default for ExtendedAudioParams {
    fn default() -> Self {
        Self {
            param_0x0a: None,
            param_0x0b: None,
            param_0x0c: None,
            param_0x0d: None,
            param_0x0e: None,
            param_0x0f: None,
            param_0x10: None,
            param_0x11: None,
            param_0x12: None,
            param_0x13: None,
            param_0x14: None,
            param_0x1a: None,
            param_0x1b: None,
            param_0x1c: None,
            param_0x1d: None,
        }
    }
}


/// Validate that a value is within 0-100 range
pub fn validate_effect_value(value: u8) -> anyhow::Result<u8> {
    if value <= 100 {
        Ok(value)
    } else {
        Err(anyhow::anyhow!("Value must be between 0 and 100, got {}", value))
    }
}
