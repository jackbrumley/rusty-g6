/// SoundBlaster X G6 USB Device Communication
/// Handles USB HID communication with the G6 device

use crate::g6_spec::*;
use crate::g6_protocol;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use log::{debug, error, info};
use std::sync::{Arc, Mutex};

/// G6 Device Manager
pub struct G6DeviceManager {
    api: Arc<Mutex<HidApi>>,
    device: Arc<Mutex<Option<HidDevice>>>,
    current_settings: Arc<Mutex<G6Settings>>,
}

impl G6DeviceManager {
    /// Create a new G6 Device Manager
    pub fn new() -> Result<Self> {
        let api = HidApi::new().context("Failed to initialize HID API")?;
        
        Ok(Self {
            api: Arc::new(Mutex::new(api)),
            device: Arc::new(Mutex::new(None)),
            current_settings: Arc::new(Mutex::new(G6Settings::default())),
        })
    }

    /// Check if a G6 device is connected
    pub fn is_connected(&self) -> bool {
        self.device.lock().unwrap().is_some()
    }

    /// Connect to the G6 device
    pub fn connect(&self) -> Result<()> {
        let api = self.api.lock().unwrap();
        
        info!("Attempting to connect to SoundBlaster X G6...");
        info!("Looking for device with VID: {:04x}, PID: {:04x}", USB_VENDOR_ID, USB_PRODUCT_ID);
        
        // The G6 has 4 interfaces (2 Audio and 2 HID), we need interface 4
        // This is critical - the other interfaces ignore commands!
        const G6_INTERFACE: i32 = 4;
        
        // Enumerate all devices to find the correct interface
        let mut device_found = false;
        let mut target_path: Option<std::ffi::CString> = None;
        
        for device_info in api.device_list() {
            info!("Found device: VID={:04x}, PID={:04x}, Interface={}", 
                  device_info.vendor_id(), 
                  device_info.product_id(),
                  device_info.interface_number());
            
            if device_info.vendor_id() == USB_VENDOR_ID && 
               device_info.product_id() == USB_PRODUCT_ID {
                device_found = true;
                
                if device_info.interface_number() == G6_INTERFACE {
                    info!("Found correct interface ({})", G6_INTERFACE);
                    target_path = Some(device_info.path().to_owned());
                    break;
                }
            }
        }
        
        if !device_found {
            error!("No G6 device found with VID={:04x}, PID={:04x}", USB_VENDOR_ID, USB_PRODUCT_ID);
            return Err(anyhow::anyhow!(
                "G6 device not found. Is it plugged in? VID={:04x}, PID={:04x}",
                USB_VENDOR_ID, USB_PRODUCT_ID
            ));
        }
        
        let path = target_path.ok_or_else(|| {
            error!("G6 device found but interface {} not available", G6_INTERFACE);
            anyhow::anyhow!(
                "G6 device found but required interface {} is not available. \
                 The device has multiple interfaces but we need the 4th one.",
                G6_INTERFACE
            )
        })?;
        
        // Open the device by path (not by VID/PID which defaults to interface 0)
        match api.open_path(&path) {
            Ok(device) => {
                info!("Successfully connected to G6 device via interface {}", G6_INTERFACE);
                let manufacturer = device.get_manufacturer_string()
                    .unwrap_or(Some("Unknown".to_string()))
                    .unwrap_or("Unknown".to_string());
                let product = device.get_product_string()
                    .unwrap_or(Some("Unknown".to_string()))
                    .unwrap_or("Unknown".to_string());
                
                info!("Device: {} {}", manufacturer, product);
                
                *self.device.lock().unwrap() = Some(device);
                Ok(())
            }
            Err(e) => {
                error!("Failed to open G6 device: {}", e);
                Err(anyhow::anyhow!("Failed to open G6 device: {}", e))
            }
        }
    }

    /// Disconnect from the G6 device
    pub fn disconnect(&self) {
        *self.device.lock().unwrap() = None;
        info!("Disconnected from G6 device");
    }

    /// Send commands to the G6 device (DATA + COMMIT pair)
    fn send_commands(&self, commands: Vec<Vec<u8>>) -> Result<()> {
        let device_guard = self.device.lock().unwrap();
        
        let device = device_guard.as_ref()
            .context("Device not connected")?;
        
        eprintln!("Sending {} commands to G6 device", commands.len());
        
        for (i, command) in commands.iter().enumerate() {
            eprintln!("Command {}/{}: {} bytes - {:02x?}", 
                     i + 1, commands.len(), command.len(), 
                     &command[0..std::cmp::min(8, command.len())]);
            
            // CRITICAL: Prepend 0x00 as report_id!
            // Without this, the first byte of our payload is used as the report_id
            // and gets cut off, breaking the command protocol.
            let mut data_with_report_id = vec![0x00];
            data_with_report_id.extend_from_slice(&command);
            
            eprintln!("  With report ID: {} bytes - {:02x?}", 
                     data_with_report_id.len(),
                     &data_with_report_id[0..std::cmp::min(9, data_with_report_id.len())]);
            
            // Write the command data to the device
            match device.write(&data_with_report_id) {
                Ok(bytes_written) => {
                    eprintln!("  ✓ Wrote {} bytes successfully", bytes_written);
                }
                Err(e) => {
                    eprintln!("  ✗ Write failed: {}", e);
                    return Err(anyhow::anyhow!("Failed to write to device: {}", e));
                }
            }
        }
        
        eprintln!("All commands sent successfully");
        Ok(())
    }

    /// Toggle output between speakers and headphones
    pub fn toggle_output(&self) -> Result<()> {
        let current = self.current_settings.lock().unwrap().output;
        let commands = g6_protocol::build_output_toggle(current);
        
        self.send_commands(commands)?;
        
        // Update current settings
        let new_output = match current {
            OutputDevice::Speakers => OutputDevice::Headphones,
            OutputDevice::Headphones => OutputDevice::Speakers,
        };
        self.current_settings.lock().unwrap().output = new_output;
        
        info!("Output toggled to {:?}", new_output);
        Ok(())
    }

    /// Set output device
    pub fn set_output(&self, output: OutputDevice) -> Result<()> {
        let commands = match output {
            OutputDevice::Headphones => g6_protocol::build_output_headphones(),
            OutputDevice::Speakers => g6_protocol::build_output_speakers(),
        };
        self.send_commands(commands)?;
        self.current_settings.lock().unwrap().output = output;
        info!("Output set to {:?}", output);
        Ok(())
    }

    /// Set surround sound
    pub fn set_surround(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_surround_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_surround_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.surround_enabled = enabled;
        settings.surround_value = value;
        info!("Surround set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set crystalizer
    pub fn set_crystalizer(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_crystalizer_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_crystalizer_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.crystalizer_enabled = enabled;
        settings.crystalizer_value = value;
        info!("Crystalizer set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set bass
    pub fn set_bass(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_bass_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_bass_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.bass_enabled = enabled;
        settings.bass_value = value;
        info!("Bass set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set smart volume
    pub fn set_smart_volume(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_smart_volume_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_smart_volume_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.smart_volume_enabled = enabled;
        settings.smart_volume_value = value;
        info!("Smart Volume set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set dialog plus
    pub fn set_dialog_plus(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_dialog_plus_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_dialog_plus_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.dialog_plus_enabled = enabled;
        settings.dialog_plus_value = value;
        info!("Dialog Plus set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Get current settings
    pub fn get_settings(&self) -> G6Settings {
        self.current_settings.lock().unwrap().clone()
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

impl Default for G6DeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create G6DeviceManager")
    }
}
