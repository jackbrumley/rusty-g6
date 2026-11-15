//! Finding, opening and closing the G6's HID control interface.
use super::G6DeviceManager;
use crate::g6_spec::*;
use anyhow::{Context, Result};
use hidapi::HidApi;
use log::{error, info};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

impl G6DeviceManager {
    /// Check if the G6 USB interface required for communication is currently available.
    pub fn is_g6_interface_available(&self) -> Result<bool> {
        const G6_INTERFACE: i32 = 4;

        let api = HidApi::new().context("Failed to refresh HID API for availability check")?;
        for device_info in api.device_list() {
            if device_info.vendor_id() == USB_VENDOR_ID
                && device_info.product_id() == USB_PRODUCT_ID
                && device_info.interface_number() == G6_INTERFACE
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Connect to the G6 device
    pub fn connect(&self) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 500;
        const G6_INTERFACE: i32 = 4;

        if self.is_connected() {
            info!("Connect requested while already connected; reusing current handle");
            return Ok(());
        }

        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            if attempt > 1 {
                info!(
                    "Connection attempt {}/{} (waiting {}ms for device enumeration)...",
                    attempt, MAX_RETRIES, RETRY_DELAY_MS
                );
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }

            // Refresh HidApi to get current device paths (critical after suspend/resume)
            let new_api = match HidApi::new() {
                Ok(api) => api,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to refresh HID API: {}", e));
                    continue;
                }
            };
            *self.api.lock().unwrap() = new_api;

            let api = self.api.lock().unwrap();

            info!("Attempting to connect to SoundBlaster X G6...");
            info!(
                "Looking for device with VID: {:04x}, PID: {:04x}",
                USB_VENDOR_ID, USB_PRODUCT_ID
            );

            // Enumerate all devices to find the correct interface
            let mut device_found = false;
            let mut target_path: Option<std::ffi::CString> = None;

            for device_info in api.device_list() {
                info!(
                    "Found device: VID={:04x}, PID={:04x}, Interface={}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info.interface_number()
                );

                if device_info.vendor_id() == USB_VENDOR_ID
                    && device_info.product_id() == USB_PRODUCT_ID
                {
                    device_found = true;

                    if device_info.interface_number() == G6_INTERFACE {
                        info!("Found correct interface ({})", G6_INTERFACE);
                        target_path = Some(device_info.path().to_owned());
                        break;
                    }
                }
            }

            if !device_found {
                last_error = Some(anyhow::anyhow!(
                    "G6 device not found. Is it plugged in? VID={:04x}, PID={:04x}",
                    USB_VENDOR_ID,
                    USB_PRODUCT_ID
                ));
                continue;
            }

            let path = match target_path {
                Some(p) => p,
                None => {
                    last_error = Some(anyhow::anyhow!(
                        "G6 device found but required interface {} is not available. \
                         The device has multiple interfaces but we need the 4th one.",
                        G6_INTERFACE
                    ));
                    continue;
                }
            };

            // Try to open the device by path
            match api.open_path(&path) {
                Ok(device) => {
                    info!(
                        "Successfully connected to G6 device via interface {} (attempt {})",
                        G6_INTERFACE, attempt
                    );
                    let manufacturer = device
                        .get_manufacturer_string()
                        .unwrap_or(Some("Unknown".to_string()))
                        .unwrap_or("Unknown".to_string());
                    let product = device
                        .get_product_string()
                        .unwrap_or(Some("Unknown".to_string()))
                        .unwrap_or("Unknown".to_string());

                    info!("Device: {} {}", manufacturer, product);

                    *self.device.lock().unwrap() = Some(device);
                    self.state_confirmed.store(false, Ordering::SeqCst);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to open G6 device: {}", e));
                    error!("Attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                }
            }
        }

        // All retries exhausted
        let error = last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown connection error"));
        error!(
            "Failed to connect after {} attempts: {}",
            MAX_RETRIES, error
        );
        Err(error)
    }

    /// Disconnect from the G6 device
    pub fn disconnect(&self) {
        *self.device.lock().unwrap() = None;
        self.state_confirmed.store(false, Ordering::SeqCst);
        info!("Disconnected from G6 device");
    }

    /// List all connected HID devices (for debugging)
    pub fn list_devices(&self) -> Result<Vec<String>> {
        let api = self.api.lock().unwrap();
        let devices = api.device_list();

        let device_list: Vec<String> = devices
            .map(|dev| {
                format!(
                    "VID: {:04x}, PID: {:04x}, Interface: {}, Manufacturer: {}, Product: {}",
                    dev.vendor_id(),
                    dev.product_id(),
                    dev.interface_number(),
                    dev.manufacturer_string().unwrap_or("Unknown"),
                    dev.product_string().unwrap_or("Unknown")
                )
            })
            .collect();

        Ok(device_list)
    }
}
