//! The Sound BlasterX G6's settings as Rusty G6 models them. Protocol bytes
//! live in `g6_protocol`; this module has no knowledge of them.
use serde::{Deserialize, Serialize};

pub const USB_VENDOR_ID: u16 = 0x041e; // Creative Technology Ltd
pub const USB_PRODUCT_ID: u16 = 0x3256; // Sound BlasterX G6

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputDevice {
    Speakers,
    Headphones,
}

/// The G6's recording source: its `PCM Capture Source` ALSA control on
/// Linux (item names in `g6_mixer::recording_source`). The kernel starts it on Line
/// In, so the mic is barely audible until External Mic is selected.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RecordingSource {
    LineIn,
    #[default]
    ExternalMic,
    SpdifIn,
    WhatUHear,
}

/// The G6's mic level (ALSA `External Mic` capture volume, Linux) and its
/// range, in dB.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MicLevel {
    pub level_db: f32,
    pub min_db: f32,
    pub max_db: f32,
}

/// Mic monitoring (hearing yourself, Linux): on/off and level in dB, with
/// the level's range.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MicMonitoring {
    pub enabled: bool,
    pub level_db: f32,
    pub min_db: f32,
    pub max_db: f32,
}

/// The user's mic monitoring choice, stored and re-applied (Linux).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MicMonitoringChoice {
    pub enabled: bool,
    pub level_db: f32,
}

/// The SBX effects that each have an on/off switch and a 0-100 level. The
/// serialized names are the keys of `SbxEffects`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SbxEffect {
    Surround,
    Crystalizer,
    Bass,
    SmartVolume,
    DialogPlus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectSettings {
    pub enabled: bool,
    /// 0-100.
    pub level: u8,
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            level: 50,
        }
    }
}

/// One `EffectSettings` per `SbxEffect`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SbxEffects {
    pub surround: EffectSettings,
    pub crystalizer: EffectSettings,
    pub bass: EffectSettings,
    pub smart_volume: EffectSettings,
    pub dialog_plus: EffectSettings,
}

impl SbxEffects {
    pub fn get_mut(&mut self, effect: SbxEffect) -> &mut EffectSettings {
        match effect {
            SbxEffect::Surround => &mut self.surround,
            SbxEffect::Crystalizer => &mut self.crystalizer,
            SbxEffect::Bass => &mut self.bass,
            SbxEffect::SmartVolume => &mut self.smart_volume,
            SbxEffect::DialogPlus => &mut self.dialog_plus,
        }
    }
}

/// Checks an effect level (0-100).
pub fn validate_effect_level(level: u8) -> anyhow::Result<u8> {
    if level <= 100 {
        Ok(level)
    } else {
        Err(anyhow::anyhow!(
            "Effect level must be between 0 and 100, got {level}"
        ))
    }
}

/// The G6's mic processing switches (recording DSP).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MicSwitch {
    NoiseReduction,
    EchoCancellation,
    SmartVolume,
    Equalizer,
}

/// The G6's mic processing state.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct MicProcessing {
    pub noise_reduction: bool,
    /// 0-100.
    pub noise_reduction_level: u8,
    pub echo_cancellation: bool,
    pub smart_volume: bool,
    pub equalizer: bool,
    /// The mic EQ's band gains in dB, lowest band first.
    pub equalizer_gains_db: [f32; 8],
}

impl MicProcessing {
    pub fn switch_mut(&mut self, switch: MicSwitch) -> &mut bool {
        match switch {
            MicSwitch::NoiseReduction => &mut self.noise_reduction,
            MicSwitch::EchoCancellation => &mut self.echo_cancellation,
            MicSwitch::SmartVolume => &mut self.smart_volume,
            MicSwitch::Equalizer => &mut self.equalizer,
        }
    }
}

/// On/off device features outside the DSP.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceFlag {
    /// Bypasses all processing (SBX, EQ) for bit-perfect playback.
    DirectMode,
    /// Bit-perfect S/PDIF output (speakers only).
    SpdifOutDirect,
}

/// The decoder's dynamic range mode for Dolby/DTS content.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DecoderMode {
    /// Full dynamic range.
    Full,
    Normal,
    /// Compressed range, for quiet listening.
    Night,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// The G6's logo lighting: on/off and a static colour.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightingSettings {
    pub enabled: bool,
    pub color: RgbColor,
}

/// The DAC's digital reconstruction filter (CS43131). The G6 lists all five
/// in its `0x6c` filter list; protocol bytes are in `g6_protocol::dac_filter`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DigitalFilter {
    FastRollOffMinimumPhase,
    SlowRollOffMinimumPhase,
    NonOversampling,
    FastRollOffLinearPhase,
    SlowRollOffLinearPhase,
}

/// The mic boost steps in dB.
pub const MICROPHONE_BOOST_STEPS_DB: [u8; 4] = [0, 10, 20, 30];

/// EQ preamp range in dB (device range dump `0x15`: -6 to +6).
pub const EQ_PREAMP_LIMIT_DB: f32 = 6.0;

/// EQ band gain range in dB (device range dump `0x15`: -12 to +12).
pub const EQ_BAND_LIMIT_DB: f32 = 12.0;

/// The device's EQ gain step in dB.
pub const EQ_GAIN_STEP_DB: f32 = 0.5;

/// Graphic equalizer state of the active output. The G6 keeps one per
/// output (speakers and headphones).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EqualizerSettings {
    pub enabled: bool,
    pub preamp_db: f32,
    /// Gains in dB of the ten bands: 31, 62, 125, 250, 500 Hz and
    /// 1, 2, 4, 8, 16 kHz.
    pub band_gains_db: [f32; 10],
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

/// What Rusty G6 knows of the device's state. Filled by state reads and
/// device reports (`DeviceEvent::apply_to`) and by confirmed writes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct G6Settings {
    pub output: OutputDevice,
    /// The SBX master switch.
    pub sbx_enabled: bool,
    pub scout_mode_enabled: bool,
    pub effects: SbxEffects,
    pub equalizer: EqualizerSettings,
    pub digital_filter: Option<DigitalFilter>,
    pub decoder_mode: Option<DecoderMode>,
    pub direct_mode: bool,
    pub spdif_out_direct: bool,
    pub lighting: LightingSettings,
    /// One of `MICROPHONE_BOOST_STEPS_DB`; `None` until read.
    pub microphone_boost_db: Option<u8>,
    pub mic_processing: MicProcessing,
    pub firmware_version: Option<String>,
    /// Unix time of the last state read that got answers.
    pub last_read_time: Option<u64>,
}

impl Default for G6Settings {
    fn default() -> Self {
        Self {
            output: OutputDevice::Headphones,
            sbx_enabled: false,
            scout_mode_enabled: false,
            effects: SbxEffects::default(),
            equalizer: EqualizerSettings::default(),
            digital_filter: None,
            decoder_mode: None,
            direct_mode: false,
            spdif_out_direct: false,
            lighting: LightingSettings::default(),
            microphone_boost_db: None,
            mic_processing: MicProcessing::default(),
            firmware_version: None,
            last_read_time: None,
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

    #[test]
    fn effect_keys_match_effect_names() {
        let json = serde_json::to_value(SbxEffects::default()).unwrap();
        for effect in [
            "surround",
            "crystalizer",
            "bass",
            "smart_volume",
            "dialog_plus",
        ] {
            let name: SbxEffect = serde_json::from_value(serde_json::json!(effect)).unwrap();
            assert!(json.get(effect).is_some(), "{name:?}");
        }
    }
}
