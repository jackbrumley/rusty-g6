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
                
                // CRITICAL: Read device state immediately after connection!
                // This is when reads work perfectly (before any writes).
                // Like the official Creative software, we read once on connect
                // then maintain state internally after that.
                drop(api); // Release API lock before reading
                
                eprintln!("\n------------------------------------------------------------");
                eprintln!("AUTO-READING DEVICE STATE ON CONNECT...");
                eprintln!("------------------------------------------------------------");
                
                match self.read_device_state() {
                    Ok(state) => {
                        info!("Successfully read initial device state: {:02x?}", &state[0..16]);
                        
                        // Parse the device state and update current settings
                        if let Err(e) = self.parse_and_update_settings(&state) {
                            error!("Failed to parse device state: {}", e);
                            eprintln!("⚠ Warning: Could not parse device state");
                        } else {
                            eprintln!("✓ Device state parsed and settings updated");
                        }
                    }
                    Err(e) => {
                        error!("Failed to read initial device state: {}", e);
                        eprintln!("⚠ Warning: Could not read device state on connect");
                        // Don't fail connection, just log the error
                    }
                }
                
                eprintln!("------------------------------------------------------------\n");
                
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

    /// Send a single read command and get response
    fn send_read_command(&self, command: Vec<u8>) -> Result<Vec<u8>> {
        let device_guard = self.device.lock().unwrap();
        
        let device = device_guard.as_ref()
            .context("Device not connected")?;
        
        eprintln!("Sending read command: {} bytes - {:02x?}", 
                 command.len(), 
                 &command[0..std::cmp::min(8, command.len())]);
        
        // Prepend 0x00 as report_id
        let mut data_with_report_id = vec![0x00];
        data_with_report_id.extend_from_slice(&command);
        
        // Send the command using write (interrupt endpoint)
        match device.write(&data_with_report_id) {
            Ok(bytes_written) => {
                eprintln!("  ✓ Wrote {} bytes", bytes_written);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to write read command: {}", e));
            }
        }
        
        // Read the response using read_timeout (interrupt endpoint)
        let mut response = vec![0u8; 64];
        match device.read_timeout(&mut response, 1000) {
            Ok(bytes_read) => {
                eprintln!("  ✓ Read {} bytes response: {:02x?}", 
                         bytes_read,
                         &response[0..std::cmp::min(16, bytes_read)]);
                Ok(response)
            }
            Err(e) => {
                Err(anyhow::anyhow!("Failed to read device response: {}", e))
            }
        }
    }

    /// Drain the response buffer after write operations
    /// After ANY write, the device enters a buffering mode where it returns
    /// status messages followed by cached previous responses. We need to drain
    /// these buffered responses before we can get clean reads.
    fn drain_response_buffer(&self) -> Result<()> {
        eprintln!("Draining response buffer...");
        
        let dummy_cmd = g6_protocol::build_status_query();
        
        // Send multiple dummy reads to drain the buffer
        // Based on testing, we need ~10 reads to fully drain
        for i in 0..12 {
            match self.send_read_command(dummy_cmd.clone()) {
                Ok(response) => {
                    // Check if response properly echoes our command (byte 1 = 0x05)
                    if response.len() > 1 && response[0] == 0x5a && response[1] == 0x05 {
                        eprintln!("  Buffer drained after {} reads (got proper echo)", i + 1);
                        return Ok(());
                    }
                    eprintln!("  Drain read {}: {:02x?}", i + 1, &response[0..4]);
                }
                Err(e) => {
                    error!("Error draining buffer: {}", e);
                    // Continue anyway
                }
            }
            
            // Small delay between drain reads
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        
        eprintln!("  Buffer drain complete (sent 12 dummy reads)");
        Ok(())
    }

    /// Send commands to the G6 device (DATA + COMMIT pair)
    fn send_commands(&self, commands: Vec<Vec<u8>>) -> Result<()> {
        let device_guard = self.device.lock().unwrap();
        
        let device = device_guard.as_ref()
            .context("Device not connected")?;
        
        eprintln!("\n============================================================");
        eprintln!("WRITING {} COMMANDS TO DEVICE", commands.len());
        eprintln!("============================================================");
        
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
        
        eprintln!("✓ All commands sent successfully\n");
        
        // Release the device lock before draining buffer (drain needs to acquire it)
        drop(device_guard);
        
        // CRITICAL: Drain the response buffer after ANY write operation
        // The device enters a buffering mode after writes where it returns
        // status updates followed by cached responses. We must drain these
        // before any subsequent reads will work properly.
        eprintln!("------------------------------------------------------------");
        eprintln!("DRAINING RESPONSE BUFFER...");
        eprintln!("------------------------------------------------------------");
        self.drain_response_buffer()?;
        eprintln!("============================================================\n");
        
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

    /// Read device state using the reliable 0x05 status query
    /// This command consistently returns clean data without buffering issues
    pub fn read_device_state(&self) -> Result<Vec<u8>> {
        info!("Reading device state...");
        
        eprintln!("\n============================================================");
        eprintln!("READING DEVICE STATE (0x05)");
        eprintln!("============================================================");
        
        // Pre-drain to clear any pending buffered data
        eprintln!("Pre-draining buffer...");
        self.drain_response_buffer()?;
        
        // CRITICAL: Read TWICE and use the second response
        // First read might still get buffered data even after drain
        let cmd = g6_protocol::build_status_query();
        
        eprintln!("First read (might be buffered)...");
        let _first = self.send_read_command(cmd.clone())?;
        
        eprintln!("Second read (should be clean)...");
        let response = self.send_read_command(cmd)?;
        
        // Verify we got a proper echo
        if response.len() > 1 && response[0] == 0x5a && response[1] == 0x05 {
            eprintln!("✓ Got clean response: {:02x?}", &response[0..16]);
        } else {
            eprintln!("⚠ Warning: Response doesn't echo command: {:02x?}", &response[0..16]);
        }
        
        eprintln!("============================================================\n");
        
        info!("Received device state response: {:02x?}", &response[0..16]);
        Ok(response)
    }

    /// Read full device state using all discovered commands
    /// Returns a vector of (command_name, response) tuples
    pub fn read_full_device_state(&self) -> Result<Vec<(String, Vec<u8>)>> {
        info!("Reading full device state with all commands...");
        
        eprintln!("\n============================================================");
        eprintln!("READING FULL DEVICE STATE");
        eprintln!("============================================================");
        
        // CRITICAL: Drain buffer BEFORE reading to clear any pending data
        eprintln!("Pre-draining buffer before reads...");
        self.drain_response_buffer()?;
        
        let mut results = Vec::new();
        
        // Send each command and collect responses
        let commands = vec![
            ("0x05_status", g6_protocol::build_status_query()),
            ("0x10", g6_protocol::build_query_10()),
            ("0x20", g6_protocol::build_query_20()),
            ("0x30", g6_protocol::build_query_30()),
            ("0x15", g6_protocol::build_query_15()),
            ("0x3a_v1", g6_protocol::build_query_3a_variant1()),
            ("0x05_status_repeat", g6_protocol::build_status_query()),
            ("0x39", g6_protocol::build_query_39()),
            ("0x3a_v2", g6_protocol::build_query_3a_variant2()),
        ];
        
        for (name, cmd) in commands {
            match self.send_read_command(cmd) {
                Ok(response) => {
                    eprintln!("{}: {:02x?}", name, &response[0..16]);
                    results.push((name.to_string(), response));
                }
                Err(e) => {
                    error!("Failed to read {}: {}", name, e);
                    // Continue with other commands even if one fails
                }
            }
            
            // Small delay between commands
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        eprintln!("============================================================\n");
        info!("Read {} responses", results.len());
        Ok(results)
    }

    /// Parse device state response and update current settings
    /// Response format: [5a 05 04 1f ...]
    /// Byte 3 (0x1f = 0b00011111) contains effect enable flags
    fn parse_and_update_settings(&self, response: &[u8]) -> Result<()> {
        if response.len() < 4 {
            return Err(anyhow::anyhow!("Response too short"));
        }
        
        // Verify response header
        if response[0] != 0x5a || response[1] != 0x05 {
            return Err(anyhow::anyhow!("Invalid response header"));
        }
        
        // Byte 3 contains effect flags (bit field)
        let flags = response[3];
        
        eprintln!("Parsing device state flags: 0x{:02x} (0b{:08b})", flags, flags);
        
        let mut settings = self.current_settings.lock().unwrap();
        
        // Parse bit flags (testing hypothesis from analysis)
        // Bit 0: Surround
        // Bit 1: Crystalizer  
        // Bit 2: Bass
        // Bit 3: Smart Volume
        // Bit 4: Dialog Plus
        
        settings.surround_enabled = if flags & 0x01 != 0 { 
            eprintln!("  Bit 0 (Surround): ENABLED");
            EffectState::Enabled 
        } else { 
            eprintln!("  Bit 0 (Surround): DISABLED");
            EffectState::Disabled 
        };
        
        settings.crystalizer_enabled = if flags & 0x02 != 0 { 
            eprintln!("  Bit 1 (Crystalizer): ENABLED");
            EffectState::Enabled 
        } else { 
            eprintln!("  Bit 1 (Crystalizer): DISABLED");
            EffectState::Disabled 
        };
        
        settings.bass_enabled = if flags & 0x04 != 0 { 
            eprintln!("  Bit 2 (Bass): ENABLED");
            EffectState::Enabled 
        } else { 
            eprintln!("  Bit 2 (Bass): DISABLED");
            EffectState::Disabled 
        };
        
        settings.smart_volume_enabled = if flags & 0x08 != 0 { 
            eprintln!("  Bit 3 (Smart Volume): ENABLED");
            EffectState::Enabled 
        } else { 
            eprintln!("  Bit 3 (Smart Volume): DISABLED");
            EffectState::Disabled 
        };
        
        settings.dialog_plus_enabled = if flags & 0x10 != 0 { 
            eprintln!("  Bit 4 (Dialog Plus): ENABLED");
            EffectState::Enabled 
        } else { 
            eprintln!("  Bit 4 (Dialog Plus): DISABLED");
            EffectState::Disabled 
        };
        
        // Note: We don't get effect VALUES (0-100) from this command
        // Those would require other commands or we maintain them from last writes
        
        info!("Settings updated from device state");
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
