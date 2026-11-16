//! Mic processing in the recording DSP family (0x95), including the mic EQ's
//! eight band gains `95:14`-`95:1b` (dB, captured with each of Creative's
//! mic EQ presets by soundblaster-x-g6-cli and linuxblaster_control).
//!
//! Captured (linuxblaster_control `set_noise_reduction_*`,
//! `set_mic_noise_reduction_level-*`): noise reduction `95:04` and its level
//! `95:05`. Decoded but not yet captured: echo cancellation `95:00`, mic
//! smart volume `95:2c` and the mic EQ switch `95:13`. linuxblaster's
//! protocol reference and Creative's own parameter table (CTHIDRpA.dll) agree
//! on all three; they are on/off switches, so a hardware test confirms them.
use super::{dsp_read, dsp_write, dsp_write_and_read, DspFamily};
use crate::g6_spec::MicSwitch;

pub const ALL_MIC_SWITCHES: [MicSwitch; 4] = [
    MicSwitch::NoiseReduction,
    MicSwitch::EchoCancellation,
    MicSwitch::SmartVolume,
    MicSwitch::Equalizer,
];

const MIC_EQ_FIRST_BAND: u8 = 0x14;
pub const MIC_EQ_BAND_COUNT: usize = 8;

/// Noise reduction strength: the device takes 0.0-0.5.
const NOISE_REDUCTION_LEVEL: u8 = 0x05;
const NOISE_REDUCTION_DEVICE_MAX: f32 = 0.5;

fn switch_id(switch: MicSwitch) -> u8 {
    match switch {
        MicSwitch::EchoCancellation => 0x00,
        MicSwitch::NoiseReduction => 0x04,
        MicSwitch::Equalizer => 0x13,
        MicSwitch::SmartVolume => 0x2c,
    }
}

/// What a recording DSP id reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicParameter {
    Switch(MicSwitch),
    NoiseReductionLevel,
    /// Mic EQ band 0 (lowest) to 7.
    EqBand(usize),
}

pub fn parameter_for_id(id: u8) -> Option<MicParameter> {
    if id == NOISE_REDUCTION_LEVEL {
        return Some(MicParameter::NoiseReductionLevel);
    }
    if let Some(band) = id.checked_sub(MIC_EQ_FIRST_BAND).map(usize::from) {
        if band < MIC_EQ_BAND_COUNT {
            return Some(MicParameter::EqBand(band));
        }
    }
    ALL_MIC_SWITCHES
        .into_iter()
        .find(|switch| switch_id(*switch) == id)
        .map(MicParameter::Switch)
}

/// Noise reduction level 0-100 from the device's 0.0-0.5.
pub fn noise_reduction_level_from_device(value: f32) -> u8 {
    (value / NOISE_REDUCTION_DEVICE_MAX * 100.0)
        .round()
        .clamp(0.0, 100.0) as u8
}

fn noise_reduction_level_to_device(level: u8) -> f32 {
    f32::from(level) / 100.0 * NOISE_REDUCTION_DEVICE_MAX
}

/// Reads of every switch and the noise reduction level.
pub fn build_mic_read_commands() -> Vec<Vec<u8>> {
    ALL_MIC_SWITCHES
        .into_iter()
        .map(switch_id)
        .chain([NOISE_REDUCTION_LEVEL])
        .chain(mic_eq_band_ids())
        .map(|id| dsp_read(DspFamily::Recording, id))
        .collect()
}

fn mic_eq_band_ids() -> impl Iterator<Item = u8> {
    (0..MIC_EQ_BAND_COUNT as u8).map(|band| MIC_EQ_FIRST_BAND + band)
}

/// Set all eight mic EQ bands (a preset): one write per band; the G6
/// confirms each with a status report.
pub fn build_set_mic_eq_bands(gains_db: &[f32; MIC_EQ_BAND_COUNT]) -> Vec<Vec<u8>> {
    mic_eq_band_ids()
        .zip(gains_db)
        .map(|(id, gain_db)| dsp_write(DspFamily::Recording, id, *gain_db))
        .collect()
}

/// Switch a mic feature on or off, then read it back.
pub fn build_set_mic_switch(switch: MicSwitch, enabled: bool) -> Vec<Vec<u8>> {
    dsp_write_and_read(
        DspFamily::Recording,
        switch_id(switch),
        if enabled { 1.0 } else { 0.0 },
    )
}

/// Set the noise reduction level (0-100), then read it back.
pub fn build_set_noise_reduction_level(level: u8) -> Vec<Vec<u8>> {
    dsp_write_and_read(
        DspFamily::Recording,
        NOISE_REDUCTION_LEVEL,
        noise_reduction_level_to_device(level),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_reduction_write_matches_capture() {
        // set_noise_reduction_on-01: 5a 12 07 01 95 04 00 00 80 3f
        let commands = build_set_mic_switch(MicSwitch::NoiseReduction, true);
        assert_eq!(
            &commands[0][..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x95, 0x04, 0x00, 0x00, 0x80, 0x3f]
        );
    }

    #[test]
    fn noise_reduction_level_uses_half_scale() {
        // 40 % is 0.2 on the device (capture: cd cc 4c 3e).
        let commands = build_set_noise_reduction_level(40);
        assert_eq!(&commands[0][6..10], &0.2f32.to_le_bytes());
        assert_eq!(noise_reduction_level_from_device(0.2), 40);
    }

    #[test]
    fn mic_eq_preset_matches_capture() {
        // Preset 1, band 1: 5a 12 07 01 95 14 00 00 40 c0 (-3.0 dB)
        let commands = build_set_mic_eq_bands(&[-3.0, -4.0, 0.0, 2.0, 3.0, -3.0, 4.0, 5.0]);
        assert_eq!(commands.len(), 8);
        assert_eq!(
            &commands[0][..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x95, 0x14, 0x00, 0x00, 0x40, 0xc0]
        );
        assert_eq!(commands[7][5], 0x1b);
    }

    #[test]
    fn ids_map_back_to_parameters() {
        assert_eq!(
            parameter_for_id(0x2c),
            Some(MicParameter::Switch(MicSwitch::SmartVolume))
        );
        assert_eq!(
            parameter_for_id(0x05),
            Some(MicParameter::NoiseReductionLevel)
        );
        assert_eq!(parameter_for_id(0x14), Some(MicParameter::EqBand(0)));
        assert_eq!(parameter_for_id(0x1b), Some(MicParameter::EqBand(7)));
        assert_eq!(parameter_for_id(0x1c), None);
    }
}
