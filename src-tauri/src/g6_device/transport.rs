//! Sending packets to the G6 and collecting its answers. While a batch runs
//! the event listener is paused, so every answer is read here.
use super::{DeviceUnresponsive, G6DeviceManager};
use crate::g6_protocol::{
    command_address, describe_packet,
    events::{parse_addressed_packet, DeviceEvent},
    SettingAddress,
};
use crate::g6_spec::G6Settings;
use anyhow::{Context, Result};
use hidapi::HidDevice;
use log::{error, info, warn};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

/// A state read stops after this many queries in a row get no packet at
/// all: a healthy G6 answers each query within a few milliseconds, so
/// waiting out the remaining queries only delays recovery.
const MAX_SILENT_QUERIES: u32 = 2;
/// How long a query waits for its answer.
const QUERY_TIMEOUT: Duration = Duration::from_millis(500);
/// How long a write waits for further packets once the G6 goes quiet.
const WRITE_ANSWER_GAP_MS: i32 = 50;
/// How long a single read blocks.
const READ_POLL_MS: i32 = 100;

impl G6DeviceManager {
    /// Run `action` on the open handle with the listener paused.
    fn with_device<T>(&self, action: impl FnOnce(&HidDevice) -> Result<T>) -> Result<T> {
        self.command_active.store(true, Ordering::SeqCst);
        let result = {
            let device_guard = self.device.lock().unwrap();
            match device_guard.as_ref() {
                Some(device) => action(device),
                None => Err(anyhow::anyhow!("Device not connected")),
            }
        };
        self.command_active.store(false, Ordering::SeqCst);
        result
    }

    /// Send writes (each followed by a read-back) and apply every report the
    /// G6 answers with to the cached settings, so they hold what the device
    /// took, not what was asked for. Returns whether any packet came back.
    /// A batch with no answer at all is logged as `UNCONFIRMED WRITE` and
    /// marks the cached state as unconfirmed. A change the G6 reports to a
    /// setting the batch didn't address is logged as `SIDE EFFECT`.
    pub(super) fn send_commands(&self, commands: Vec<Vec<u8>>) -> Result<bool> {
        let answers = self.with_device(|device| {
            let mut answers = Vec::new();
            for command in &commands {
                write_packet(device, command, "TX")?;
                while let Some(answer) = read_packet(device, WRITE_ANSWER_GAP_MS)? {
                    answers.push(answer);
                }
            }
            Ok(answers)
        })?;

        let mut addressed: Vec<SettingAddress> = Vec::new();
        for address in commands
            .iter()
            .filter_map(|command| command_address(command))
        {
            if !addressed.contains(&address) {
                addressed.push(address);
            }
        }
        let mut settings = self.current_settings.lock().unwrap();
        let side_effects: Vec<_> = answers
            .iter()
            .flat_map(|answer| apply_answer(&mut settings, answer, &addressed))
            .collect();
        drop(settings);
        if !side_effects.is_empty() {
            let written: Vec<String> = addressed.iter().map(ToString::to_string).collect();
            for (address, event) in side_effects {
                warn!(
                    "SIDE EFFECT: writing {} made the G6 change {address}: {event:?}",
                    written.join(", ")
                );
            }
        }

        if answers.is_empty() {
            warn!(
                "UNCONFIRMED WRITE: the G6 sent no answer to {} command(s); \
                 cached state is no longer trusted",
                commands.len()
            );
            self.state_confirmed.store(false, Ordering::SeqCst);
        }
        Ok(!answers.is_empty())
    }

    /// Send one write after which the G6 re-enumerates on USB (it drops off
    /// the bus and returns about a second later). Its acknowledgement is the
    /// confirmation; a read failure after it means the device went away as
    /// expected, not that the write failed. The state read after the
    /// reconnect reports the new value. Returns whether it was acknowledged.
    pub(super) fn send_write_before_reenumeration(&self, command: Vec<u8>) -> Result<bool> {
        self.with_device(|device| {
            write_packet(device, &command, "TX")?;
            loop {
                match read_packet(device, WRITE_ANSWER_GAP_MS) {
                    Ok(Some(answer)) if answer[1] == 0x02 => return Ok(true),
                    Ok(Some(_)) => continue,
                    Ok(None) => return Ok(false),
                    Err(error) => {
                        info!("The G6 went away after the write, as expected: {error:#}");
                        return Ok(false);
                    }
                }
            }
        })
    }

    /// Send queries and return each one's answer: the first packet of the
    /// same family, or an empty packet if none came. Fails with
    /// `DeviceUnresponsive` once `MAX_SILENT_QUERIES` in a row got no packet.
    pub(super) fn send_read_commands(&self, commands: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        info!("Sending {} read commands to G6 device", commands.len());
        self.with_device(|device| {
            let mut answers = Vec::with_capacity(commands.len());
            let mut silent_in_a_row = 0;
            for command in &commands {
                write_packet(device, command, "TX-READ")?;
                let (answer, heard_anything) = await_answer(device, command[1])?;
                if answer.is_none() {
                    error!(
                        "No answer to {} ({})",
                        describe_packet(command),
                        hex(&command[..8])
                    );
                }
                answers.push(answer.unwrap_or_default());

                silent_in_a_row = if heard_anything {
                    0
                } else {
                    silent_in_a_row + 1
                };
                if silent_in_a_row >= MAX_SILENT_QUERIES {
                    return Err(DeviceUnresponsive(format!(
                        "{silent_in_a_row} state queries in a row got no packet at all \
                         (the HID handle is open and writes succeed, but nothing comes back)"
                    ))
                    .into());
                }
            }
            Ok(answers)
        })
    }
}

/// Apply a write's answer to `settings`. Returns the changes the G6 made
/// to settings outside `addressed` (its firmware sometimes switches one mic
/// feature when another is written).
fn apply_answer(
    settings: &mut G6Settings,
    answer: &[u8],
    addressed: &[SettingAddress],
) -> Vec<(SettingAddress, DeviceEvent)> {
    let mut side_effects = Vec::new();
    for (address, event) in parse_addressed_packet(answer) {
        let before = settings.clone();
        event.apply_to(settings);
        if !addressed.contains(&address) && *settings != before {
            side_effects.push((address, event));
        }
    }
    side_effects
}

/// Wait up to `QUERY_TIMEOUT` for a packet of `family`. Returns it, and
/// whether any packet arrived at all.
fn await_answer(device: &HidDevice, family: u8) -> Result<(Option<Vec<u8>>, bool)> {
    let started = Instant::now();
    let mut heard_anything = false;
    while started.elapsed() < QUERY_TIMEOUT {
        if let Some(packet) = read_packet(device, READ_POLL_MS)? {
            heard_anything = true;
            if packet[1] == family {
                return Ok((Some(packet), true));
            }
        }
    }
    Ok((None, heard_anything))
}

fn write_packet(device: &HidDevice, packet: &[u8], label: &str) -> Result<()> {
    info!("[{label}] {} | {}", hex(packet), describe_packet(packet));
    let mut report = Vec::with_capacity(packet.len() + 1);
    report.push(0x00); // HID report ID
    report.extend_from_slice(packet);
    device
        .write(&report)
        .map_err(|error| anyhow::anyhow!("Failed to write to the G6: {error}"))?;
    Ok(())
}

/// The next G6 packet within `timeout_ms`, without the report ID. Non-G6
/// data is skipped.
pub(super) fn read_packet(device: &HidDevice, timeout_ms: i32) -> Result<Option<Vec<u8>>> {
    let mut buffer = [0u8; 65];
    let length = device
        .read_timeout(&mut buffer, timeout_ms)
        .context("Failed to read from the G6")?;
    let data = match &buffer[..length] {
        [0x00, rest @ ..] if !rest.is_empty() => rest,
        other => other,
    };
    if data.len() < 2 || data[0] != 0x5a {
        return Ok(None);
    }
    info!("[RX] {} | {}", hex(data), describe_packet(data));
    Ok(Some(data.to_vec()))
}

fn hex(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .rposition(|&byte| byte != 0)
        .map_or(0, |last| last + 1);
    bytes[..end.max(2.min(bytes.len()))]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::g6_protocol::{dsp_write_and_read, DspFamily};

    fn hex(text: &str) -> Vec<u8> {
        text.split_whitespace()
            .map(|byte| u8::from_str_radix(byte, 16).unwrap())
            .collect()
    }

    fn addressed(commands: &[Vec<u8>]) -> Vec<SettingAddress> {
        commands
            .iter()
            .filter_map(|command| command_address(command))
            .collect()
    }

    // The G6's answer to "voice morph off" on 2026-11-16 at 18:43:28: it
    // also reported noise reduction (95:04) off.
    const VOICE_MORPH_OFF_ANSWER: &str = "5a 11 14 03 00 95 04 00 00 00 00 \
        95 0a 00 00 00 00 95 0a 00 00 00 00";

    #[test]
    fn a_change_to_another_setting_is_a_side_effect() {
        let mut settings = G6Settings::default();
        settings.mic_processing.noise_reduction = true;
        settings.mic_processing.voice_morph.enabled = true;
        let commands = dsp_write_and_read(DspFamily::Recording, 0x0a, 0.0);

        let side_effects = apply_answer(
            &mut settings,
            &hex(VOICE_MORPH_OFF_ANSWER),
            &addressed(&commands),
        );

        assert_eq!(side_effects.len(), 1);
        assert_eq!(side_effects[0].0.to_string(), "95:04");
        assert!(!settings.mic_processing.noise_reduction);
        assert!(!settings.mic_processing.voice_morph.enabled);
    }

    #[test]
    fn an_unchanged_setting_is_not_a_side_effect() {
        let mut settings = G6Settings::default();
        settings.mic_processing.voice_morph.enabled = true;
        let commands = dsp_write_and_read(DspFamily::Recording, 0x0a, 0.0);

        let side_effects = apply_answer(
            &mut settings,
            &hex(VOICE_MORPH_OFF_ANSWER),
            &addressed(&commands),
        );

        assert!(side_effects.is_empty());
    }
}
