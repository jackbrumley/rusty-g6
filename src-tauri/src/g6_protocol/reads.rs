//! State queries, and the set a full state read sends.
use super::{
    dac_filter, decoder, effects, equalizer, firmware, lighting, microphone, packet, CommandFamily,
};

/// SBX master and Scout Mode: `5a 26 03 08 ff ff`, answered with a bitmask
/// (`events.rs`).
pub fn build_global_mode_read() -> Vec<u8> {
    packet(CommandFamily::GlobalMode, &[0x03, 0x08, 0xff, 0xff])
}

/// Current output: `5a 2c 01 01`, answered with `5a 2c 05 01 [output]`.
pub fn build_output_read() -> Vec<u8> {
    packet(CommandFamily::Output, &[0x01, 0x01])
}

/// Mic boost: `5a 3c 02 01 00`, answered with `5a 3c 04 01 00 02 [dB]`.
pub fn build_microphone_boost_read() -> Vec<u8> {
    packet(CommandFamily::MicBoost, &[0x02, 0x01, 0x00])
}

/// Every query of a full state read.
pub fn build_read_all_state_commands() -> Vec<Vec<u8>> {
    let mut commands = vec![
        firmware::build_firmware_query(),
        build_output_read(),
        build_global_mode_read(),
        build_microphone_boost_read(),
    ];
    commands.extend(effects::build_effect_read_commands());
    commands.extend(equalizer::build_eq_read_commands());
    commands.extend(microphone::build_mic_read_commands());
    commands.push(dac_filter::build_dac_filter_read());
    commands.push(decoder::build_decoder_mode_read());
    commands.extend(lighting::build_lighting_reads());
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_read_starts_with_firmware_output_and_modes() {
        let commands = build_read_all_state_commands();
        assert_eq!(&commands[0][..4], &[0x5a, 0x07, 0x01, 0x02]);
        assert_eq!(&commands[1][..4], &[0x5a, 0x2c, 0x01, 0x01]);
        assert_eq!(&commands[2][..6], &[0x5a, 0x26, 0x03, 0x08, 0xff, 0xff]);
    }
}
