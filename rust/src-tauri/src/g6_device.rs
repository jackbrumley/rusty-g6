/// SoundBlaster X G6 USB Device Communication
/// Handles USB HID communication with the G6 device

use crate::g6_spec::*;
use crate::g6_protocol;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use log::{debug, error, info};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::time::Duration;

/// G6 Device Manager
pub struct G6DeviceManager {
    api: Arc<Mutex<HidApi>>,
    device: Arc<Mutex<Option<HidDevice>>>,
    current_settings: Arc<Mutex<G6Settings>>,
    command_active: Arc<AtomicBool>,
}

impl G6DeviceManager {
    /// Create a new G6 Device Manager
    pub fn new() -> Result<Self> {
        let api = HidApi::new().context("Failed to initialize HID API")?;
        
        Ok(Self {
            api: Arc::new(Mutex::new(api)),
            device: Arc::new(Mutex::new(None)),
            current_settings: Arc::new(Mutex::new(G6Settings::default())),
            command_active: Arc::new(AtomicBool::new(false)),
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
        // Signal listener to pause
        self.command_active.store(true, Ordering::SeqCst);
        
        // Ensure strictly scoped lock
        let result = (|| -> Result<()> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref()
                .context("Device not connected")?;
            
            eprintln!("Sending {} commands to G6 device", commands.len());
            
            for (i, command) in commands.iter().enumerate() {
                eprintln!("Command {}/{}: {} bytes - {:02x?}", 
                        i + 1, commands.len(), command.len(), 
                        &command[0..std::cmp::min(8, command.len())]);
                
                let mut data_with_report_id = vec![0x00];
                data_with_report_id.extend_from_slice(&command);
                
                device.write(&data_with_report_id)
                    .map_err(|e| anyhow::anyhow!("Failed to write to device: {}", e))?;
            }
            Ok(())
        })();
        
        // Resume listener
        self.command_active.store(false, Ordering::SeqCst);
        
        result
    }

    /// Send read commands and collect responses with robust filtering
    fn send_read_commands(&self, commands: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        self.command_active.store(true, Ordering::SeqCst);
        
        let result = (|| -> Result<Vec<Vec<u8>>> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref()
                .context("Device not connected")?;
            
            let mut responses = Vec::new();
            info!("Sending {} read commands to G6 device", commands.len());
            
            for (i, command) in commands.iter().enumerate() {
                if command.len() < 2 { continue; }
                let cmd_type = command[1];
                
                let mut data_with_report_id = vec![0x00];
                data_with_report_id.extend_from_slice(&command);
                
                if let Err(e) = device.write(&data_with_report_id) {
                    error!("  ✗ Write failed for command {}: {}", i + 1, e);
                    responses.push(Vec::new());
                    continue;
                }
                
                let mut matched_response = None;
                let start_time = std::time::Instant::now();
                
                while start_time.elapsed().as_millis() < 500 { // 500ms total timeout per command
                    let mut buffer = vec![0u8; 65]; 
                    match device.read_timeout(&mut buffer, 100) { // 100ms per read
                        Ok(bytes_read) if bytes_read > 0 => {
                            let response = if buffer[0] == 0x00 && bytes_read > 1 {
                                buffer[1..bytes_read].to_vec()
                            } else {
                                buffer[0..bytes_read].to_vec()
                            };
                            
                            if response.len() < 2 || response[0] != 0x5a { continue; }
                            
                            if response[1] == cmd_type {
                                matched_response = Some(response);
                                break;
                            }
                            // Ignore stray events here as needed
                        }
                        _ => {}
                    }
                }
                
                if let Some(resp) = matched_response {
                    responses.push(resp);
                } else {
                    responses.push(Vec::new());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(responses)
        })();
        
        self.command_active.store(false, Ordering::SeqCst);
        result
    }
    
    /// Start the event listener thread
    pub fn start_listener<F>(&self, on_event: F) 
    where F: Fn() + Send + Sync + 'static 
    {
        let device_arc = self.device.clone();
        let settings_arc = self.current_settings.clone();
        let active_arc = self.command_active.clone();
        
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
                        Err(_) => Ok(0) // Poisoned lock
                    }
                };
                
                match read_result {
                    Ok(bytes) if bytes > 0 => {
                        let packet = if buffer[0] == 0x00 && bytes > 1 {
                            &buffer[1..bytes]
                        } else {
                            &buffer[0..bytes]
                        };
                        
                        // Parse packet and update settings if scout mode detected
                        if packet.len() > 2 && packet[0] == 0x5a {
                            // Check for Scout Mode event (0x26 family)
                            if packet[1] == 0x26 {
                                // Heuristic: Look for Feature 0x02 and Values 0x01 (On) or 0x00 (Off)
                                // Packet structures vary (set echo vs report), but Feature is usually at index 4 or 5
                                
                                let mut found_feature = false;
                                let mut new_state = None;
                                
                                // Search packet for Feature 0x02 followed closely by 0x01 or 0x00
                                for i in 2..packet.len()-2 {
                                    if packet[i] == 0x02 { // Feature ID found?
                                        // Check if it's really feature ID based on context?
                                        // Usually 5a 26 05 07 02 00 01 ...
                                        // Or 5a 26 0b ... 02 (Press event)
                                        
                                        // If 0x02 is followed by 0x00 then 0x01 (Enabled)
                                        if i+2 < packet.len() && packet[i+1] == 0x00 && packet[i+2] == 0x01 {
                                            new_state = Some(ScoutModeState::Enabled);
                                            found_feature = true;
                                            break;
                                        }
                                        // If 0x02 is followed by 0x00 then 0x00 (Disabled)
                                        if i+2 < packet.len() && packet[i+1] == 0x00 && packet[i+2] == 0x00 {
                                            new_state = Some(ScoutModeState::Disabled);
                                            found_feature = true;
                                            break;
                                        }
                                    }
                                }

                                if let Some(state) = new_state {
                                    if let Ok(mut settings) = settings_arc.lock() {
                                        settings.scout_mode = state;
                                        info!("Detected external Scout Mode change: {:?}", state);
                                    }
                                }
                            }

                            on_event();
                        }
                    }
                    _ => {}
                }
                
                thread::sleep(Duration::from_millis(10));
            }
        });
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

    /// Set SBX Master Mode
    pub fn set_sbx_mode(&self, enabled: EffectState) -> Result<()> {
        let commands = match enabled {
            EffectState::Enabled => g6_protocol::build_sbx_mode_enable(),
            EffectState::Disabled => g6_protocol::build_sbx_mode_disable(),
        };
        
        self.send_commands(commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.sbx_enabled = enabled;
        drop(settings); // Release lock
        
        // Save to disk
        self.save_settings_to_disk()?;
        
        info!("SBX Mode set to {:?}", enabled);
        Ok(())
    }

    /// Set Scout Mode
    pub fn set_scout_mode(&self, enabled: ScoutModeState) -> Result<()> {
        let commands = match enabled {
            ScoutModeState::Enabled => g6_protocol::build_scout_mode_enable(),
            ScoutModeState::Disabled => g6_protocol::build_scout_mode_disable(),
        };
        
        self.send_commands(commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.scout_mode = enabled;
        drop(settings); // Release lock
        
        // Save to disk
        self.save_settings_to_disk()?;
        
        info!("Scout Mode set to {:?}", enabled);
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
        // Disabled per user request (State should be ephemeral/read from device)
        debug!("Settings save requested but disk persistence is disabled");
        Ok(())
    }

    /// Load settings from disk (new format only)
    pub fn load_settings_from_disk(&self) -> Result<G6Settings> {
        // Disabled per user request
        info!("Disk settings loading disabled, using defaults");
        Ok(G6Settings::default())
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
        // Apply SBX mode first as it might be required for effects
        self.set_sbx_mode_internal(settings.sbx_enabled)?;
        // Apply Scout Mode
        self.set_scout_mode_internal(settings.scout_mode)?;
        
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

    /// Internal method to set SBX Mode without saving
    fn set_sbx_mode_internal(&self, enabled: EffectState) -> Result<()> {
        let commands = match enabled {
            EffectState::Enabled => g6_protocol::build_sbx_mode_enable(),
            EffectState::Disabled => g6_protocol::build_sbx_mode_disable(),
        };
        
        self.send_commands(commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.sbx_enabled = enabled;
        Ok(())
    }

    /// Internal method to set Scout Mode without saving
    fn set_scout_mode_internal(&self, enabled: ScoutModeState) -> Result<()> {
        let commands = match enabled {
            ScoutModeState::Enabled => g6_protocol::build_scout_mode_enable(),
            ScoutModeState::Disabled => g6_protocol::build_scout_mode_disable(),
        };
        
        self.send_commands(commands)?;
        
        let mut settings = self.current_settings.lock().unwrap();
        settings.scout_mode = enabled;
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
        let mut settings = g6_protocol::parse_device_state_responses(&responses)
            .map_err(|e| anyhow::anyhow!("Failed to parse device state: {}", e))?;
        
        // Preserve SBX Mode state since we can't read it yet
        // This prevents "Read State" from accidentally disabling SBX in the UI
        let current_sbx = self.current_settings.lock().unwrap().sbx_enabled;
        settings.sbx_enabled = current_sbx;

        // Preserve Scout Mode state
        let current_scout = self.current_settings.lock().unwrap().scout_mode;
        settings.scout_mode = current_scout;

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
                error!("Device synchronization failed: {}.", e);
                // Do NOT apply defaults on failure per user request.
                // Just leave internal state as default but don't write to device.
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
        // Apply SBX mode first
        self.set_sbx_mode_internal(settings.sbx_enabled)?;
        // Apply Scout Mode
        self.set_scout_mode_internal(settings.scout_mode)?;
        
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
