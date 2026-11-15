//! Graphic EQ commands: a toggle, a preamp and ten bands in the playback DSP
//! family (0x96). Bytes from linuxblaster_control's hardware-verified
//! protocol reference (captures `set_eq_on`, `set_eq_off`, `set_eq_62hz_0db`,
//! https://github.com/RizeCrime/linuxblaster_control).
use super::reads::build_audio_effect_read;
use super::{CommandFamily, G6CommandBuilder};

/// EQ on/off (f32 0.0 / 1.0).
pub const FEATURE_EQ_TOGGLE: u8 = 0x09;
/// EQ preamp in dB.
pub const FEATURE_EQ_PREAMP: u8 = 0x0a;
/// First of the ten band gains (31 Hz); the bands follow in order up to 0x14.
pub const FEATURE_EQ_FIRST_BAND: u8 = 0x0b;
/// Last band (16 kHz).
pub const FEATURE_EQ_LAST_BAND: u8 = 0x14;

/// The feature id of EQ band `band` (0 = 31 Hz ... 9 = 16 kHz).
pub fn band_feature(band: usize) -> Option<u8> {
    u8::try_from(band)
        .ok()
        .map(|offset| FEATURE_EQ_FIRST_BAND + offset)
        .filter(|feature| *feature <= FEATURE_EQ_LAST_BAND)
}

/// The band index of an EQ band feature id.
pub fn band_index(feature: u8) -> Option<usize> {
    (FEATURE_EQ_FIRST_BAND..=FEATURE_EQ_LAST_BAND)
        .contains(&feature)
        .then(|| usize::from(feature - FEATURE_EQ_FIRST_BAND))
}

/// Reads of the toggle, the preamp and every band.
pub fn build_eq_read_commands() -> Vec<Vec<u8>> {
    (FEATURE_EQ_TOGGLE..=FEATURE_EQ_LAST_BAND)
        .map(build_audio_effect_read)
        .collect()
}

/// Turn the EQ on or off: `5a 12 07 01 96 09 [f32]`, then a read-back.
pub fn build_set_eq_toggle(enabled: bool) -> Vec<Vec<u8>> {
    build_eq_write(FEATURE_EQ_TOGGLE, if enabled { 1.0 } else { 0.0 })
}

/// Set the preamp or a band (`feature`) to `gain_db`:
/// `5a 12 07 01 96 [feature] [f32 dB]`, then a read-back.
pub fn build_set_eq_gain(feature: u8, gain_db: f32) -> Vec<Vec<u8>> {
    build_eq_write(feature, gain_db)
}

/// Set all ten bands at once (a preset): one write per band, without
/// read-backs. The G6 confirms each write with a status report anyway.
pub fn build_set_eq_bands(gains_db: &[f32; 10]) -> Vec<Vec<u8>> {
    (FEATURE_EQ_FIRST_BAND..=FEATURE_EQ_LAST_BAND)
        .zip(gains_db)
        .map(|(feature, gain_db)| build_feature_write(feature, *gain_db))
        .collect()
}

fn build_eq_write(feature: u8, value: f32) -> Vec<Vec<u8>> {
    vec![
        build_feature_write(feature, value),
        build_audio_effect_read(feature),
    ]
}

fn build_feature_write(feature: u8, value: f32) -> Vec<u8> {
    G6CommandBuilder::new(CommandFamily::DataControl)
        .operation(&[0x07, 0x01, 0x96, feature])
        .float_value(value)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_features_cover_0x0b_to_0x14() {
        assert_eq!(band_feature(0), Some(0x0b));
        assert_eq!(band_feature(9), Some(0x14));
        assert_eq!(band_feature(10), None);
        assert_eq!(band_index(0x0c), Some(1));
        assert_eq!(band_index(FEATURE_EQ_PREAMP), None);
    }

    #[test]
    fn gain_write_matches_capture_layout() {
        // 1 kHz (0x10) to +2.5 dB: 5a 12 07 01 96 10 00 00 20 40
        let commands = build_set_eq_gain(0x10, 2.5);
        assert_eq!(
            &commands[0][..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x96, 0x10, 0x00, 0x00, 0x20, 0x40]
        );
        assert_eq!(&commands[1][..6], &[0x5a, 0x11, 0x03, 0x01, 0x96, 0x10]);
    }

    #[test]
    fn band_batch_writes_every_band_in_order() {
        let commands = build_set_eq_bands(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
        assert_eq!(commands.len(), 10);
        assert_eq!(commands[0][5], 0x0b);
        assert_eq!(commands[9][5], 0x14);
        assert_eq!(&commands[1][6..10], &1.0f32.to_le_bytes());
    }

    #[test]
    fn toggle_on_writes_one() {
        let commands = build_set_eq_toggle(true);
        assert_eq!(
            &commands[0][..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x96, 0x09, 0x00, 0x00, 0x80, 0x3f]
        );
    }
}
