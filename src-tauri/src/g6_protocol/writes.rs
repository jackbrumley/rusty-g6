//! Writes that are not DSP parameters: output, SBX master / Scout Mode and
//! mic boost. Each write is followed by a read, so the device reports the
//! value it took.
use super::reads::{build_global_mode_read, build_microphone_boost_read, build_output_read};
use super::{packet, CommandFamily};
use crate::g6_spec::OutputDevice;

/// Switch the output: `5a 2c 05 00 [02 speakers | 04 headphones]`.
pub fn build_set_output(output: OutputDevice) -> Vec<Vec<u8>> {
    let value = match output {
        OutputDevice::Speakers => 0x02,
        OutputDevice::Headphones => 0x04,
    };
    vec![
        packet(CommandFamily::Output, &[0x05, 0x00, value]),
        build_output_read(),
    ]
}

const GLOBAL_MODE_SBX: u8 = 0x01;
const GLOBAL_MODE_SCOUT: u8 = 0x02;

/// `5a 26 05 07 [mode] 00 [0|1] 00`.
fn build_set_global_mode(mode: u8, enabled: bool) -> Vec<Vec<u8>> {
    vec![
        packet(
            CommandFamily::GlobalMode,
            &[0x05, 0x07, mode, 0x00, u8::from(enabled), 0x00],
        ),
        build_global_mode_read(),
    ]
}

/// The SBX master switch.
pub fn build_set_sbx_mode(enabled: bool) -> Vec<Vec<u8>> {
    build_set_global_mode(GLOBAL_MODE_SBX, enabled)
}

pub fn build_set_scout_mode(enabled: bool) -> Vec<Vec<u8>> {
    build_set_global_mode(GLOBAL_MODE_SCOUT, enabled)
}

/// Mic boost in dB (0, 10, 20 or 30; the byte is the dB value):
/// `5a 3c 04 00 00 02 [dB]`.
pub fn build_set_microphone_boost(db_value: u8) -> Vec<Vec<u8>> {
    vec![
        packet(CommandFamily::MicBoost, &[0x04, 0x00, 0x00, 0x02, db_value]),
        build_microphone_boost_read(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_match_the_captures() {
        assert_eq!(
            &build_set_output(OutputDevice::Speakers)[0][..5],
            &[0x5a, 0x2c, 0x05, 0x00, 0x02]
        );
        assert_eq!(
            &build_set_scout_mode(true)[0][..8],
            &[0x5a, 0x26, 0x05, 0x07, 0x02, 0x00, 0x01, 0x00]
        );
        assert_eq!(
            &build_set_microphone_boost(20)[0][..7],
            &[0x5a, 0x3c, 0x04, 0x00, 0x00, 0x02, 0x14]
        );
    }
}
