//! Device settings writes: output, SBX effects, SBX/Scout mode, mic boost.
use super::G6DeviceManager;
use crate::g6_protocol_v2::{equalizer, writes};
use crate::g6_spec::*;
use anyhow::{Context, Result};
use log::{info, warn};
use std::sync::atomic::Ordering;

impl G6DeviceManager {
    /// Toggle output between speakers and headphones
    /// Now using Protocol V2 (clean 2-command version)
    pub fn toggle_output(&self) -> Result<()> {
        // The toggle target comes from the cached output, so it must be the
        // device's real output. Without this check a device that never
        // answered left the cache at its default (Headphones) and every click
        // re-sent "switch to Speakers": one relay click, then nothing.
        if !self.state_confirmed.load(Ordering::SeqCst) {
            info!("Output state not confirmed by the device; reading it before toggling");
            self.read_device_state()
                .context("Cannot toggle output: the current output is unknown")?;
        }

        let current = self.current_settings.lock().unwrap().output;
        let target = toggle_target(current);

        // Use V2 protocol - just 2 commands instead of 30!
        let commands = writes::build_toggle_output_simple(current);

        let acknowledged = self.send_commands(commands)?;
        info!("Output toggle command sent: {:?} → {:?}", current, target);
        if !acknowledged {
            self.verify_unacknowledged_output_write(target)?;
        }

        // State will be updated by listener when device confirms the change
        Ok(())
    }

    /// An output write got no acknowledgement: re-read the device (healing
    /// the handle if needed) to learn whether the switch happened.
    fn verify_unacknowledged_output_write(&self, target: OutputDevice) -> Result<()> {
        warn!(
            "Output switch to {:?} was not acknowledged; re-reading device state",
            target
        );
        let settings = self.read_device_state()?;
        if settings.output == target {
            info!(
                "Device reports {:?} after re-read; the switch took effect",
                target
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Output switch to {:?} was not applied (device reports {:?})",
                target,
                settings.output
            ))
        }
    }

    /// Set output device
    /// Now using Protocol V2 (clean 2-command version)
    pub fn set_output(&self, output: OutputDevice) -> Result<()> {
        // Use V2 protocol - just 2 commands!
        let commands = vec![
            writes::build_set_output(output),
            writes::build_commit_output(),
        ];

        let acknowledged = self.send_commands(commands)?;

        // State will be updated by listener when device confirms
        info!("Set output command sent using V2: {:?}", output);
        if !acknowledged {
            self.verify_unacknowledged_output_write(output)?;
        }
        Ok(())
    }

    /// Set surround sound
    /// Now using Protocol V2
    pub fn set_surround(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = writes::build_set_surround_toggle(enabled_bool);
        let value_commands = writes::build_set_surround_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.surround_enabled = enabled;
        settings.surround_value = value;

        info!(
            "Surround set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set crystalizer
    /// Now using Protocol V2
    pub fn set_crystalizer(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = writes::build_set_crystalizer_toggle(enabled_bool);
        let value_commands = writes::build_set_crystalizer_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.crystalizer_enabled = enabled;
        settings.crystalizer_value = value;

        info!(
            "Crystalizer set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set bass
    /// Now using Protocol V2
    pub fn set_bass(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // IMPORTANT: Official software sends ONLY the toggle command when enabling/disabling
        // The value command should be sent separately only when adjusting the slider
        // For now, we only send the toggle command to match official behavior
        let toggle_commands = writes::build_set_bass_toggle(enabled_bool);
        self.send_commands(toggle_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.bass_enabled = enabled;
        settings.bass_value = value;

        info!(
            "Bass set to {:?} using V2 (value {} stored locally)",
            enabled, value
        );
        Ok(())
    }

    /// Set smart volume
    /// Now using Protocol V2
    pub fn set_smart_volume(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = writes::build_set_smart_volume_toggle(enabled_bool);
        let value_commands = writes::build_set_smart_volume_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.smart_volume_enabled = enabled;
        settings.smart_volume_value = value;

        info!(
            "Smart Volume set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set dialog plus
    /// Now using Protocol V2
    pub fn set_dialog_plus(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = writes::build_set_dialog_plus_toggle(enabled_bool);
        let value_commands = writes::build_set_dialog_plus_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.dialog_plus_enabled = enabled;
        settings.dialog_plus_value = value;

        info!(
            "Dialog Plus set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set SBX Master Mode
    /// Now using Protocol V2
    pub fn set_sbx_mode(&self, enabled: EffectState) -> Result<()> {
        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - Gaming family (0x26)
        let commands = writes::build_set_sbx_mode(enabled_bool);

        self.send_commands(commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.sbx_enabled = enabled;

        info!("SBX Mode set to {:?} using V2", enabled);
        Ok(())
    }

    /// Set Scout Mode
    /// Now using Protocol V2
    pub fn set_scout_mode(&self, enabled: ScoutModeState) -> Result<()> {
        let enabled_bool = matches!(enabled, ScoutModeState::Enabled);

        // Use V2 protocol - Gaming family (0x26)
        let commands = writes::build_set_scout_mode(enabled_bool);

        self.send_commands(commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.scout_mode = enabled;

        info!("Scout Mode set to {:?} using V2", enabled);
        Ok(())
    }

    /// Set Microphone Boost
    /// Now using Protocol V2 - discovered from packet capture
    /// Valid values: 0, 10, 20, or 30 dB
    pub fn set_microphone_boost(&self, db_value: u8) -> Result<()> {
        // Validate: only 0, 10, 20, or 30 dB allowed
        if ![0, 10, 20, 30].contains(&db_value) {
            return Err(anyhow::anyhow!(
                "Invalid microphone boost value: {}dB. Valid values: 0, 10, 20, 30",
                db_value
            ));
        }

        // Use V2 protocol - AudioConfig family (0x3c)
        let commands = writes::build_set_microphone_boost(db_value);

        self.send_commands(commands)?;

        info!("Microphone Boost set to {}dB using V2", db_value);
        Ok(())
    }

    /// Turn the graphic EQ on or off.
    pub fn set_eq_enabled(&self, enabled: EffectState) -> Result<()> {
        let commands = equalizer::build_set_eq_toggle(enabled == EffectState::Enabled);
        self.send_commands(commands)?;

        self.current_settings.lock().unwrap().equalizer.enabled = enabled;
        info!("EQ set to {:?}", enabled);
        Ok(())
    }

    /// Set the EQ preamp. Returns the gain sent, snapped to the 0.5 dB step.
    pub fn set_eq_preamp(&self, gain_db: f32) -> Result<f32> {
        let gain_db = validate_eq_gain(gain_db, EQ_PREAMP_LIMIT_DB)?;
        let commands = equalizer::build_set_eq_gain(equalizer::FEATURE_EQ_PREAMP, gain_db);
        self.send_commands(commands)?;

        self.current_settings.lock().unwrap().equalizer.preamp_db = gain_db;
        info!("EQ preamp set to {} dB", gain_db);
        Ok(gain_db)
    }

    /// Set EQ band `band` (0 = 31 Hz ... 9 = 16 kHz). Returns the gain sent,
    /// snapped to the 0.5 dB step.
    pub fn set_eq_band(&self, band: usize, gain_db: f32) -> Result<f32> {
        let feature = equalizer::band_feature(band)
            .with_context(|| format!("EQ band {band} does not exist (bands are 0-9)"))?;
        let gain_db = validate_eq_gain(gain_db, EQ_BAND_LIMIT_DB)?;
        let commands = equalizer::build_set_eq_gain(feature, gain_db);
        self.send_commands(commands)?;

        self.current_settings
            .lock()
            .unwrap()
            .equalizer
            .band_gains_db[band] = gain_db;
        info!("EQ band {} set to {} dB", band, gain_db);
        Ok(gain_db)
    }

    /// Set all ten EQ bands at once (a preset). Every gain is checked before
    /// anything is sent. Returns the gains sent, snapped to the 0.5 dB step.
    pub fn set_eq_bands(&self, gains_db: [f32; 10]) -> Result<[f32; 10]> {
        let mut snapped = [0.0; 10];
        for (band, gain_db) in gains_db.iter().enumerate() {
            snapped[band] = validate_eq_gain(*gain_db, EQ_BAND_LIMIT_DB)
                .with_context(|| format!("EQ band {band}"))?;
        }
        self.send_commands(equalizer::build_set_eq_bands(&snapped))?;

        self.current_settings
            .lock()
            .unwrap()
            .equalizer
            .band_gains_db = snapped;
        info!("EQ bands set to {:?} dB", snapped);
        Ok(snapped)
    }
}

/// The output a toggle switches to.
fn toggle_target(current: OutputDevice) -> OutputDevice {
    match current {
        OutputDevice::Headphones => OutputDevice::Speakers,
        OutputDevice::Speakers => OutputDevice::Headphones,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_target_flips_output() {
        assert_eq!(
            toggle_target(OutputDevice::Speakers),
            OutputDevice::Headphones
        );
        assert_eq!(
            toggle_target(OutputDevice::Headphones),
            OutputDevice::Speakers
        );
    }
}
