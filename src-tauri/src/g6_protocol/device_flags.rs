//! On/off device features in family 0x39.
//!
//! Writes are captured (soundblaster-x-g6-cli `payloads/raw/g6_playback.txt`):
//! `5a 39 03 00 [feature] [0|1]`. The G6 acknowledges the write and then
//! re-enumerates on USB (observed 2026-11-15 for Direct mode), so no read
//! follows it; the state read after the reconnect uses `5a 39 01 01`. The
//! G6 answers the read with `5a 39 05 01 [u32 LE]`, one bit per feature
//! (linuxblaster_control's `sbc_startup-01` capture shows the all-off
//! answer). The bit number equals the feature byte, as decoded from Creative's
//! CTHIDRpA.dll; a switch that snaps back on hardware would mean otherwise.
use super::{packet, CommandFamily};
use crate::g6_spec::DeviceFlag;

fn feature_byte(flag: DeviceFlag) -> u8 {
    match flag {
        DeviceFlag::DirectMode => 0x05,
        DeviceFlag::SpdifOutDirect => 0x0d,
    }
}

pub fn build_device_flags_read() -> Vec<u8> {
    packet(CommandFamily::DeviceFlags, &[0x01, 0x01])
}

/// The write alone: the G6 re-enumerates after it.
pub fn build_set_device_flag(flag: DeviceFlag, enabled: bool) -> Vec<u8> {
    packet(
        CommandFamily::DeviceFlags,
        &[0x03, 0x00, feature_byte(flag), u8::from(enabled)],
    )
}

/// The flag word in a `5a 39 05 01 [u32 LE]` answer.
pub fn parse_device_flags_report(packet: &[u8]) -> Option<u32> {
    if packet.get(2..4)? != [0x05, 0x01] {
        return None;
    }
    let word = packet.get(4..8)?;
    Some(u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
}

/// Whether `flag` is on in a flag word.
pub fn is_set(flags: u32, flag: DeviceFlag) -> bool {
    flags & (1 << feature_byte(flag)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_match_capture() {
        // Direct mode on: 5a 39 03 00 05 01
        let write = build_set_device_flag(DeviceFlag::DirectMode, true);
        assert_eq!(&write[..6], &[0x5a, 0x39, 0x03, 0x00, 0x05, 0x01]);
        // S/PDIF-out direct off: 5a 39 03 00 0d 00
        let write = build_set_device_flag(DeviceFlag::SpdifOutDirect, false);
        assert_eq!(&write[..6], &[0x5a, 0x39, 0x03, 0x00, 0x0d, 0x00]);
        assert_eq!(&build_device_flags_read()[..4], &[0x5a, 0x39, 0x01, 0x01]);
    }

    #[test]
    fn flag_word_is_read_per_bit() {
        let flags =
            parse_device_flags_report(&[0x5a, 0x39, 0x05, 0x01, 0x20, 0x20, 0x00, 0x00]).unwrap();
        assert!(is_set(flags, DeviceFlag::DirectMode));
        assert!(is_set(flags, DeviceFlag::SpdifOutDirect));
        assert_eq!(
            parse_device_flags_report(&[0x5a, 0x39, 0x05, 0x01, 0, 0, 0, 0]),
            Some(0)
        );
    }
}
