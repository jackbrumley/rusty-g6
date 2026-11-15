//! SBX effects: each has an on/off parameter and a level parameter in the
//! playback DSP family. Toggles are f32 0.0 / 1.0; levels are 0.0-1.0,
//! shown as 0-100.
use super::{dsp_read, dsp_write_and_read};
use crate::g6_spec::SbxEffect;

pub const ALL_EFFECTS: [SbxEffect; 5] = [
    SbxEffect::Surround,
    SbxEffect::Crystalizer,
    SbxEffect::Bass,
    SbxEffect::SmartVolume,
    SbxEffect::DialogPlus,
];

/// Which of an effect's two parameters a DSP id is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectParameter {
    Toggle,
    Level,
}

fn toggle_id(effect: SbxEffect) -> u8 {
    match effect {
        SbxEffect::Surround => 0x00,
        SbxEffect::DialogPlus => 0x02,
        SbxEffect::SmartVolume => 0x04,
        SbxEffect::Crystalizer => 0x07,
        SbxEffect::Bass => 0x18,
    }
}

/// Every effect's level id directly follows its toggle id.
fn level_id(effect: SbxEffect) -> u8 {
    toggle_id(effect) + 1
}

/// The effect and parameter a playback DSP id belongs to.
pub fn parameter_for_id(id: u8) -> Option<(SbxEffect, EffectParameter)> {
    ALL_EFFECTS.into_iter().find_map(|effect| {
        if id == toggle_id(effect) {
            Some((effect, EffectParameter::Toggle))
        } else if id == level_id(effect) {
            Some((effect, EffectParameter::Level))
        } else {
            None
        }
    })
}

/// Level 0-100 as the device's 0.0-1.0.
pub fn level_to_device(level: u8) -> f32 {
    f32::from(level) / 100.0
}

/// The device's 0.0-1.0 as level 0-100.
pub fn level_from_device(value: f32) -> u8 {
    (value * 100.0).round().clamp(0.0, 100.0) as u8
}

/// Reads of every effect's toggle and level.
pub fn build_effect_read_commands() -> Vec<Vec<u8>> {
    ALL_EFFECTS
        .into_iter()
        .flat_map(|effect| [dsp_read(toggle_id(effect)), dsp_read(level_id(effect))])
        .collect()
}

/// Switch `effect` on or off, then read it back.
pub fn build_set_effect_enabled(effect: SbxEffect, enabled: bool) -> Vec<Vec<u8>> {
    dsp_write_and_read(toggle_id(effect), if enabled { 1.0 } else { 0.0 })
}

/// Set `effect`'s level (0-100), then read it back.
pub fn build_set_effect_level(effect: SbxEffect, level: u8) -> Vec<Vec<u8>> {
    dsp_write_and_read(level_id(effect), level_to_device(level))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_match_the_protocol_reference() {
        // Toggle/level pairs: surround 00/01, dialog+ 02/03, smart volume
        // 04/05, crystalizer 07/08, bass 18/19.
        assert_eq!(
            parameter_for_id(0x08),
            Some((SbxEffect::Crystalizer, EffectParameter::Level))
        );
        assert_eq!(
            parameter_for_id(0x18),
            Some((SbxEffect::Bass, EffectParameter::Toggle))
        );
        assert_eq!(parameter_for_id(0x06), None);
        assert_eq!(parameter_for_id(0x09), None);
    }

    #[test]
    fn level_write_uses_fraction() {
        let commands = build_set_effect_level(SbxEffect::Surround, 50);
        assert_eq!(&commands[0][5..10], &[0x01, 0x00, 0x00, 0x00, 0x3f]);
        assert_eq!(level_from_device(0.5), 50);
    }
}
