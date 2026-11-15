//! DAC filter commands (family 0x6c). Write `5a 6c 03 00 [filter] 00`, read
//! `5a 6c 01 01`, reply `5a 6c 03 01 [filter] 00`: linuxblaster_control
//! captures `set_filter_*`. The G6's filter list (`5a 6c 01 02`, capture
//! `sbc_startup-01`) answers `5a 6c 0e 02 05 85 00 01 00 02 00 03 00 04 00
//! 05 00`: five filters, including 0x03 (NOS), whose write is not captured.
use super::{packet, CommandFamily};
use crate::g6_spec::DigitalFilter;

/// The filter's byte in `0x6c` packets.
pub fn filter_byte(filter: DigitalFilter) -> u8 {
    match filter {
        DigitalFilter::FastRollOffMinimumPhase => 0x01,
        DigitalFilter::SlowRollOffMinimumPhase => 0x02,
        DigitalFilter::NonOversampling => 0x03,
        DigitalFilter::FastRollOffLinearPhase => 0x04,
        DigitalFilter::SlowRollOffLinearPhase => 0x05,
    }
}

/// The filter a `0x6c` packet byte stands for.
pub fn filter_from_byte(byte: u8) -> Option<DigitalFilter> {
    match byte {
        0x01 => Some(DigitalFilter::FastRollOffMinimumPhase),
        0x02 => Some(DigitalFilter::SlowRollOffMinimumPhase),
        0x03 => Some(DigitalFilter::NonOversampling),
        0x04 => Some(DigitalFilter::FastRollOffLinearPhase),
        0x05 => Some(DigitalFilter::SlowRollOffLinearPhase),
        _ => None,
    }
}

/// Read the current filter: `5a 6c 01 01`.
pub fn build_dac_filter_read() -> Vec<u8> {
    packet(CommandFamily::DacFilter, &[0x01, 0x01])
}

/// Select `filter`, then read it back.
pub fn build_set_dac_filter(filter: DigitalFilter) -> Vec<Vec<u8>> {
    vec![
        packet(
            CommandFamily::DacFilter,
            &[0x03, 0x00, filter_byte(filter), 0x00],
        ),
        build_dac_filter_read(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_matches_capture() {
        // set_filter_fast_linear: 5a 6c 03 00 04 00, then 5a 6c 01 01
        let commands = build_set_dac_filter(DigitalFilter::FastRollOffLinearPhase);
        assert_eq!(&commands[0][..6], &[0x5a, 0x6c, 0x03, 0x00, 0x04, 0x00]);
        assert_eq!(&commands[1][..4], &[0x5a, 0x6c, 0x01, 0x01]);
    }

    #[test]
    fn every_filter_round_trips() {
        for byte in 1..=5 {
            assert_eq!(filter_from_byte(byte).map(filter_byte), Some(byte));
        }
        assert_eq!(filter_from_byte(0x06), None);
    }
}
