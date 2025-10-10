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

/// G6 device settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct G6Settings {
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
