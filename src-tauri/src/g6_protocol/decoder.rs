//! Decoder mode (Dolby/DTS dynamic range), DSP parameter `97:02`:
//! 2.0 Normal, 1.0 Full, 3.0 Night. Captured by soundblaster-x-g6-cli
//! (`payloads/raw/g6_decoder.txt`); confirmed by linuxblaster_control.
use super::{dsp_read, dsp_write_and_read, DspFamily};
use crate::g6_spec::DecoderMode;

const DECODER_MODE: u8 = 0x02;

fn mode_value(mode: DecoderMode) -> f32 {
    match mode {
        DecoderMode::Full => 1.0,
        DecoderMode::Normal => 2.0,
        DecoderMode::Night => 3.0,
    }
}

/// The mode a decoder DSP report (`97:id = value`) stands for.
pub fn mode_for_report(id: u8, value: f32) -> Option<DecoderMode> {
    if id != DECODER_MODE {
        return None;
    }
    [DecoderMode::Full, DecoderMode::Normal, DecoderMode::Night]
        .into_iter()
        .find(|mode| (mode_value(*mode) - value).abs() < 0.01)
}

pub fn build_decoder_mode_read() -> Vec<u8> {
    dsp_read(DspFamily::Decoder, DECODER_MODE)
}

pub fn build_set_decoder_mode(mode: DecoderMode) -> Vec<Vec<u8>> {
    dsp_write_and_read(DspFamily::Decoder, DECODER_MODE, mode_value(mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_matches_capture() {
        // 5a 12 07 01 97 02 00 00 40 40
        let commands = build_set_decoder_mode(DecoderMode::Night);
        assert_eq!(
            &commands[0][..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x97, 0x02, 0x00, 0x00, 0x40, 0x40]
        );
        assert_eq!(mode_for_report(0x02, 2.0), Some(DecoderMode::Normal));
        assert_eq!(mode_for_report(0x02, 7.0), None);
    }
}
