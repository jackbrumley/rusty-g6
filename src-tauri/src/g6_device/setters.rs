//! Device settings writes. Each setter checks its input and sends the
//! commands; the G6's answers update the cached settings (`send_commands`),
//! so the cache holds what the device took.
use super::G6DeviceManager;
use crate::g6_protocol::{dac_filter, effects, equalizer, writes};
use crate::g6_spec::*;
use anyhow::{Context, Result};
use log::{info, warn};
use std::sync::atomic::Ordering;

impl G6DeviceManager {
    /// Send a write and fail if the G6 does not answer it.
    fn send_confirmed(&self, commands: Vec<Vec<u8>>, change: &str) -> Result<()> {
        if self.send_commands(commands)? {
            info!("{change}: done");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "The G6 did not answer ({change}). Use Read Device State to check it"
            ))
        }
    }

    /// Switch between speakers and headphones.
    pub fn toggle_output(&self) -> Result<()> {
        // The target comes from the cached output, so it must be the device's
        // real output. Without this check a device that never answered left
        // the cache at its default (Headphones) and every click re-sent
        // "switch to Speakers": one relay click, then nothing.
        if !self.state_confirmed.load(Ordering::SeqCst) {
            info!("Output state not confirmed by the device; reading it before toggling");
            self.read_device_state()
                .context("Cannot toggle output: the current output is unknown")?;
        }

        let current = self.current_settings.lock().unwrap().output;
        let target = toggle_target(current);
        info!("Switching output: {:?} -> {:?}", current, target);
        if !self.send_commands(writes::build_set_output(target))? {
            self.verify_unanswered_output_switch(target)?;
        }
        Ok(())
    }

    /// An output switch got no answer: re-read the device (healing the
    /// handle if needed) to learn whether the switch happened.
    fn verify_unanswered_output_switch(&self, target: OutputDevice) -> Result<()> {
        warn!("Output switch to {target:?} was not answered; re-reading device state");
        let settings = self.read_device_state()?;
        if settings.output == target {
            info!("Device reports {target:?} after re-read; the switch took effect");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Output switch to {target:?} was not applied (device reports {:?})",
                settings.output
            ))
        }
    }

    pub fn set_effect_enabled(&self, effect: SbxEffect, enabled: bool) -> Result<()> {
        self.send_confirmed(
            effects::build_set_effect_enabled(effect, enabled),
            &format!("{effect:?} {}", on_off(enabled)),
        )
    }

    /// `level` is 0-100.
    pub fn set_effect_level(&self, effect: SbxEffect, level: u8) -> Result<()> {
        let level = validate_effect_level(level)?;
        self.send_confirmed(
            effects::build_set_effect_level(effect, level),
            &format!("{effect:?} level {level}"),
        )
    }

    /// The SBX master switch.
    pub fn set_sbx_enabled(&self, enabled: bool) -> Result<()> {
        self.send_confirmed(
            writes::build_set_sbx_mode(enabled),
            &format!("SBX {}", on_off(enabled)),
        )
    }

    pub fn set_scout_mode_enabled(&self, enabled: bool) -> Result<()> {
        self.send_confirmed(
            writes::build_set_scout_mode(enabled),
            &format!("Scout Mode {}", on_off(enabled)),
        )
    }

    /// `db_value` is one of `MICROPHONE_BOOST_STEPS_DB`.
    pub fn set_microphone_boost(&self, db_value: u8) -> Result<()> {
        if !MICROPHONE_BOOST_STEPS_DB.contains(&db_value) {
            return Err(anyhow::anyhow!(
                "Mic boost must be one of {MICROPHONE_BOOST_STEPS_DB:?} dB, got {db_value}"
            ));
        }
        self.send_confirmed(
            writes::build_set_microphone_boost(db_value),
            &format!("mic boost {db_value} dB"),
        )
    }

    pub fn set_eq_enabled(&self, enabled: bool) -> Result<()> {
        self.send_confirmed(
            equalizer::build_set_eq_toggle(enabled),
            &format!("EQ {}", on_off(enabled)),
        )
    }

    pub fn set_eq_preamp(&self, gain_db: f32) -> Result<()> {
        let gain_db = validate_eq_gain(gain_db, EQ_PREAMP_LIMIT_DB)?;
        self.send_confirmed(
            equalizer::build_set_eq_gain(equalizer::FEATURE_EQ_PREAMP, gain_db),
            &format!("EQ preamp {gain_db} dB"),
        )
    }

    /// `band` is 0 (31 Hz) to 9 (16 kHz).
    pub fn set_eq_band(&self, band: usize, gain_db: f32) -> Result<()> {
        let feature = equalizer::band_feature(band)
            .with_context(|| format!("EQ band {band} does not exist (bands are 0-9)"))?;
        let gain_db = validate_eq_gain(gain_db, EQ_BAND_LIMIT_DB)?;
        self.send_confirmed(
            equalizer::build_set_eq_gain(feature, gain_db),
            &format!("EQ band {band} {gain_db} dB"),
        )
    }

    /// Set all ten EQ bands at once (a preset). Every gain is checked before
    /// anything is sent.
    pub fn set_eq_bands(&self, gains_db: [f32; 10]) -> Result<()> {
        let mut snapped = [0.0; 10];
        for (band, gain_db) in gains_db.iter().enumerate() {
            snapped[band] = validate_eq_gain(*gain_db, EQ_BAND_LIMIT_DB)
                .with_context(|| format!("EQ band {band}"))?;
        }
        self.send_confirmed(
            equalizer::build_set_eq_bands(&snapped),
            &format!("EQ bands {snapped:?} dB"),
        )
    }

    pub fn set_dac_filter(&self, filter: DigitalFilter) -> Result<()> {
        self.send_confirmed(
            dac_filter::build_set_dac_filter(filter),
            &format!("DAC filter {filter:?}"),
        )
    }
}

fn on_off(enabled: bool) -> &'static str {
    if enabled {
        "on"
    } else {
        "off"
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
