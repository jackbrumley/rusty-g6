//! The device event listener: one thread for the life of the app that reads
//! change events from the G6 and applies them to the cached settings.

use super::G6DeviceManager;
use crate::g6_protocol_v2;
use crate::g6_protocol_v2::events::DeviceEvent;
use log::{error, info};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

impl G6DeviceManager {
    /// Start the event listener thread. It runs for the life of the app and
    /// reads from whatever handle is current, so calling this again after a
    /// reconnect is a no-op.
    pub fn start_listener<F>(&self, on_event: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if self.listener_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let device_arc = self.device.clone();
        let settings_arc = self.current_settings.clone();
        let active_arc = self.command_active.clone();
        let confirmed_arc = self.state_confirmed.clone();

        thread::spawn(move || {
            info!("Device Event Listener Started");
            loop {
                // If main thread is sending commands, back off
                if active_arc.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                let mut buffer = vec![0u8; 65];
                let read_result = {
                    // Try to acquire lock - if connection drops, lock might fail?
                    match device_arc.lock() {
                        Ok(guard) => {
                            if let Some(dev) = guard.as_ref() {
                                dev.read_timeout(&mut buffer, 100)
                            } else {
                                // Device disconnected
                                Ok(0)
                            }
                        }
                        Err(_) => Ok(0), // Poisoned lock
                    }
                };

                match read_result {
                    Ok(bytes) if bytes > 0 => {
                        let packet = if buffer[0] == 0x00 && bytes > 1 {
                            &buffer[1..bytes]
                        } else {
                            &buffer[0..bytes]
                        };

                        // Parse packet using the unified event parser
                        if packet.len() > 2 && packet[0] == 0x5a {
                            // Get high-precision timestamp
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap();
                            let millis = timestamp.as_millis();
                            let micros = timestamp.as_micros() % 1000;

                            // Log all incoming events with full hex dump (up to 20 bytes)
                            let hex_str: String = packet
                                .iter()
                                .take(20)
                                .map(|b| format!("{:02x}", b))
                                .collect();
                            let desc = g6_protocol_v2::describe_packet(packet);

                            // Enhanced logging with timestamp
                            info!(
                                "\x1b[33m[RX-EVENT @{}.{:03}µs] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                                millis, micros, hex_str, desc
                            );

                            // Parse events using the protocol V2 parser
                            let events =
                                crate::g6_protocol_v2::events::G6EventParser::parse(packet);

                            // Process each detected event
                            for event in &events {
                                if let Ok(mut settings) = settings_arc.lock() {
                                    event.apply_to(&mut settings);
                                    if matches!(event, DeviceEvent::OutputChanged(_)) {
                                        confirmed_arc.store(true, Ordering::SeqCst);
                                    }
                                    info!("Detected {:?}", event);
                                }
                            }

                            // Notify UI if any events were processed
                            if !events.is_empty() {
                                on_event();
                            }
                        }
                    }
                    Ok(_) => {} // No data
                    Err(e) => {
                        error!("Read error in listener (possibly disconnected): {}", e);
                        // Clear the device on error to trigger reconnection flow
                        *device_arc.lock().unwrap() = None;
                        confirmed_arc.store(false, Ordering::SeqCst);
                        // Notify UI of disconnect
                        on_event();
                        // Sleep a bit longer before next attempt
                        thread::sleep(Duration::from_millis(500));
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
