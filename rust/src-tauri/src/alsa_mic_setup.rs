use anyhow::{Context, Result};
use std::process::Command;

/// Represents the microphone setup result
#[derive(Debug)]
pub struct MicSetupResult {
    pub success: bool,
    pub message: String,
}

/// Automatically configures the G6 microphone settings via ALSA
/// This automates the manual alsamixer setup process based on community solutions
pub fn setup_g6_microphone() -> Result<MicSetupResult> {
    // Find the G6 card number
    let card_number = find_g6_card_number()?;
    
    let mut steps_completed = Vec::new();
    let mut errors = Vec::new();
    
    // Step 1: Set PCM Capture Source to "External Mic"
    // This is the critical step - it's an enumeration control
    match set_pcm_capture_source(&card_number, "External Mic") {
        Ok(_) => steps_completed.push("✓ Set PCM Capture Source to 'External Mic'"),
        Err(e) => errors.push(format!("Failed to set PCM Capture Source: {}", e)),
    }
    
    // Step 2: Set Input Gain Control to step 3 (maximum boost)
    // The G6 uses step values (0-3), not percentages
    match set_input_gain(&card_number, 3) {
        Ok(_) => steps_completed.push("✓ Set Input Gain Control to level 3 (maximum)"),
        Err(e) => errors.push(format!("Failed to set Input Gain: {}", e)),
    }
    
    // Step 3: Enable capture on External Mic (if needed)
    match enable_external_mic_capture(&card_number) {
        Ok(_) => steps_completed.push("✓ Enabled External Mic capture"),
        Err(e) => {
            // This might not exist on all systems, so just log it
            eprintln!("Note: Could not enable External Mic capture switch: {}", e);
        }
    }
    
    // Step 4: Set External Mic capture volume to maximum
    match set_external_mic_volume(&card_number, 100) {
        Ok(_) => steps_completed.push("✓ Set External Mic capture volume to 100%"),
        Err(e) => {
            eprintln!("Note: Could not set External Mic volume: {}", e);
        }
    }
    
    let mut message = String::new();
    if !steps_completed.is_empty() {
        message.push_str("Microphone configured successfully:\n");
        message.push_str(&steps_completed.join("\n"));
    }
    
    if !errors.is_empty() {
        if !message.is_empty() {
            message.push_str("\n\n");
        }
        message.push_str("Warnings:\n");
        message.push_str(&errors.join("\n"));
    }
    
    if message.is_empty() {
        message = "No configuration changes were made".to_string();
    }
    
    Ok(MicSetupResult {
        success: !steps_completed.is_empty(),
        message,
    })
}

/// Find the G6 card number using aplay -l
fn find_g6_card_number() -> Result<String> {
    let output = Command::new("aplay")
        .arg("-l")
        .output()
        .context(
            "Failed to execute 'aplay'. ALSA utilities not found.\n\n\
             Please install ALSA utilities:\n\
             Ubuntu/Debian/PikaOS: sudo apt install alsa-utils\n\
             Fedora: sudo dnf install alsa-utils\n\
             Arch: sudo pacman -S alsa-utils"
        )?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("aplay command failed: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Look for "Sound BlasterX G6" in the output
    for line in stdout.lines() {
        if line.contains("Sound BlasterX G6") || line.contains("Sound Blaster X G6") {
            // Extract card number from format: "card 1: G6 [Sound BlasterX G6]"
            if let Some(card_part) = line.split_whitespace().nth(1) {
                let card_num = card_part.trim_end_matches(':');
                return Ok(card_num.to_string());
            }
        }
    }
    
    anyhow::bail!(
        "Sound Blaster G6 not found in 'aplay -l' output.\n\
         Please ensure the device is connected.\n\
         Output was:\n{}", 
        stdout
    )
}

/// Set PCM Capture Source to the specified value using amixer
/// This is an enumeration control and requires special handling
fn set_pcm_capture_source(card_number: &str, source: &str) -> Result<()> {
    let output = Command::new("amixer")
        .args(&["-c", card_number, "-q", "set", "PCM Capture Source", source])
        .output()
        .context("Failed to execute amixer command")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("amixer failed: {}", stderr);
    }
    
    Ok(())
}

/// Set Input Gain Control using amixer
/// The G6 uses discrete steps (0-3), not percentages
fn set_input_gain(card_number: &str, step: u8) -> Result<()> {
    if step > 3 {
        anyhow::bail!("Input Gain Control step must be 0-3, got {}", step);
    }
    
    let output = Command::new("amixer")
        .args(&["-c", card_number, "-q", "sset", "Input Gain Control", &step.to_string()])
        .output()
        .context("Failed to execute amixer command")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("amixer failed: {}", stderr);
    }
    
    Ok(())
}

/// Enable capture on External Mic using amixer
fn enable_external_mic_capture(card_number: &str) -> Result<()> {
    // Try to enable capture on External Mic
    let output = Command::new("amixer")
        .args(&["-c", card_number, "-q", "set", "External Mic", "cap"])
        .output()
        .context("Failed to execute amixer command")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("amixer failed: {}", stderr);
    }
    
    Ok(())
}

/// Set External Mic capture volume to a percentage
fn set_external_mic_volume(card_number: &str, percent: u8) -> Result<()> {
    if percent > 100 {
        anyhow::bail!("Volume percentage must be 0-100, got {}", percent);
    }
    
    let output = Command::new("amixer")
        .args(&["-c", card_number, "-q", "set", "External Mic", &format!("{}%", percent), "cap"])
        .output()
        .context("Failed to execute amixer command")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("amixer failed: {}", stderr);
    }
    
    Ok(())
}

/// Get current microphone status using amixer
pub fn get_mic_status() -> Result<String> {
    let card_number = find_g6_card_number()?;
    
    let output = Command::new("amixer")
        .args(&["-c", &card_number, "scontents"])
        .output()
        .context("Failed to execute amixer command")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("amixer failed: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut status = String::from("Microphone Status:\n");
    status.push_str(&format!("Card Number: {}\n\n", card_number));
    
    // Extract relevant controls
    let mut current_control = String::new();
    for line in stdout.lines() {
        if line.starts_with("Simple mixer control") {
            current_control = line.to_string();
        } else if current_control.contains("PCM Capture Source") 
               || current_control.contains("External Mic")
               || current_control.contains("Input Gain Control") {
            status.push_str(&format!("{}\n{}\n", current_control, line));
        }
    }
    
    if status.len() <= 50 {
        status.push_str("No relevant controls found\n");
    }
    
    Ok(status)
}
