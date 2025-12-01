/// SoundBlaster X G6 USB Device Communication
/// Handles USB HID communication with the G6 device

use crate::g6_spec::*;
use crate::g6_protocol;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use log::{debug, error, info};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;

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

    /// Send read commands and collect responses
    fn send_read_commands(&self, commands: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        let device_guard = self.device.lock().unwrap();
        
        let device = device_guard.as_ref()
            .context("Device not connected")?;
        
        let mut responses = Vec::new();
        
        info!("Sending {} read commands to G6 device", commands.len());
        
        for (i, command) in commands.iter().enumerate() {
            debug!("Read command {}/{}: {} bytes - {:02x?}", 
                   i + 1, commands.len(), command.len(), 
                   &command[0..std::cmp::min(8, command.len())]);
            
            // CRITICAL: Prepend 0x00 as report_id!
            let mut data_with_report_id = vec![0x00];
            data_with_report_id.extend_from_slice(&command);
            
            // Write the read command
            match device.write(&data_with_report_id) {
                Ok(_) => {
                    // Try to read response
                    let mut buffer = vec![0u8; 65]; // 64 bytes + potential report ID
                    match device.read_timeout(&mut buffer, USB_TIMEOUT_MS as i32) {
                        Ok(bytes_read) => {
                            if bytes_read > 0 {
                                // Remove report ID if present and trim to actual data
                                let response = if buffer[0] == 0x00 && bytes_read > 1 {
                                    buffer[1..bytes_read].to_vec()
                                } else {
                                    buffer[0..bytes_read].to_vec()
                                };
                                
                                debug!("  ✓ Read {} bytes: {:02x?}", 
                                       response.len(), 
                                       &response[0..std::cmp::min(8, response.len())]);
                                responses.push(response);
                            } else {
                                debug!("  ! No data received for command {}", i + 1);
                                responses.push(Vec::new());
                            }
                        }
                        Err(e) => {
                            debug!("  ✗ Read failed for command {}: {}", i + 1, e);
                            responses.push(Vec::new());
                        }
                    }
                }
                Err(e) => {
                    error!("  ✗ Write failed for command {}: {}", i + 1, e);
                    responses.push(Vec::new());
                }
            }
            
            // Small delay between commands to avoid overwhelming the device
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        info!("Read {} responses from G6 device", responses.len());
        Ok(responses)
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
        
        // Save to disk
        self.save_settings_to_disk()?;
        
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
        
        // Save to disk
        self.save_settings_to_disk()?;
        
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
        drop(settings); // Release lock before saving
        
        // Save to disk
        self.save_settings_to_disk()?;
        
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
        drop(settings); // Release lock before saving
        
        // Save to disk
        self.save_settings_to_disk()?;
        
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
        drop(settings); // Release lock before saving
        
        // Save to disk
        self.save_settings_to_disk()?;
        
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
        drop(settings); // Release lock before saving
        
        // Save to disk
        self.save_settings_to_disk()?;
        
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
        drop(settings); // Release lock before saving
        
        // Save to disk
        self.save_settings_to_disk()?;
        
        info!("Dialog Plus set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Get current settings
    pub fn get_settings(&self) -> G6Settings {
        self.current_settings.lock().unwrap().clone()
    }

    /// Get the path to the config file
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        let app_config_dir = config_dir.join("rusty-g6");
        
        // Create the directory if it doesn't exist
        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)
                .context("Failed to create config directory")?;
        }
        
        Ok(app_config_dir.join("g6_settings.json"))
    }

    /// Save current settings to disk
    pub fn save_settings_to_disk(&self) -> Result<()> {
        let settings = self.current_settings.lock().unwrap().clone();
        let config_path = Self::get_config_path()?;
        
        let json = serde_json::to_string_pretty(&settings)
            .context("Failed to serialize settings")?;
        
        fs::write(&config_path, json)
            .context("Failed to write config file")?;
        
        info!("Settings saved to {:?}", config_path);
        Ok(())
    }

    /// Load settings from disk (new format only)
    pub fn load_settings_from_disk(&self) -> Result<G6Settings> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            info!("No config file found, using defaults");
            return Ok(G6Settings::default());
        }
        
        let json = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        // Try to parse with new format
        match serde_json::from_str::<G6Settings>(&json) {
            Ok(settings) => {
                info!("Settings loaded from {:?}", config_path);
                Ok(settings)
            }
            Err(e) => {
                info!("Config file incompatible with new format ({}), starting fresh...", e);
                
                // Backup the old file
                let backup_path = config_path.with_extension("json.old");
                if let Err(backup_err) = fs::copy(&config_path, &backup_path) {
                    error!("Failed to backup old config: {}", backup_err);
                } else {
                    info!("Old config backed up to {:?}", backup_path);
                }
                
                // Remove the old file and start fresh
                if let Err(remove_err) = fs::remove_file(&config_path) {
                    error!("Failed to remove old config file: {}", remove_err);
                }
                
                info!("Using default settings and will create new config format");
                Ok(G6Settings::default())
            }
        }
    }

    /// Apply all settings from config to the device
    pub fn apply_all_settings(&self) -> Result<()> {
        info!("Applying all settings to device...");
        
        // Load settings from disk
        let settings = self.load_settings_from_disk()?;
        
        // Update in-memory settings
        *self.current_settings.lock().unwrap() = settings.clone();
        
        // Apply all settings to device (without individual saves)
        self.set_output_internal(settings.output)?;
        self.set_surround_internal(settings.surround_enabled, settings.surround_value)?;
        self.set_crystalizer_internal(settings.crystalizer_enabled, settings.crystalizer_value)?;
        self.set_bass_internal(settings.bass_enabled, settings.bass_value)?;
        self.set_smart_volume_internal(settings.smart_volume_enabled, settings.smart_volume_value)?;
        self.set_dialog_plus_internal(settings.dialog_plus_enabled, settings.dialog_plus_value)?;
        
        // Save once at the end
        self.save_settings_to_disk()?;
        
        info!("All settings applied successfully");
        Ok(())
    }

    /// Internal method to set output without saving
    fn set_output_internal(&self, output: OutputDevice) -> Result<()> {
        let commands = match output {
            OutputDevice::Headphones => g6_protocol::build_output_headphones(),
            OutputDevice::Speakers => g6_protocol::build_output_speakers(),
        };
        self.send_commands(commands)?;
        self.current_settings.lock().unwrap().output = output;
        Ok(())
    }

    /// Internal method to set surround without saving
    fn set_surround_internal(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_surround_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_surround_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.surround_enabled = enabled;
        settings.surround_value = value;
        Ok(())
    }

    /// Internal method to set crystalizer without saving
    fn set_crystalizer_internal(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_crystalizer_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_crystalizer_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.crystalizer_enabled = enabled;
        settings.crystalizer_value = value;
        Ok(())
    }

    /// Internal method to set bass without saving
    fn set_bass_internal(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_bass_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_bass_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.bass_enabled = enabled;
        settings.bass_value = value;
        Ok(())
    }

    /// Internal method to set smart volume without saving
    fn set_smart_volume_internal(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_smart_volume_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_smart_volume_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.smart_volume_enabled = enabled;
        settings.smart_volume_value = value;
        Ok(())
    }

    /// Internal method to set dialog plus without saving
    fn set_dialog_plus_internal(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;
        
        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_dialog_plus_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_dialog_plus_slider(value);
        
        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.dialog_plus_enabled = enabled;
        settings.dialog_plus_value = value;
        Ok(())
    }

    /// Read current device state from hardware
    pub fn read_device_state(&self) -> Result<G6Settings> {
        if !self.is_connected() {
            return Err(anyhow::anyhow!("Device not connected"));
        }
        
        info!("Reading complete device state from G6...");
        
        // Build all read commands
        let commands = g6_protocol::build_read_all_state_commands();
        
        // Send commands and collect responses
        let responses = self.send_read_commands(commands)?;
        
        // Parse responses into device settings
        let settings = g6_protocol::parse_device_state_responses(&responses)
            .map_err(|e| anyhow::anyhow!("Failed to parse device state: {}", e))?;
        
        info!("Device state read successfully: {} effects, firmware: {:?}", 
              responses.len(),
              settings.firmware_info.as_ref().map(|f| &f.version));
        
        // Update our internal state
        *self.current_settings.lock().unwrap() = settings.clone();
        
        Ok(settings)
    }
    
    /// Synchronize with device state on startup
    pub fn synchronize_with_device(&self) -> Result<()> {
        info!("Synchronizing with device state...");
        
        match self.read_device_state() {
            Ok(device_settings) => {
                info!("Device synchronization successful");
                
                // Save the read state as our baseline
                self.save_settings_to_disk()?;
                
                // Log interesting findings
                if let Some(firmware) = &device_settings.firmware_info {
                    info!("Device firmware: {}", firmware.version);
                }
                
                if let Some(eq) = &device_settings.equalizer {
                    info!("EQ bands detected: {}, enabled: {:?}", 
                          eq.bands.len(), eq.enabled);
                }
                
                if let Some(ext) = &device_settings.extended_params {
                    let non_null_params = [
                        ext.param_0x0a, ext.param_0x0b, ext.param_0x0c, ext.param_0x0d,
                        ext.param_0x0e, ext.param_0x0f, ext.param_0x10, ext.param_0x11,
                        ext.param_0x12, ext.param_0x13, ext.param_0x14, ext.param_0x1a,
                        ext.param_0x1b, ext.param_0x1c, ext.param_0x1d,
                    ].iter().filter(|p| p.is_some()).count();
                    
                    info!("Extended parameters detected: {}/15", non_null_params);
                }
                
                Ok(())
            }
            Err(e) => {
                error!("Device synchronization failed: {}. Using disk settings.", e);
                
                // Fallback to disk settings and apply them
                let disk_settings = self.load_settings_from_disk()?;
                *self.current_settings.lock().unwrap() = disk_settings.clone();
                
                // Try to apply disk settings to device
                self.apply_all_settings()?;
                
                Ok(())
            }
        }
    }
    
    /// Apply settings from disk to device (enhanced version that syncs first if possible)
    pub fn apply_all_settings_enhanced(&self) -> Result<()> {
        info!("Applying settings with device synchronization...");
        
        // Try to read current device state first
        let use_device_state = match self.read_device_state() {
            Ok(device_settings) => {
                info!("Device state read successfully, merging with disk settings");
                
                // Load disk settings 
                let mut disk_settings = self.load_settings_from_disk()?;
                
                // Preserve read-only data from device
                disk_settings.firmware_info = device_settings.firmware_info;
                disk_settings.scout_mode = device_settings.scout_mode;
                disk_settings.equalizer = device_settings.equalizer;
                disk_settings.extended_params = device_settings.extended_params;
                disk_settings.is_connected = true;
                disk_settings.last_read_time = device_settings.last_read_time;
                
                // Update our state
                *self.current_settings.lock().unwrap() = disk_settings.clone();
                
                true
            }
            Err(e) => {
                info!("Could not read device state ({}), using disk settings only", e);
                
                // Load settings from disk as fallback
                let settings = self.load_settings_from_disk()?;
                *self.current_settings.lock().unwrap() = settings.clone();
                
                false
            }
        };
        
        // Get current settings to apply
        let settings = self.current_settings.lock().unwrap().clone();
        
        // Apply controllable settings to device
        self.set_output_internal(settings.output)?;
        self.set_surround_internal(settings.surround_enabled, settings.surround_value)?;
        self.set_crystalizer_internal(settings.crystalizer_enabled, settings.crystalizer_value)?;
        self.set_bass_internal(settings.bass_enabled, settings.bass_value)?;
        self.set_smart_volume_internal(settings.smart_volume_enabled, settings.smart_volume_value)?;
        self.set_dialog_plus_internal(settings.dialog_plus_enabled, settings.dialog_plus_value)?;
        
        // Save the final merged state
        self.save_settings_to_disk()?;
        
        if use_device_state {
            info!("Settings applied successfully with device synchronization");
        } else {
            info!("Settings applied successfully (disk settings only)");
        }
        
        Ok(())
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
