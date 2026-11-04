//! Reading the G6's state, and healing a handle that stopped answering
//! (`SELF-HEAL` in the session log, docs/TODO.md).

use super::{is_unresponsive, DeviceUnresponsive, G6DeviceManager};
use crate::g6_spec::*;
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl G6DeviceManager {
    /// Read current device state from hardware and update the cached
    /// settings. Fails unless at least one response was parsed.
    ///
    /// When the device does not answer, the handle is healed before giving
    /// up, escalating like a manual replug would:
    /// 1. close and reopen the HID handle, read again;
    /// 2. (Linux) reset the G6 on the USB bus, reconnect, read again.
    ///
    /// Every step is logged so the next occurrence is diagnosable.
    pub fn read_device_state(&self) -> Result<G6Settings> {
        if !self.is_connected() {
            return Err(anyhow::anyhow!("Device not connected"));
        }

        let error = match self.read_state_once() {
            Ok(settings) => return Ok(settings),
            Err(error) if is_unresponsive(&error) => error,
            Err(error) => return Err(error),
        };

        warn!(
            "SELF-HEAL: {}. Reopening the HID handle (software replug)...",
            error
        );
        self.disconnect();
        self.connect()
            .context("Self-heal: reopening the G6 HID handle failed")?;
        let error = match self.read_state_once() {
            Ok(settings) => {
                info!("SELF-HEAL: device answered after reopening the HID handle");
                return Ok(settings);
            }
            Err(error) if is_unresponsive(&error) => error,
            Err(error) => return Err(error),
        };

        self.reset_usb_and_read(error)
    }

    #[cfg(target_os = "linux")]
    fn reset_usb_and_read(&self, error: anyhow::Error) -> Result<G6Settings> {
        if self.usb_reset_attempted.swap(true, Ordering::SeqCst) {
            error!(
                "SELF-HEAL FAILED: {} after reopening the handle, and a USB reset was already tried. \
                 Unplug and replug the G6.",
                error
            );
            return Err(error.context(
                "The G6 is still not responding after a USB reset. Unplug and replug it",
            ));
        }

        warn!(
            "SELF-HEAL: {} after reopening the handle. Resetting the G6 on the USB bus...",
            error
        );
        self.disconnect();
        reset_usb_device().context("Self-heal: USB reset failed")?;
        info!("SELF-HEAL: USB reset done; reconnecting");
        self.connect()
            .context("Self-heal: reconnecting after the USB reset failed")?;
        match self.read_state_once() {
            Ok(settings) => {
                info!("SELF-HEAL: device answered after the USB reset");
                Ok(settings)
            }
            Err(error) => {
                error!(
                    "SELF-HEAL FAILED: {} even after a USB reset. Unplug and replug the G6.",
                    error
                );
                Err(error.context(
                    "The G6 is still not responding after a USB reset. Unplug and replug it",
                ))
            }
        }
    }

    /// Windows: the G6 is bound to the HID driver, so libusb cannot reset it.
    #[cfg(not(target_os = "linux"))]
    fn reset_usb_and_read(&self, error: anyhow::Error) -> Result<G6Settings> {
        error!(
            "SELF-HEAL FAILED: {} after reopening the handle. Unplug and replug the G6.",
            error
        );
        Err(error.context("The G6 is not responding. Unplug and replug it"))
    }

    /// One state read on the current handle, without recovery.
    fn read_state_once(&self) -> Result<G6Settings> {
        info!("Reading complete device state from G6...");

        // Build all read commands using V2 protocol
        let commands = crate::g6_protocol_v2::build_read_all_state_commands();

        // Send commands and get responses
        let responses = self.send_read_commands(commands)?;

        info!("Processing {} read responses...", responses.len());

        let mut settings = self.current_settings.lock().unwrap();
        let parsed = apply_read_responses(&mut settings, &responses);
        if parsed == 0 {
            return Err(DeviceUnresponsive(format!(
                "none of the {} state queries got a usable response",
                responses.len()
            ))
            .into());
        }

        settings.last_read_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        );
        self.state_confirmed.store(true, Ordering::SeqCst);
        self.usb_reset_attempted.store(false, Ordering::SeqCst);

        info!(
            "Device state read complete ({}/{} responses parsed)",
            parsed,
            responses.len()
        );

        // Return updated settings
        Ok(settings.clone())
    }

    /// Synchronize with device state on startup
    pub fn synchronize_with_device(&self) -> Result<()> {
        info!("Synchronizing with device state...");

        match self.read_device_state() {
            Ok(device_settings) => {
                info!("Device synchronization successful");

                // Log interesting findings
                if let Some(firmware) = &device_settings.firmware_info {
                    info!("Device firmware: {}", firmware.version);
                }

                if let Some(eq) = &device_settings.equalizer {
                    info!(
                        "EQ bands detected: {}, enabled: {:?}",
                        eq.bands.len(),
                        eq.enabled
                    );
                }

                if let Some(ext) = &device_settings.extended_params {
                    let non_null_params = [
                        ext.param_0x0a,
                        ext.param_0x0b,
                        ext.param_0x0c,
                        ext.param_0x0d,
                        ext.param_0x0e,
                        ext.param_0x0f,
                        ext.param_0x10,
                        ext.param_0x11,
                        ext.param_0x12,
                        ext.param_0x13,
                        ext.param_0x14,
                        ext.param_0x1a,
                        ext.param_0x1b,
                        ext.param_0x1c,
                        ext.param_0x1d,
                    ]
                    .iter()
                    .filter(|p| p.is_some())
                    .count();

                    info!("Extended parameters detected: {}/15", non_null_params);
                }

                Ok(())
            }
            Err(e) => {
                error!("Device synchronization failed: {}.", e);
                // Do NOT apply defaults on failure per user request.
                // Just leave internal state as default but don't write to device.
                Ok(())
            }
        }
    }
}

/// Applies state-read responses to `settings`. Returns how many responses
/// carried data that was parsed; empty (unanswered) responses count as none.
fn apply_read_responses(settings: &mut G6Settings, responses: &[Vec<u8>]) -> usize {
    use crate::g6_protocol_v2::{DeviceEvent, G6EventParser, G6ResponseParser, ParsedResponse};

    let mut parsed = 0;
    for response in responses {
        if response.len() < 2 {
            continue;
        }

        match response[1] {
            0x07 => {
                // Firmware query
                if let (Ok(ParsedResponse::FirmwareInfo(info)), _) =
                    G6ResponseParser::parse(response)
                {
                    info!("Firmware: {}", info.version);
                    settings.firmware_info = Some(info);
                    parsed += 1;
                }
            }
            0x2c => {
                // Output routing
                if let (Ok(ParsedResponse::OutputDevice(output)), _) =
                    G6ResponseParser::parse(response)
                {
                    info!("Output: {:?}", output);
                    settings.output = output;
                    parsed += 1;
                }
            }
            _ => {
                let mut applied = false;
                for event in G6EventParser::parse(response) {
                    match event {
                        DeviceEvent::SbxModeChanged(state) => {
                            settings.sbx_enabled = state;
                            info!("SBX Mode: {:?}", state);
                        }
                        DeviceEvent::ScoutModeChanged(state) => {
                            settings.scout_mode = state;
                            info!("Scout Mode: {:?}", state);
                        }
                        DeviceEvent::SurroundToggled(state) => {
                            settings.surround_enabled = state;
                            info!("Surround enabled: {:?}", state);
                        }
                        DeviceEvent::SurroundValueChanged(value) => {
                            settings.surround_value = value;
                            info!("Surround value: {}", value);
                        }
                        DeviceEvent::CrystalizerToggled(state) => {
                            settings.crystalizer_enabled = state;
                            info!("Crystalizer enabled: {:?}", state);
                        }
                        DeviceEvent::CrystalizerValueChanged(value) => {
                            settings.crystalizer_value = value;
                            info!("Crystalizer value: {}", value);
                        }
                        DeviceEvent::BassToggled(state) => {
                            settings.bass_enabled = state;
                            info!("Bass enabled: {:?}", state);
                        }
                        DeviceEvent::BassValueChanged(value) => {
                            settings.bass_value = value;
                            info!("Bass value: {}", value);
                        }
                        DeviceEvent::SmartVolumeToggled(state) => {
                            settings.smart_volume_enabled = state;
                            info!("Smart Volume enabled: {:?}", state);
                        }
                        DeviceEvent::SmartVolumeValueChanged(value) => {
                            settings.smart_volume_value = value;
                            info!("Smart Volume value: {}", value);
                        }
                        DeviceEvent::DialogPlusToggled(state) => {
                            settings.dialog_plus_enabled = state;
                            info!("Dialog Plus enabled: {:?}", state);
                        }
                        DeviceEvent::DialogPlusValueChanged(value) => {
                            settings.dialog_plus_value = value;
                            info!("Dialog Plus value: {}", value);
                        }
                        DeviceEvent::DigitalFilterChanged(filter) => {
                            settings.digital_filter = Some(filter);
                            info!("Digital filter: {:?}", filter);
                        }
                        DeviceEvent::AudioConfigChanged(config) => {
                            settings.audio_config = Some(config);
                            info!("Audio config: {:?}", config);
                        }
                        _ => continue, // Ignore other events
                    }
                    applied = true;
                }
                if applied {
                    parsed += 1;
                }
            }
        }
    }
    parsed
}

/// Resets the G6 on the USB bus (USBDEVFS_RESET through libusb): the same
/// re-enumeration a physical replug causes, without the power cycle. Needs
/// write access to the G6's usbfs node, which the udev rule grants.
#[cfg(target_os = "linux")]
fn reset_usb_device() -> Result<()> {
    let devices = rusb::devices().context("Failed to list USB devices")?;
    for device in devices.iter() {
        let Ok(descriptor) = device.device_descriptor() else {
            continue;
        };
        if descriptor.vendor_id() != USB_VENDOR_ID || descriptor.product_id() != USB_PRODUCT_ID {
            continue;
        }
        info!(
            "Resetting G6 at USB bus {:03} device {:03}",
            device.bus_number(),
            device.address()
        );
        let handle = device
            .open()
            .context("Failed to open the G6 USB device (check the udev rule)")?;
        return match handle.reset() {
            // NotFound: the device re-enumerated as part of the reset.
            Ok(()) | Err(rusb::Error::NotFound) => Ok(()),
            Err(error) => Err(anyhow::anyhow!("USB reset failed: {}", error)),
        };
    }
    Err(anyhow::anyhow!("G6 not found on the USB bus"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Responses captured from a healthy G6 (session log, 2026-11-03).
    fn healthy_responses() -> Vec<Vec<u8>> {
        vec![
            hex("5a0710322e312e3235303930332e313332340000"),
            hex("5a2c0501020000000000"),
            hex("5a260b08ffff05000000"),
            hex("5a1108010096000000803f00000000"),
        ]
    }

    #[test]
    fn unanswered_read_parses_nothing() {
        // After a cold boot every query came back empty; that must not count
        // as a successful read.
        let mut settings = G6Settings::default();
        let responses = vec![Vec::new(); 14];
        assert_eq!(apply_read_responses(&mut settings, &responses), 0);
        assert!(settings.firmware_info.is_none());
    }

    #[test]
    fn healthy_read_applies_firmware_output_and_modes() {
        let mut settings = G6Settings::default();
        let parsed = apply_read_responses(&mut settings, &healthy_responses());
        assert_eq!(parsed, 4);
        assert_eq!(settings.firmware_info.unwrap().version, "2.1.250903.1324");
        assert_eq!(settings.output, OutputDevice::Speakers);
        assert_eq!(settings.sbx_enabled, EffectState::Enabled);
        assert_eq!(settings.surround_enabled, EffectState::Enabled);
    }
}
