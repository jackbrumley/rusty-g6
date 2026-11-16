//! Mic monitoring: the ALSA `External Mic` playback switch and volume, which
//! mix the mic into the G6's output so you hear yourself. Rusty G6 stores
//! the choice and applies it when the card appears, like the mic level.
use super::CARD;
use crate::g6_spec::MicMonitoring;
use alsa::mixer::{MilliBel, Mixer, Selem, SelemChannelId, SelemId};
use alsa::Round;

const MIC_PLAYBACK: &str = "External Mic";

fn mic_playback(mixer: &Mixer) -> Result<Selem<'_>, String> {
    mixer
        .find_selem(&SelemId::new(MIC_PLAYBACK, 0))
        .filter(|selem| selem.has_playback_switch() && selem.has_playback_volume())
        .ok_or_else(|| format!("ALSA playback control '{MIC_PLAYBACK}' not found on {CARD}"))
}

/// The current monitoring state and its level range.
pub(super) fn read(mixer: &Mixer) -> Result<MicMonitoring, String> {
    let selem = mic_playback(mixer)?;
    let enabled = selem
        .get_playback_switch(SelemChannelId::FrontLeft)
        .map_err(|error| format!("cannot read the '{MIC_PLAYBACK}' switch: {error}"))?
        != 0;
    let (min, max) = selem.get_playback_db_range();
    let level = selem
        .get_playback_vol_db(SelemChannelId::FrontLeft)
        .map_err(|error| format!("cannot read the '{MIC_PLAYBACK}' level: {error}"))?;
    Ok(MicMonitoring {
        enabled,
        level_db: level.to_db(),
        min_db: min.to_db(),
        max_db: max.to_db(),
    })
}

/// Switch monitoring on or off and set its level (clamped to the range);
/// returns what the controls took.
pub(super) fn apply(mixer: &Mixer, enabled: bool, level_db: f32) -> Result<MicMonitoring, String> {
    let selem = mic_playback(mixer)?;
    let (min, max) = selem.get_playback_db_range();
    selem
        .set_playback_db_all(
            MilliBel::from_db(level_db.clamp(min.to_db(), max.to_db())),
            Round::Floor,
        )
        .map_err(|error| format!("cannot set the '{MIC_PLAYBACK}' level: {error}"))?;
    selem
        .set_playback_switch_all(i32::from(enabled))
        .map_err(|error| format!("cannot set the '{MIC_PLAYBACK}' switch: {error}"))?;
    read(mixer)
}
