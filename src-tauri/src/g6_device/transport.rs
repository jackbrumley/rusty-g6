//! Sending packets to the G6 and collecting its answers. While a batch runs
//! the event listener is paused, so every answer is read here.
use super::{DeviceUnresponsive, G6DeviceManager};
use crate::g6_protocol::{describe_packet, events::parse_device_packet};
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
    /// marks the cached state as unconfirmed.
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

        let mut settings = self.current_settings.lock().unwrap();
        for answer in &answers {
            for event in parse_device_packet(answer) {
                event.apply_to(&mut settings);
            }
        }
        drop(settings);

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
