//! The G6's ALSA mixer controls that Rusty G6 manages on Linux: the
//! recording source, the mic level and mic monitoring. Both are USB Audio Class controls the
//! kernel exposes on card `G6`; writing one sends the same SET_CUR request
//! Creative's Windows driver sends. `watch` keeps the stored values applied.
//! (Approach after linuxblaster_control's `src/mixer.rs`, MIT.)
use alsa::mixer::Mixer;

mod mic_level;
mod mic_monitoring;
mod recording_source;
mod watch;
pub use watch::MixerWatch;

/// The kernel names the card after the device, whatever its index.
const CARD: &str = "hw:G6";

fn open_mixer() -> Result<Mixer, String> {
    Mixer::new(CARD, false).map_err(|error| format!("cannot open ALSA mixer {CARD}: {error}"))
}
