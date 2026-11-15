//! Events the G6 reports on its interrupt IN endpoint (a setting changed on
//! the device or by another app), and their parser. State reads reuse the
//! parser for the responses that share the event format.

use super::equalizer::{band_index, FEATURE_EQ_PREAMP, FEATURE_EQ_TOGGLE};
use super::PREFIX;
use crate::g6_spec::{EffectState, G6Settings, OutputDevice};

// ============================================================================
// DEVICE EVENT TYPES (for event listener)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceEvent {
    // Output switching
    OutputChanged(OutputDevice),

    // Gaming modes
    SbxModeChanged(EffectState),
    ScoutModeChanged(crate::g6_spec::ScoutModeState),

    // Audio effects - toggles
    SurroundToggled(EffectState),
    CrystalizerToggled(EffectState),
    BassToggled(EffectState),
    SmartVolumeToggled(EffectState),
    DialogPlusToggled(EffectState),

    // Audio effects - values
    SurroundValueChanged(u8),
    CrystalizerValueChanged(u8),
    BassValueChanged(u8),
    SmartVolumeValueChanged(u8),
    DialogPlusValueChanged(u8),

    // Graphic EQ
    EqToggled(EffectState),
    EqPreampChanged(f32),
    EqBandChanged { band: usize, gain_db: f32 },

    // Digital filter
    DigitalFilterChanged(crate::g6_spec::DigitalFilter),

    // Audio config (0x3c - purpose unclear)
    AudioConfigChanged(crate::g6_spec::AudioConfig),

    // Microphone boost
    MicrophoneBoostChanged(u8), // 0, 10, 20, or 30 dB
}

impl DeviceEvent {
    /// Apply this event to `settings`. The one place that maps device
    /// reports to settings, shared by state reads and the event listener.
    pub fn apply_to(&self, settings: &mut G6Settings) {
        match *self {
            DeviceEvent::OutputChanged(output) => settings.output = output,
            DeviceEvent::SbxModeChanged(state) => settings.sbx_enabled = state,
            DeviceEvent::ScoutModeChanged(state) => settings.scout_mode = state,
            DeviceEvent::SurroundToggled(state) => settings.surround_enabled = state,
            DeviceEvent::CrystalizerToggled(state) => settings.crystalizer_enabled = state,
            DeviceEvent::BassToggled(state) => settings.bass_enabled = state,
            DeviceEvent::SmartVolumeToggled(state) => settings.smart_volume_enabled = state,
            DeviceEvent::DialogPlusToggled(state) => settings.dialog_plus_enabled = state,
            DeviceEvent::SurroundValueChanged(value) => settings.surround_value = value,
            DeviceEvent::CrystalizerValueChanged(value) => settings.crystalizer_value = value,
            DeviceEvent::BassValueChanged(value) => settings.bass_value = value,
            DeviceEvent::SmartVolumeValueChanged(value) => settings.smart_volume_value = value,
            DeviceEvent::DialogPlusValueChanged(value) => settings.dialog_plus_value = value,
            DeviceEvent::EqToggled(state) => settings.equalizer.enabled = state,
            DeviceEvent::EqPreampChanged(gain_db) => settings.equalizer.preamp_db = gain_db,
            DeviceEvent::EqBandChanged { band, gain_db } => {
                settings.equalizer.band_gains_db[band] = gain_db
            }
            DeviceEvent::DigitalFilterChanged(filter) => settings.digital_filter = Some(filter),
            DeviceEvent::AudioConfigChanged(config) => settings.audio_config = Some(config),
            DeviceEvent::MicrophoneBoostChanged(db_value) => settings.microphone_boost = db_value,
        }
    }
}

// ============================================================================
// EVENT PARSER (for device listener)
// ============================================================================

pub struct G6EventParser;

impl G6EventParser {
    /// Parse incoming device events from a packet
    pub fn parse(packet: &[u8]) -> Vec<DeviceEvent> {
        let mut events = Vec::new();

        // Validate packet
        if packet.len() < 3 || packet[0] != PREFIX {
            return events;
        }

        // Parse based on command family
        if let Some(event) = Self::parse_output_event(packet) {
            events.push(event);
        }

        events.extend(Self::parse_gaming_mode_events(packet));
        events.extend(Self::parse_audio_effect_events(packet));

        if let Some(event) = Self::parse_digital_filter_event(packet) {
            events.push(event);
        }

        if let Some(event) = Self::parse_audio_config_event(packet) {
            events.push(event);
        }

        events
    }

    /// Check if packet is an output change event (0x2c family)
    fn parse_output_event(packet: &[u8]) -> Option<DeviceEvent> {
        if packet.len() < 5 || packet[1] != 0x2c {
            return None;
        }

        // Exact format: 5a 2c 05 01 [VALUE] 00...
        // Value at index 4 (0x02=Speakers, 0x04=Headphones)
        match packet[4] {
            0x04 => Some(DeviceEvent::OutputChanged(OutputDevice::Headphones)),
            0x02 => Some(DeviceEvent::OutputChanged(OutputDevice::Speakers)),
            _ => None,
        }
    }

    /// Parse gaming mode events (0x26 family) - SBX & Scout Mode
    fn parse_gaming_mode_events(packet: &[u8]) -> Vec<DeviceEvent> {
        let mut events = Vec::new();

        if packet.len() < 7 || packet[1] != 0x26 {
            return events;
        }

        // Check if this is a REPORT (0x0b) - different format than COMMAND (0x05)
        // Report format: 5a 26 0b 08 ff ff [VALUE] 00 00...
        // Value at index [6] encodes BOTH SBX and Scout Mode:
        //   0x00 = Both OFF
        //   0x01 = SBX ON, Scout OFF
        //   0x02 = SBX OFF, Scout ON
        //   0x03 = Both ON
        if packet[2] == 0x0b && packet.len() >= 7 {
            let mode_value = packet[6];

            // Extract SBX state (bit 0)
            let sbx_enabled = (mode_value & 0x01) != 0;
            events.push(DeviceEvent::SbxModeChanged(if sbx_enabled {
                EffectState::Enabled
            } else {
                EffectState::Disabled
            }));

            // Extract Scout state (bit 1)
            let scout_enabled = (mode_value & 0x02) != 0;
            events.push(DeviceEvent::ScoutModeChanged(if scout_enabled {
                crate::g6_spec::ScoutModeState::Enabled
            } else {
                crate::g6_spec::ScoutModeState::Disabled
            }));
        } else {
            // Command format: 5a 26 05 07 [FEATURE] 00 [VALUE] 00 00...
            // Search for feature IDs + values
            for i in 2..packet.len() - 2 {
                if packet[i + 1] == 0x00 {
                    // Check SBX Mode (Feature 0x01)
                    if packet[i] == 0x01 {
                        if packet[i + 2] == 0x01 {
                            events.push(DeviceEvent::SbxModeChanged(EffectState::Enabled));
                        } else if packet[i + 2] == 0x00 {
                            events.push(DeviceEvent::SbxModeChanged(EffectState::Disabled));
                        }
                    }

                    // Check Scout Mode (Feature 0x02)
                    if packet[i] == 0x02 {
                        if packet[i + 2] == 0x01 {
                            events.push(DeviceEvent::ScoutModeChanged(
                                crate::g6_spec::ScoutModeState::Enabled,
                            ));
                        } else if packet[i + 2] == 0x00 {
                            events.push(DeviceEvent::ScoutModeChanged(
                                crate::g6_spec::ScoutModeState::Disabled,
                            ));
                        }
                    }
                }
            }
        }

        events
    }

    /// Parse DSP status reports (0x11 family): `5a 11 [len] [count] 00`
    /// followed by `count` blocks of `[family] [feature] [f32 LE]`. A
    /// single-feature report has len 0x08; on an output switch the G6 pushes
    /// up to eight blocks per packet (len = 2 + count * 6). Only the playback
    /// DSP family (0x96) is mapped; recording (0x95) and decoder (0x97)
    /// reports use the same feature numbers for other settings.
    fn parse_audio_effect_events(packet: &[u8]) -> Vec<DeviceEvent> {
        if packet.len() < 5 || packet[1] != 0x11 {
            return Vec::new();
        }

        let count = usize::from(packet[3]);
        let blocks_end = 5 + count * 6;
        if usize::from(packet[2]) != 2 + count * 6 || packet.len() < blocks_end {
            return Vec::new();
        }

        let (blocks, _) = packet[5..blocks_end].as_chunks::<6>();
        blocks
            .iter()
            .filter_map(|block| {
                let value = f32::from_le_bytes([block[2], block[3], block[4], block[5]]);
                Self::playback_dsp_event(block[0], block[1], value)
            })
            .collect()
    }

    /// Map one playback DSP (family 0x96) value to an event.
    fn playback_dsp_event(family: u8, feature: u8, value: f32) -> Option<DeviceEvent> {
        if family != 0x96 {
            return None;
        }

        let enabled = if value > 0.0001 {
            EffectState::Enabled
        } else {
            EffectState::Disabled
        };
        let percentage = (value * 100.0).round() as u8;

        let event = match feature {
            0x00 => DeviceEvent::SurroundToggled(enabled),
            0x01 => DeviceEvent::SurroundValueChanged(percentage),
            0x02 => DeviceEvent::DialogPlusToggled(enabled),
            0x03 => DeviceEvent::DialogPlusValueChanged(percentage),
            0x04 => DeviceEvent::SmartVolumeToggled(enabled),
            0x05 => DeviceEvent::SmartVolumeValueChanged(percentage),
            0x07 => DeviceEvent::CrystalizerToggled(enabled),
            0x08 => DeviceEvent::CrystalizerValueChanged(percentage),
            FEATURE_EQ_TOGGLE => DeviceEvent::EqToggled(enabled),
            FEATURE_EQ_PREAMP => DeviceEvent::EqPreampChanged(value),
            0x18 => DeviceEvent::BassToggled(enabled),
            0x19 => DeviceEvent::BassValueChanged(percentage),
            other => DeviceEvent::EqBandChanged {
                band: band_index(other)?,
                gain_db: value,
            },
        };
        Some(event)
    }

    /// Parse digital filter event (0x6c family)
    /// Format: 5a 6c 03 01 [FILTER_VALUE] 00 00...
    fn parse_digital_filter_event(packet: &[u8]) -> Option<DeviceEvent> {
        if packet.len() < 5 || packet[1] != 0x6c || packet[2] != 0x03 {
            return None;
        }

        use crate::g6_spec::DigitalFilter;

        // Filter value at index 4
        match packet[4] {
            0x01 => Some(DeviceEvent::DigitalFilterChanged(
                DigitalFilter::FastRollOffMinimumPhase,
            )),
            0x02 => Some(DeviceEvent::DigitalFilterChanged(
                DigitalFilter::SlowRollOffMinimumPhase,
            )),
            0x04 => Some(DeviceEvent::DigitalFilterChanged(
                DigitalFilter::FastRollOffLinearPhase,
            )),
            0x05 => Some(DeviceEvent::DigitalFilterChanged(
                DigitalFilter::SlowRollOffLinearPhase,
            )),
            _ => None,
        }
    }

    /// Parse audio config event (0x3c family)
    /// Format: 5a 3c 04 01 00 02 [DB_VALUE] 00 00... (microphone boost)
    /// Microphone boost values: 0x00=0dB, 0x0a=10dB, 0x14=20dB, 0x1e=30dB
    fn parse_audio_config_event(packet: &[u8]) -> Option<DeviceEvent> {
        if packet.len() < 7 || packet[1] != 0x3c {
            return None;
        }

        // Check if this is microphone boost: 5a 3c 04 01 00 02 [DB_VALUE]
        if packet[2] == 0x04 && packet.len() >= 7 && packet[5] == 0x02 {
            let raw_value = packet[6];
            // Convert hex value to dB: 0x00=0, 0x0a=10, 0x14=20, 0x1e=30
            let db_value = match raw_value {
                0x00 => 0,
                0x0a => 10,
                0x14 => 20,
                0x1e => 30,
                _ => return None, // Unknown value, skip
            };
            return Some(DeviceEvent::MicrophoneBoostChanged(db_value));
        }

        // Fallback to generic audio config
        use crate::g6_spec::AudioConfig;
        let value = packet[5];
        Some(DeviceEvent::AudioConfigChanged(AudioConfig::Unknown(value)))
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
        let events = G6EventParser::parse(&hex("5a 11 08 01 00 96 07 00 00 80 3f"));
        assert_eq!(
            events,
            vec![DeviceEvent::CrystalizerToggled(EffectState::Enabled)]
        );
    }

    #[test]
    fn recording_family_is_not_read_as_playback() {
        // 95:04 is noise reduction, not Smart Volume (96:04).
        let events = G6EventParser::parse(&hex("5a 11 08 01 00 95 04 00 00 80 3f"));
        assert!(events.is_empty());
    }

    #[test]
    fn multi_feature_report_maps_every_block() {
        // Two blocks: EQ toggle on, 62 Hz band at -3.0 dB.
        let events =
            G6EventParser::parse(&hex("5a 11 0e 02 00 96 09 00 00 80 3f 96 0c 00 00 40 c0"));
        assert_eq!(
            events,
            vec![
                DeviceEvent::EqToggled(EffectState::Enabled),
                DeviceEvent::EqBandChanged {
                    band: 1,
                    gain_db: -3.0
                },
            ]
        );
    }

    #[test]
    fn eq_events_update_settings() {
        let mut settings = G6Settings::default();
        DeviceEvent::EqPreampChanged(-2.5).apply_to(&mut settings);
        DeviceEvent::EqBandChanged {
            band: 9,
            gain_db: 4.0,
        }
        .apply_to(&mut settings);
        assert_eq!(settings.equalizer.preamp_db, -2.5);
        assert_eq!(settings.equalizer.band_gains_db[9], 4.0);
    }
}
