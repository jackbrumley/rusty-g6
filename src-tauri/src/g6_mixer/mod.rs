//! The G6's recording source on Linux. The kernel exposes the G6's USB Audio
//! Class selector as the ALSA enum `PCM Capture Source` on card `G6`; writing
//! it sends the same SET_CUR request Creative's Windows driver sends.
//!
//! The kernel starts it on Line In at every enumeration, alsa-restore can put
//! a stored Line In back at boot, and PipeWire's Line In port selects Line In
//! whenever it activates. So the source is not set once: `watch` re-applies
//! the stored choice whenever the card appears or the control changes.
//! (Approach after linuxblaster_control's `src/mixer.rs`, MIT.)
use crate::g6_spec::RecordingSource;
use alsa::mixer::{Mixer, Selem, SelemChannelId, SelemId};

mod watch;
pub use watch::RecordingSourceWatch;

/// The kernel names the card after the device, whatever its index.
const CARD: &str = "hw:G6";
const CAPTURE_SOURCE: &str = "PCM Capture Source";

/// The item name of `source` in the ALSA `PCM Capture Source` enum.
pub fn alsa_item(source: RecordingSource) -> &'static str {
    match source {
        RecordingSource::LineIn => "Line In",
        RecordingSource::ExternalMic => "External Mic",
        RecordingSource::SpdifIn => "S/PDIF In",
        RecordingSource::WhatUHear => "What U Hear",
    }
}

fn open_mixer() -> Result<Mixer, String> {
    Mixer::new(CARD, false).map_err(|error| format!("cannot open ALSA mixer {CARD}: {error}"))
}

fn capture_source(mixer: &Mixer) -> Result<Selem<'_>, String> {
    mixer
        .find_selem(&SelemId::new(CAPTURE_SOURCE, 0))
        .filter(Selem::is_enumerated)
        .ok_or_else(|| format!("ALSA control '{CAPTURE_SOURCE}' not found on {CARD}"))
}

/// The item name currently selected in `PCM Capture Source`.
fn current_item(selem: &Selem) -> Result<String, String> {
    let index = selem
        .get_enum_item(SelemChannelId::mono())
        .map_err(|error| format!("cannot read '{CAPTURE_SOURCE}': {error}"))?;
    selem
        .get_enum_item_name(index)
        .map_err(|error| format!("cannot name '{CAPTURE_SOURCE}' item {index}: {error}"))
}

/// Selects `source` unless it is already selected. Returns the item that
/// was selected before when it changed something, `None` otherwise.
fn apply(mixer: &Mixer, source: RecordingSource) -> Result<Option<String>, String> {
    let selem = capture_source(mixer)?;
    let previous = current_item(&selem)?;
    if previous == alsa_item(source) {
        return Ok(None);
    }

    let index = (0..selem.get_enum_items().map_err(|error| error.to_string())?)
        .find(|index| selem.get_enum_item_name(*index).as_deref() == Ok(alsa_item(source)))
        .ok_or_else(|| format!("'{CAPTURE_SOURCE}' has no item '{}'", alsa_item(source)))?;
    selem
        .set_enum_item(SelemChannelId::mono(), index)
        .map_err(|error| format!("cannot set '{CAPTURE_SOURCE}': {error}"))?;
    Ok(Some(previous))
}
