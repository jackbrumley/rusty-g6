//! The G6's logo lighting (family 0x3a). Captured by soundblaster-x-g6-cli
//! (`payloads/raw/g6_lighting.txt`) and verified on hardware by
//! linuxblaster_control:
//! - on/off: `5a 3a 02 06 [0|1]`
//! - static colour: `5a 3a 06 04 00 03 01 00 01`, then
//!   `5a 3a 09 0a 00 03 01 01 [A B G R]` (A is always ff)
//! - reads: `5a 3a 01 07` -> `5a 3a 02 07 [0|1]`, and
//!   `5a 3a 06 0b 01 00 01 01 01` -> `5a 3a 09 0b 01 00 01 01 [A B G R]`
//!
//! Effects other than a static colour are not captured.
use super::{packet, CommandFamily};
use crate::g6_spec::RgbColor;

pub fn build_lighting_reads() -> Vec<Vec<u8>> {
    vec![
        packet(CommandFamily::Lighting, &[0x01, 0x07]),
        packet(
            CommandFamily::Lighting,
            &[0x06, 0x0b, 0x01, 0x00, 0x01, 0x01, 0x01],
        ),
    ]
}

pub fn build_set_lighting_enabled(enabled: bool) -> Vec<Vec<u8>> {
    let mut commands = vec![packet(
        CommandFamily::Lighting,
        &[0x02, 0x06, u8::from(enabled)],
    )];
    commands.extend(build_lighting_reads().into_iter().take(1));
    commands
}

pub fn build_set_lighting_color(color: RgbColor) -> Vec<Vec<u8>> {
    let mut commands = vec![
        packet(
            CommandFamily::Lighting,
            &[0x06, 0x04, 0x00, 0x03, 0x01, 0x00, 0x01],
        ),
        packet(
            CommandFamily::Lighting,
            &[
                0x09,
                0x0a,
                0x00,
                0x03,
                0x01,
                0x01,
                0xff,
                color.blue,
                color.green,
                color.red,
            ],
        ),
    ];
    commands.extend(build_lighting_reads().into_iter().skip(1));
    commands
}

/// What a lighting answer reports: the on/off state or the colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingReport {
    Enabled(bool),
    Color(RgbColor),
}

pub fn parse_lighting_report(packet: &[u8]) -> Option<LightingReport> {
    match packet.get(2..4)? {
        [0x02, 0x07] => Some(LightingReport::Enabled(*packet.get(4)? != 0)),
        [0x09, 0x0b] if packet.get(4..8)? == [0x01, 0x00, 0x01, 0x01] => {
            let abgr = packet.get(8..12)?;
            Some(LightingReport::Color(RgbColor {
                red: abgr[3],
                green: abgr[2],
                blue: abgr[1],
            }))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_write_matches_capture() {
        // RGB 255:255:0 -> 5a 3a 09 0a 00 03 01 01 ff 00 ff ff
        let commands = build_set_lighting_color(RgbColor {
            red: 255,
            green: 255,
            blue: 0,
        });
        assert_eq!(
            &commands[0][..9],
            &[0x5a, 0x3a, 0x06, 0x04, 0x00, 0x03, 0x01, 0x00, 0x01]
        );
        assert_eq!(
            &commands[1][..12],
            &[0x5a, 0x3a, 0x09, 0x0a, 0x00, 0x03, 0x01, 0x01, 0xff, 0x00, 0xff, 0xff]
        );
    }

    #[test]
    fn reads_parse() {
        assert_eq!(
            parse_lighting_report(&[0x5a, 0x3a, 0x02, 0x07, 0x01]),
            Some(LightingReport::Enabled(true))
        );
        assert_eq!(
            parse_lighting_report(&[
                0x5a, 0x3a, 0x09, 0x0b, 0x01, 0x00, 0x01, 0x01, 0xff, 0x10, 0x20, 0x30
            ]),
            Some(LightingReport::Color(RgbColor {
                red: 0x30,
                green: 0x20,
                blue: 0x10
            }))
        );
    }
}
