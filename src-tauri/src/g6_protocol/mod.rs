//! The G6's HID protocol: 64-byte packets `5a [family] [payload...]`, zero
//! padded. This module owns the framing and the DSP parameter read/write
//! that most settings use; each area has its own module.
pub mod dac_filter;
pub mod decoder;
pub mod effects;
pub mod equalizer;
pub mod events;
pub mod firmware;
pub mod lighting;
pub mod microphone;
pub mod reads;
pub mod writes;

const PREFIX: u8 = 0x5a;
const PACKET_SIZE: usize = 64;

/// The DSP parameter families: settings are addressed as (family, id).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DspFamily {
    /// Mic processing: noise reduction, echo cancellation, mic EQ ...
    Recording = 0x95,
    /// Playback: SBX effects and the graphic EQ.
    Playback = 0x96,
    /// The Dolby/DTS decoder.
    Decoder = 0x97,
}

/// The packet families Rusty G6 sends. Others seen in captures (0x02 ack,
/// 0x05, 0x10, 0x15, 0x20, 0x30) are only named in `describe_packet`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandFamily {
    FirmwareQuery = 0x07,
    /// Read one DSP parameter: `5a 11 03 01 [dsp family] [id]`.
    DspRead = 0x11,
    /// Write one DSP parameter: `5a 12 07 01 [dsp family] [id] [f32 LE]`.
    DspWrite = 0x12,
    /// SBX master switch and Scout Mode.
    GlobalMode = 0x26,
    /// Speakers or headphones.
    Output = 0x2c,
    Lighting = 0x3a,
    MicBoost = 0x3c,
    DacFilter = 0x6c,
}

/// A packet of `family` with `payload`, zero padded to 64 bytes.
pub fn packet(family: CommandFamily, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(PACKET_SIZE);
    packet.push(PREFIX);
    packet.push(family as u8);
    packet.extend_from_slice(payload);
    packet.resize(PACKET_SIZE, 0x00);
    packet
}

/// Read DSP parameter `id`: `5a 11 03 01 [family] [id]`.
pub fn dsp_read(family: DspFamily, id: u8) -> Vec<u8> {
    packet(CommandFamily::DspRead, &[0x03, 0x01, family as u8, id])
}

/// Write DSP parameter `id`: `5a 12 07 01 [family] [id] [f32 LE]`.
pub fn dsp_write(family: DspFamily, id: u8, value: f32) -> Vec<u8> {
    let mut payload = vec![0x07, 0x01, family as u8, id];
    payload.extend_from_slice(&value.to_le_bytes());
    packet(CommandFamily::DspWrite, &payload)
}

/// Write DSP parameter `id`, then read it back.
pub fn dsp_write_and_read(family: DspFamily, id: u8, value: f32) -> Vec<Vec<u8>> {
    vec![dsp_write(family, id, value), dsp_read(family, id)]
}

/// A short name for a packet's family, for logging.
pub fn describe_packet(packet: &[u8]) -> String {
    let Some(&family) = packet.get(1) else {
        return "Invalid packet (too short)".to_string();
    };
    let name = match family {
        0x02 => "Acknowledgement",
        0x05 => "Identification",
        0x07 => "Firmware",
        0x10 => "Serial",
        0x11 => "DSP read / status",
        0x12 => "DSP write",
        0x15 => "Parameter ranges",
        0x20 => "Hardware ID",
        0x26 => "SBX / Scout mode",
        0x2c => "Output",
        0x30 => "DSP version",
        0x3a => "Lighting",
        0x3c => "Mic boost",
        0x6c => "DAC filter",
        _ => return format!("Unknown family 0x{family:02x}"),
    };
    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsp_write_matches_capture_layout() {
        // Crystalizer on: 5a 12 07 01 96 07 00 00 80 3f
        let write = dsp_write(DspFamily::Playback, 0x07, 1.0);
        assert_eq!(write.len(), 64);
        assert_eq!(
            &write[..10],
            &[0x5a, 0x12, 0x07, 0x01, 0x96, 0x07, 0x00, 0x00, 0x80, 0x3f]
        );
        assert_eq!(
            &dsp_read(DspFamily::Playback, 0x07)[..6],
            &[0x5a, 0x11, 0x03, 0x01, 0x96, 0x07]
        );
    }
}
