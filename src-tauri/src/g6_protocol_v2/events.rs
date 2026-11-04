//! Events the G6 reports on its interrupt IN endpoint (a setting changed on
//! the device or by another app), and their parser. State reads reuse the
//! parser for the responses that share the event format.

use super::PREFIX;
use crate::g6_spec::{EffectState, OutputDevice};

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

    // Digital filter
    DigitalFilterChanged(crate::g6_spec::DigitalFilter),

    // Audio config (0x3c - purpose unclear)
    AudioConfigChanged(crate::g6_spec::AudioConfig),

    // Microphone boost
    MicrophoneBoostChanged(u8), // 0, 10, 20, or 30 dB
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

    /// Parse audio effect events (0x11 family)
    /// Format: 5a 11 08 01 00 96 [FEATURE] 00 [FLOAT_VALUE]
    fn parse_audio_effect_events(packet: &[u8]) -> Vec<DeviceEvent> {
        let mut events = Vec::new();

        if packet.len() < 11 || packet[1] != 0x11 || packet[2] != 0x08 {
            return events;
        }

        // Feature ID is at index 6
        let feature = packet[6];

        // Float value at indices 7-10 (little-endian)
        let value_bytes = &packet[7..11];
        let float_value = f32::from_le_bytes([
            value_bytes[0],
            value_bytes[1],
            value_bytes[2],
            value_bytes[3],
        ]);

        let enabled = if float_value > 0.0001 {
            EffectState::Enabled
        } else {
            EffectState::Disabled
        };

        let percentage = (float_value * 100.0).round() as u8;

        // Map feature ID to event
        match feature {
            0x00 => events.push(DeviceEvent::SurroundToggled(enabled)),
            0x01 => events.push(DeviceEvent::SurroundValueChanged(percentage)),
            0x02 => events.push(DeviceEvent::DialogPlusToggled(enabled)),
            0x03 => events.push(DeviceEvent::DialogPlusValueChanged(percentage)),
            0x04 => events.push(DeviceEvent::SmartVolumeToggled(enabled)),
            0x05 => events.push(DeviceEvent::SmartVolumeValueChanged(percentage)),
            0x07 => events.push(DeviceEvent::CrystalizerToggled(enabled)),
            0x08 => events.push(DeviceEvent::CrystalizerValueChanged(percentage)),
            0x18 => events.push(DeviceEvent::BassToggled(enabled)),
            0x19 => events.push(DeviceEvent::BassValueChanged(percentage)),
            _ => {}
        }

        events
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
