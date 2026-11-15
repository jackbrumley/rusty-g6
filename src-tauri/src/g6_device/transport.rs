//! Sending packets to the G6 and collecting its answers.
use super::{DeviceUnresponsive, G6DeviceManager};
use crate::g6_protocol_v2;
use anyhow::{Context, Result};
use log::{error, info, warn};
use std::sync::atomic::Ordering;
use std::time::Duration;

/// A state read stops after this many queries in a row get no packet at
/// all: a healthy G6 answers each query within a few milliseconds, so
/// waiting out the remaining queries (500 ms each) only delays recovery.
const MAX_SILENT_QUERIES: u32 = 2;
impl G6DeviceManager {
    /// Send commands to the G6 device (DATA + COMMIT pair). Returns whether
    /// the device acknowledged the batch with at least one response packet.
    /// An unacknowledged batch is logged and marks the cached state as
    /// unconfirmed, so the next toggle re-reads the device first.
    pub(super) fn send_commands(&self, commands: Vec<Vec<u8>>) -> Result<bool> {
        // Signal listener to pause
        self.command_active.store(true, Ordering::SeqCst);

        // Ensure strictly scoped lock
        let result = (|| -> Result<bool> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;
            let mut acknowledged = false;

            for command in commands.iter() {
                // Helper logging
                let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
                let desc = g6_protocol_v2::describe_packet(command);

                // Color Scheme: Green for TX
                info!("\x1b[32m[TX] {}\x1b[0m | \x1b[35m{}\x1b[0m", hex_str, desc);

                let mut data_with_report_id = vec![0x00];
                data_with_report_id.extend_from_slice(command);

                device
                    .write(&data_with_report_id)
                    .map_err(|e| anyhow::anyhow!("Failed to write to device: {}", e))?;

                // Wait for device to process
                std::thread::sleep(Duration::from_millis(20));

                // Try to read acknowledgment
                let mut buffer = vec![0u8; 65];
                if let Ok(bytes) = device.read_timeout(&mut buffer, 50) {
                    if bytes > 0 {
                        let response = if buffer[0] == 0x00 && bytes > 1 {
                            buffer[1..bytes].to_vec()
                        } else {
                            buffer[0..bytes].to_vec()
                        };

                        if response.len() >= 2 && response[0] == 0x5a {
                            acknowledged = true;
                            let rx_hex: String =
                                response.iter().map(|b| format!("{:02x}", b)).collect();
                            let rx_desc = g6_protocol_v2::describe_packet(&response);
                            info!(
                                "\x1b[33m[RX-ACK] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                                rx_hex, rx_desc
                            );
                        }
                    }
                }
            }
            Ok(acknowledged)
        })();

        // Resume listener
        self.command_active.store(false, Ordering::SeqCst);

        if let Ok(false) = result {
            warn!(
                "UNCONFIRMED WRITE: the G6 sent no acknowledgement for {} command(s); \
                 cached state is no longer trusted",
                commands.len()
            );
            self.state_confirmed.store(false, Ordering::SeqCst);
        }

        result
    }

    /// Send a single raw command and return the response (for Protocol V2 testing)
    pub fn send_raw_command(&self, command: &[u8]) -> Result<Vec<u8>> {
        self.command_active.store(true, Ordering::SeqCst);

        let result = (|| -> Result<Vec<u8>> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;

            // Log TX
            let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
            info!("\x1b[32m[TX-RAW] {}\x1b[0m", hex_str);

            // Prepend report ID
            let mut data_with_report_id = vec![0x00];
            data_with_report_id.extend_from_slice(command);

            // Send command
            device
                .write(&data_with_report_id)
                .context("Failed to write raw command to device")?;

            // Read response with timeout (100ms)
            let mut buffer = vec![0u8; 512];
            let bytes_read = device
                .read_timeout(&mut buffer, 100)
                .context("Failed to read raw response from device")?;

            // Remove report ID if present
            let response = if bytes_read > 0 && buffer[0] == 0x00 && bytes_read > 1 {
                buffer[1..bytes_read].to_vec()
            } else {
                buffer[0..bytes_read].to_vec()
            };

            // Log RX
            let rx_hex: String = response.iter().map(|b| format!("{:02x}", b)).collect();
            info!("\x1b[33m[RX-RAW] {}\x1b[0m", rx_hex);

            Ok(response)
        })();

        self.command_active.store(false, Ordering::SeqCst);
        result
    }

    /// Send read commands and collect responses with robust filtering
    pub(super) fn send_read_commands(&self, commands: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        self.command_active.store(true, Ordering::SeqCst);

        let result = (|| -> Result<Vec<Vec<u8>>> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;

            let mut responses = Vec::new();
            let mut consecutive_silent = 0;
            info!("Sending {} read commands to G6 device", commands.len());

            for (i, command) in commands.iter().enumerate() {
                if command.len() < 2 {
                    continue;
                }
                let cmd_type = command[1];

                // Log TX
                let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
                let desc = g6_protocol_v2::describe_packet(command);
                info!(
                    "\x1b[32m[TX-READ] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                    hex_str, desc
                );

                let mut data_with_report_id = vec![0x00];
                data_with_report_id.extend_from_slice(command);

                if let Err(e) = device.write(&data_with_report_id) {
                    error!("  ✗ Write failed for command {}: {}", i + 1, e);
                    responses.push(Vec::new());
                    continue;
                }

                let mut matched_response = None;
                let mut all_received = Vec::new();
                let mut packets_seen = 0;
                let start_time = std::time::Instant::now();

                while start_time.elapsed().as_millis() < 500 {
                    // 500ms total timeout per command
                    let mut buffer = vec![0u8; 65];
                    match device.read_timeout(&mut buffer, 100) {
                        // 100ms per read
                        Ok(bytes_read) if bytes_read > 0 => {
                            packets_seen += 1;
                            let response = if buffer[0] == 0x00 && bytes_read > 1 {
                                buffer[1..bytes_read].to_vec()
                            } else {
                                buffer[0..bytes_read].to_vec()
                            };

                            if response.len() < 2 || response[0] != 0x5a {
                                continue;
                            }

                            // Log ALL RX packets with detailed info
                            let rx_hex: String =
                                response.iter().map(|b| format!("{:02x}", b)).collect();
                            let rx_desc = g6_protocol_v2::describe_packet(&response);

                            // Check if this matches our expected response
                            let matches = response[1] == cmd_type;
                            let match_status = if matches { "✓ MATCH" } else { "✗ NO MATCH" };

                            info!(
                                "\x1b[33m[RX-READ] {}\x1b[0m | \x1b[35m{}\x1b[0m | Expected: 0x{:02x}, Got: 0x{:02x} {}",
                                rx_hex, rx_desc, cmd_type, response[1], match_status
                            );

                            all_received.push(response.clone());

                            if matches {
                                matched_response = Some(response);
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(resp) = matched_response {
                    info!(
                        "  ✓ Found matching response for command {} (0x{:02x})",
                        i + 1,
                        cmd_type
                    );
                    responses.push(resp);
                } else {
                    error!("  ✗ No matching response for command {} (0x{:02x}). Received {} packet(s) total.",
                        i + 1, cmd_type, all_received.len());
                    // No fallback used - return empty if not matched
                    responses.push(Vec::new());
                }

                if packets_seen == 0 {
                    consecutive_silent += 1;
                    if consecutive_silent >= MAX_SILENT_QUERIES {
                        return Err(DeviceUnresponsive(format!(
                            "{} state queries in a row got no packet at all \
                             (the HID handle is open and writes succeed, but nothing comes back)",
                            consecutive_silent
                        ))
                        .into());
                    }
                } else {
                    consecutive_silent = 0;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(responses)
        })();

        self.command_active.store(false, Ordering::SeqCst);
        result
    }
}
