//! Voice morph in the recording DSP family (0x95): the switch `95:0a` and
//! its eight values `95:0b`-`95:12` (`VoiceMorphValue` order).
//!
//! Decoded from Creative's parameter table (CTHIDRpA.dll), which also holds
//! the captured mic features (noise reduction `95:04`, mic EQ `95:14`-`1b`).
//! Confirmed on a G6 (firmware 2.1.250903.1324): reads of all nine answer
//! with values inside the table's ranges. The G6 has no preset command;
//! a preset is its eight values, written one by one.
use super::{dsp_read, dsp_write, dsp_write_and_read, DspFamily};
use crate::g6_spec::{VoiceMorphValue, VOICE_MORPH_VALUE_COUNT};

const ENABLE: u8 = 0x0a;
const FIRST_VALUE: u8 = 0x0b;

/// What a recording DSP id reports, if it is a voice morph id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMorphParameter {
    Enable,
    Value(VoiceMorphValue),
}

pub fn parameter_for_id(id: u8) -> Option<VoiceMorphParameter> {
    if id == ENABLE {
        return Some(VoiceMorphParameter::Enable);
    }
    let index = usize::from(id.checked_sub(FIRST_VALUE)?);
    VoiceMorphValue::ALL
        .get(index)
        .copied()
        .map(VoiceMorphParameter::Value)
}

fn value_ids() -> impl Iterator<Item = u8> {
    (0..VOICE_MORPH_VALUE_COUNT as u8).map(|index| FIRST_VALUE + index)
}

/// Reads of the switch and all eight values.
pub fn build_voice_morph_read_commands() -> Vec<Vec<u8>> {
    std::iter::once(ENABLE)
        .chain(value_ids())
        .map(|id| dsp_read(DspFamily::Recording, id))
        .collect()
}

/// Switch voice morph on or off, then read it back.
pub fn build_set_voice_morph_enabled(enabled: bool) -> Vec<Vec<u8>> {
    dsp_write_and_read(
        DspFamily::Recording,
        ENABLE,
        if enabled { 1.0 } else { 0.0 },
    )
}

/// Set all eight values (a preset): one write per value; the G6 confirms
/// each with a status report.
pub fn build_set_voice_morph_values(values: &[f32; VOICE_MORPH_VALUE_COUNT]) -> Vec<Vec<u8>> {
    value_ids()
        .zip(values)
        .map(|(id, value)| dsp_write(DspFamily::Recording, id, *value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_map_to_parameters() {
        assert_eq!(parameter_for_id(0x0a), Some(VoiceMorphParameter::Enable));
        assert_eq!(
            parameter_for_id(0x0b),
            Some(VoiceMorphParameter::Value(VoiceMorphValue::Formant1Hz))
        );
        assert_eq!(
            parameter_for_id(0x12),
            Some(VoiceMorphParameter::Value(VoiceMorphValue::ContourDepth))
        );
        assert_eq!(parameter_for_id(0x09), None);
        assert_eq!(parameter_for_id(0x13), None);
    }

    #[test]
    fn switch_write_matches_the_dsp_layout() {
        let commands = build_set_voice_morph_enabled(true);
        assert_eq!(
            &commands[0][..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x95, 0x0a, 0x00, 0x00, 0x80, 0x3f]
        );
        assert_eq!(&commands[1][..6], &[0x5a, 0x11, 0x03, 0x01, 0x95, 0x0a]);
    }

    #[test]
    fn values_write_in_id_order() {
        let commands =
            build_set_voice_morph_values(&[400.0, 1400.0, 2000.0, 2.0, 1.0, 1.0, 0.0, 0.0]);
        assert_eq!(commands.len(), 8);
        assert_eq!(&commands[0][4..10], &[0x95, 0x0b, 0x00, 0x00, 0xc8, 0x43]);
        assert_eq!(&commands[3][4..10], &[0x95, 0x0e, 0x00, 0x00, 0x00, 0x40]);
        assert_eq!(commands[7][5], 0x12);
    }
}
