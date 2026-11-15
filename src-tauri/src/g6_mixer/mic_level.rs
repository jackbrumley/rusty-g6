//! The mic level: the ALSA `External Mic` capture volume, a gain stage in
//! the G6 after the mic boost (-48 to +9 dB). PipeWire never drives it (its
//! G6 input controls `Line In`), so Rusty G6 stores it and applies it when
//! the card appears.
use super::CARD;
use crate::g6_spec::MicLevel;
use alsa::mixer::{Mixer, Selem, SelemChannelId, SelemId};
use alsa::Round;

const MIC_CAPTURE: &str = "External Mic";

fn mic_capture(mixer: &Mixer) -> Result<Selem<'_>, String> {
    mixer
        .find_selem(&SelemId::new(MIC_CAPTURE, 0))
        .filter(Selem::has_capture_volume)
        .ok_or_else(|| format!("ALSA capture volume '{MIC_CAPTURE}' not found on {CARD}"))
}

/// The current mic level and its range.
pub(super) fn read(mixer: &Mixer) -> Result<MicLevel, String> {
    let selem = mic_capture(mixer)?;
    let (min, max) = selem.get_capture_db_range();
    let level = selem
        .get_capture_vol_db(SelemChannelId::FrontLeft)
        .map_err(|error| format!("cannot read '{MIC_CAPTURE}': {error}"))?;
    Ok(MicLevel {
        level_db: level.to_db(),
        min_db: min.to_db(),
        max_db: max.to_db(),
    })
}

/// Set the mic level (clamped to its range, nearest step) and return what
/// the control took.
pub(super) fn apply(mixer: &Mixer, level_db: f32) -> Result<MicLevel, String> {
    let selem = mic_capture(mixer)?;
    let (min, max) = selem.get_capture_db_range();
    let clamped = level_db.clamp(min.to_db(), max.to_db());
    selem
        .set_capture_db_all(alsa::mixer::MilliBel::from_db(clamped), Round::Floor)
        .map_err(|error| format!("cannot set '{MIC_CAPTURE}': {error}"))?;
    read(mixer)
}
