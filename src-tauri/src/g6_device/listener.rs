//! The device event listener: one thread for the life of the app that reads
//! the reports the G6 pushes (a change made on the device, an output switch)
//! and applies them to the cached settings.

use super::transport::read_packet;
use super::G6DeviceManager;
use crate::g6_protocol::events::{parse_device_packet, DeviceEvent};
use log::{error, info};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

/// How long a read blocks while holding the device handle.
const LISTEN_POLL_MS: i32 = 100;

impl G6DeviceManager {
    /// Start the event listener thread. It runs for the life of the app and
    /// reads from whatever handle is current, so calling this again after a
    /// reconnect is a no-op. `on_event` runs after every applied report and
    /// after a disconnect.
    pub fn start_listener<F>(&self, on_event: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if self.listener_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let device = self.device.clone();
        let settings = self.current_settings.clone();
        let command_active = self.command_active.clone();
        let state_confirmed = self.state_confirmed.clone();

        thread::spawn(move || {
            info!("Device event listener started");
            loop {
                // A command batch reads its own answers.
                if command_active.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                let read = {
                    let guard = device.lock().unwrap();
                    guard
                        .as_ref()
                        .map(|handle| read_packet(handle, LISTEN_POLL_MS))
                };

                match read {
                    // Not connected.
                    None => thread::sleep(Duration::from_millis(500)),
                    Some(Ok(None)) => {}
                    Some(Ok(Some(packet))) => {
                        let events = parse_device_packet(&packet);
                        if events.is_empty() {
                            continue;
                        }
                        let mut cached = settings.lock().unwrap();
                        for event in &events {
                            event.apply_to(&mut cached);
                            if matches!(event, DeviceEvent::OutputChanged(_)) {
                                state_confirmed.store(true, Ordering::SeqCst);
                            }
                            info!("Device reported {:?}", event);
                        }
                        drop(cached);
                        on_event();
                    }
                    Some(Err(read_error)) => {
                        error!("Listener read failed (G6 disconnected?): {read_error:#}");
                        *device.lock().unwrap() = None;
                        state_confirmed.store(false, Ordering::SeqCst);
                        on_event();
                        thread::sleep(Duration::from_millis(500));
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
