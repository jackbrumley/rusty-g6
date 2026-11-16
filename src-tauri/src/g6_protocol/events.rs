//! Reports from the G6: answers to state queries, and changes it pushes on
//! its own (a write's confirmation, an output switch, a button on the
//! device). `DeviceEvent::apply_to` is the one mapping from a report to
//! `G6Settings`, used by both the state read and the event listener.
use super::dac_filter::filter_from_byte;
use super::decoder;
use super::device_flags;
use super::effects::{level_from_device, parameter_for_id, EffectParameter};
use super::equalizer::{band_index, FEATURE_EQ_PREAMP, FEATURE_EQ_TOGGLE};
use super::firmware::parse_firmware_response;
use super::lighting::{parse_lighting_report, LightingReport};
use super::microphone::{self, noise_reduction_level_from_device, MicParameter};
use super::voice_morph::{self, VoiceMorphParameter};
use super::{DspFamily, SettingAddress, PREFIX};
use crate::g6_spec::{
    DecoderMode, DeviceFlag, DigitalFilter, G6Settings, MicSwitch, OutputDevice, RgbColor,
    SbxEffect, VoiceMorphValue, MICROPHONE_BOOST_STEPS_DB,
};

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceEvent {
    FirmwareReported(String),
    OutputChanged(OutputDevice),
    SbxModeChanged(bool),
    ScoutModeChanged(bool),
    EffectToggled(SbxEffect, bool),
    EffectLevelChanged(SbxEffect, u8),
    EqToggled(bool),
    EqPreampChanged(f32),
    EqBandChanged {
        band: usize,
        gain_db: f32,
    },
    DigitalFilterChanged(DigitalFilter),
    MicrophoneBoostChanged(u8),
    MicSwitchChanged(MicSwitch, bool),
    DecoderModeChanged(DecoderMode),
    DeviceFlagsReported(u32),
    LightingToggled(bool),
    LightingColorChanged(RgbColor),
    NoiseReductionLevelChanged(u8),
    MicEqBandChanged {
        band: usize,
        gain_db: f32,
    },
    VoiceMorphToggled(bool),
    VoiceMorphValueChanged {
        value: VoiceMorphValue,
        reported: f32,
    },
}

impl DeviceEvent {
    /// Apply this event to `settings`.
    pub fn apply_to(&self, settings: &mut G6Settings) {
        match *self {
            DeviceEvent::FirmwareReported(ref version) => {
                settings.firmware_version = Some(version.clone())
            }
            DeviceEvent::OutputChanged(output) => settings.output = output,
            DeviceEvent::SbxModeChanged(enabled) => settings.sbx_enabled = enabled,
            DeviceEvent::ScoutModeChanged(enabled) => settings.scout_mode_enabled = enabled,
            DeviceEvent::EffectToggled(effect, enabled) => {
                settings.effects.get_mut(effect).enabled = enabled
            }
            DeviceEvent::EffectLevelChanged(effect, level) => {
                settings.effects.get_mut(effect).level = level
            }
            DeviceEvent::EqToggled(enabled) => settings.equalizer.enabled = enabled,
            DeviceEvent::EqPreampChanged(gain_db) => settings.equalizer.preamp_db = gain_db,
            DeviceEvent::EqBandChanged { band, gain_db } => {
                settings.equalizer.band_gains_db[band] = gain_db
            }
            DeviceEvent::DigitalFilterChanged(filter) => settings.digital_filter = Some(filter),
            DeviceEvent::MicrophoneBoostChanged(db_value) => {
                settings.microphone_boost_db = Some(db_value)
            }
            DeviceEvent::MicSwitchChanged(switch, enabled) => {
                *settings.mic_processing.switch_mut(switch) = enabled
            }
            DeviceEvent::NoiseReductionLevelChanged(level) => {
                settings.mic_processing.noise_reduction_level = level
            }
            DeviceEvent::MicEqBandChanged { band, gain_db } => {
                settings.mic_processing.equalizer_gains_db[band] = gain_db
            }
            DeviceEvent::VoiceMorphToggled(enabled) => {
                settings.mic_processing.voice_morph.enabled = enabled
            }
            DeviceEvent::VoiceMorphValueChanged { value, reported } => {
                settings.mic_processing.voice_morph.values[value as usize] = reported
            }
            DeviceEvent::DecoderModeChanged(mode) => settings.decoder_mode = Some(mode),
            DeviceEvent::DeviceFlagsReported(flags) => {
                settings.direct_mode = device_flags::is_set(flags, DeviceFlag::DirectMode);
                settings.spdif_out_direct = device_flags::is_set(flags, DeviceFlag::SpdifOutDirect);
            }
            DeviceEvent::LightingToggled(enabled) => settings.lighting.enabled = enabled,
            DeviceEvent::LightingColorChanged(color) => settings.lighting.color = color,
        }
    }
}

/// The events a packet from the G6 reports. Other packets (acks, echoes of
/// requests) report none.
pub fn parse_device_packet(packet: &[u8]) -> Vec<DeviceEvent> {
    parse_addressed_packet(packet)
        .into_iter()
        .map(|(_, event)| event)
        .collect()
}

/// Like `parse_device_packet`, with the setting each event is about: a DSP
/// report's blocks each address their own parameter.
pub fn parse_addressed_packet(packet: &[u8]) -> Vec<(SettingAddress, DeviceEvent)> {
    if packet.len() < 3 || packet[0] != PREFIX {
        return Vec::new();
    }
    if packet[1] == 0x11 {
        return parse_dsp_report(packet);
    }
    let address = SettingAddress::Family(packet[1]);
    parse_family_packet(packet)
        .into_iter()
        .map(|event| (address, event))
        .collect()
}

/// The events of a report outside the DSP families.
fn parse_family_packet(packet: &[u8]) -> Vec<DeviceEvent> {
    match packet[1] {
        0x07 => parse_firmware_response(packet)
            .map(DeviceEvent::FirmwareReported)
            .into_iter()
            .collect(),
        0x26 => parse_global_mode_report(packet)
            .into_iter()
            .flatten()
            .collect(),
        0x2c => parse_output_report(packet).into_iter().collect(),
        0x39 => device_flags::parse_device_flags_report(packet)
            .map(DeviceEvent::DeviceFlagsReported)
            .into_iter()
            .collect(),
        0x3a => parse_lighting_report(packet)
            .map(|report| match report {
                LightingReport::Enabled(enabled) => DeviceEvent::LightingToggled(enabled),
                LightingReport::Color(color) => DeviceEvent::LightingColorChanged(color),
            })
            .into_iter()
            .collect(),
        0x3c => parse_microphone_boost_report(packet).into_iter().collect(),
        0x6c => parse_dac_filter_report(packet).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// `5a 2c 05 01 [02 speakers | 04 headphones]`.
fn parse_output_report(packet: &[u8]) -> Option<DeviceEvent> {
    if packet.get(2..4)? != [0x05, 0x01] {
        return None;
    }
    match packet.get(4)? {
        0x02 => Some(DeviceEvent::OutputChanged(OutputDevice::Speakers)),
        0x04 => Some(DeviceEvent::OutputChanged(OutputDevice::Headphones)),
        _ => None,
    }
}

/// `5a 26 0b 08 ff ff [bitmask]`: bit 0 SBX (any playback effect or the EQ
/// on), bit 1 Scout Mode. Bit 2 (voice clarity) is not modelled yet.
fn parse_global_mode_report(packet: &[u8]) -> Option<[DeviceEvent; 2]> {
    if packet.get(2..6)? != [0x0b, 0x08, 0xff, 0xff] {
        return None;
    }
    let bitmask = *packet.get(6)?;
    Some([
        DeviceEvent::SbxModeChanged(bitmask & 0x01 != 0),
        DeviceEvent::ScoutModeChanged(bitmask & 0x02 != 0),
    ])
}

/// `5a 3c 04 01 00 02 [dB]`.
fn parse_microphone_boost_report(packet: &[u8]) -> Option<DeviceEvent> {
    if packet.get(2..6)? != [0x04, 0x01, 0x00, 0x02] {
        return None;
    }
    let db_value = *packet.get(6)?;
    MICROPHONE_BOOST_STEPS_DB
        .contains(&db_value)
        .then_some(DeviceEvent::MicrophoneBoostChanged(db_value))
}

/// `5a 6c 03 01 [filter] 00`.
fn parse_dac_filter_report(packet: &[u8]) -> Option<DeviceEvent> {
    if packet.get(2..4)? != [0x03, 0x01] {
        return None;
    }
    filter_from_byte(*packet.get(4)?).map(DeviceEvent::DigitalFilterChanged)
}

/// DSP status reports: `5a 11 [len] [count] 00` followed by `count` blocks
/// of `[family] [id] [f32 LE]`, with len = 2 + count * 6. A single value has
/// count 1; on an output switch the G6 pushes the new output's profile, up to
/// eight blocks per packet. Each family has its own ids: playback (0x96) and
/// recording (0x95) and decoder (0x97) are mapped.
fn parse_dsp_report(packet: &[u8]) -> Vec<(SettingAddress, DeviceEvent)> {
    let (Some(&length), Some(&count)) = (packet.get(2), packet.get(3)) else {
        return Vec::new();
    };
    let count = usize::from(count);
    let blocks_end = 5 + count * 6;
    if usize::from(length) != 2 + count * 6 || packet.len() < blocks_end {
        return Vec::new();
    }
    let (blocks, _) = packet[5..blocks_end].as_chunks::<6>();
    blocks
        .iter()
        .filter_map(|block| {
            let value = f32::from_le_bytes([block[2], block[3], block[4], block[5]]);
            let event = match block[0] {
                family if family == DspFamily::Playback as u8 => {
                    playback_dsp_event(block[1], value)
                }
                family if family == DspFamily::Decoder as u8 => {
                    decoder::mode_for_report(block[1], value).map(DeviceEvent::DecoderModeChanged)
                }
                family if family == DspFamily::Recording as u8 => {
                    recording_dsp_event(block[1], value)
                }
                _ => None,
            }?;
            let address = SettingAddress::Dsp {
                family: block[0],
                id: block[1],
            };
            Some((address, event))
        })
        .collect()
}

/// One recording DSP value as an event.
fn recording_dsp_event(id: u8, value: f32) -> Option<DeviceEvent> {
    if let Some(parameter) = voice_morph::parameter_for_id(id) {
        return Some(match parameter {
            VoiceMorphParameter::Enable => DeviceEvent::VoiceMorphToggled(value > 0.0001),
            VoiceMorphParameter::Value(which) => DeviceEvent::VoiceMorphValueChanged {
                value: which,
                reported: value,
            },
        });
    }
    Some(match microphone::parameter_for_id(id)? {
        MicParameter::Switch(switch) => DeviceEvent::MicSwitchChanged(switch, value > 0.0001),
        MicParameter::NoiseReductionLevel => {
            DeviceEvent::NoiseReductionLevelChanged(noise_reduction_level_from_device(value))
        }
        MicParameter::EqBand(band) => DeviceEvent::MicEqBandChanged {
            band,
            gain_db: value,
        },
    })
}

/// One playback DSP value as an event.
fn playback_dsp_event(id: u8, value: f32) -> Option<DeviceEvent> {
    let switched_on = value > 0.0001;
    if let Some((effect, parameter)) = parameter_for_id(id) {
        return Some(match parameter {
            EffectParameter::Toggle => DeviceEvent::EffectToggled(effect, switched_on),
            EffectParameter::Level => {
                DeviceEvent::EffectLevelChanged(effect, level_from_device(value))
            }
        });
    }
    match id {
        FEATURE_EQ_TOGGLE => Some(DeviceEvent::EqToggled(switched_on)),
        FEATURE_EQ_PREAMP => Some(DeviceEvent::EqPreampChanged(value)),
        _ => band_index(id).map(|band| DeviceEvent::EqBandChanged {
            band,
            gain_db: value,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        text.split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16).unwrap())
            .collect()
    }

    #[test]
    fn single_report_maps_playback_feature() {
        let events = parse_device_packet(&hex("5a 11 08 01 00 96 07 00 00 80 3f"));
        assert_eq!(
            events,
            vec![DeviceEvent::EffectToggled(SbxEffect::Crystalizer, true)]
        );
    }

    #[test]
    fn voice_morph_reports_map_to_settings() {
        // Answers from a G6 (firmware 2.1.250903.1324) to reads of 95:0a and 95:12.
        let mut settings = G6Settings::default();
        for report in [
            "5a 11 08 01 00 95 0a 00 00 80 3f",
            "5a 11 08 01 00 95 12 6f 67 87 3e",
        ] {
            for event in parse_device_packet(&hex(report)) {
                event.apply_to(&mut settings);
            }
        }
        let voice_morph = &settings.mic_processing.voice_morph;
        assert!(voice_morph.enabled);
        assert!((voice_morph.values[7] - 0.2645).abs() < 0.0001);
    }

    #[test]
    fn recording_family_is_not_read_as_playback() {
        // 95:04 is noise reduction, not Smart Volume (96:04).
        assert_eq!(
            parse_device_packet(&hex("5a 11 08 01 00 95 04 00 00 80 3f")),
            vec![DeviceEvent::MicSwitchChanged(
                MicSwitch::NoiseReduction,
                true
            )]
        );
    }

    #[test]
    fn read_requests_are_not_reports() {
        assert!(parse_device_packet(&hex("5a 11 03 01 96 07")).is_empty());
        assert!(parse_device_packet(&hex("5a 2c 01 01")).is_empty());
    }

    #[test]
    fn multi_feature_report_maps_every_block() {
        // EQ toggle on, 62 Hz band at -3.0 dB.
        let events =
            parse_device_packet(&hex("5a 11 0e 02 00 96 09 00 00 80 3f 96 0c 00 00 40 c0"));
        assert_eq!(
            events,
            vec![
                DeviceEvent::EqToggled(true),
                DeviceEvent::EqBandChanged {
                    band: 1,
                    gain_db: -3.0
                },
            ]
        );
    }

    #[test]
    fn other_families_are_parsed_exactly() {
        assert_eq!(
            parse_device_packet(&hex("5a 2c 05 01 02 00 00 00")),
            vec![DeviceEvent::OutputChanged(OutputDevice::Speakers)]
        );
        assert_eq!(
            parse_device_packet(&hex("5a 26 0b 08 ff ff 02 00")),
            vec![
                DeviceEvent::SbxModeChanged(false),
                DeviceEvent::ScoutModeChanged(true)
            ]
        );
        assert_eq!(
            parse_device_packet(&hex("5a 3c 04 01 00 02 14")),
            vec![DeviceEvent::MicrophoneBoostChanged(20)]
        );
        assert_eq!(
            parse_device_packet(&hex("5a 6c 03 01 03 00")),
            vec![DeviceEvent::DigitalFilterChanged(
                DigitalFilter::NonOversampling
            )]
        );
    }

    #[test]
    fn events_update_settings() {
        let mut settings = G6Settings::default();
        DeviceEvent::EffectLevelChanged(SbxEffect::Bass, 70).apply_to(&mut settings);
        DeviceEvent::EqBandChanged {
            band: 9,
            gain_db: 4.0,
        }
        .apply_to(&mut settings);
        assert_eq!(settings.effects.bass.level, 70);
        assert_eq!(settings.equalizer.band_gains_db[9], 4.0);
    }
}
