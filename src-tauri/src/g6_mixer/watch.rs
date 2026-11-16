//! Keeps the stored mixer settings applied: the recording source whenever
//! the G6's ALSA card appears and whenever another program changes it, the
//! mic level whenever the card appears.
use super::recording_source::{alsa_item, apply};
use super::{mic_level, mic_monitoring, open_mixer};
use crate::g6_spec::{MicLevel, MicMonitoring, MicMonitoringChoice, RecordingSource};
use alsa::mixer::Mixer;
use alsa::poll::{Descriptors, Flags};
use log::{error, info, warn};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

/// How often to look for the card while it is absent.
const CARD_RETRY: Duration = Duration::from_secs(2);
/// Poll timeout while following control changes, in milliseconds.
const POLL_TIMEOUT_MS: i32 = 1000;
/// Quiet time after a change before re-applying, in milliseconds.
const SETTLE_MS: i32 = 1500;
/// If another program switches the source back this often (counted after
/// each burst settles), it is fighting us: stop re-applying (until the user
/// picks a source again) and say so.
const MAX_REAPPLIES: usize = 5;
const REAPPLY_WINDOW: Duration = Duration::from_secs(60);
/// The UI shows this event's payload as an error toast.
const ERROR_EVENT: &str = "mixer-error";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reason {
    CardAppeared,
    ChangedElsewhere,
}

struct WatchState {
    desired: RecordingSource,
    /// `None` until the user sets a mic level: the kernel's level is kept.
    mic_level_db: Option<f32>,
    /// `None` until the user changes monitoring: the kernel's state is kept.
    mic_monitoring: Option<MicMonitoringChoice>,
    /// Set when another program kept switching the source back.
    paused: bool,
    reapplies: VecDeque<Instant>,
    /// The last error reported to the UI, so a lasting error is shown once.
    last_error: Option<String>,
}

#[derive(Clone)]
pub struct MixerWatch {
    state: Arc<Mutex<WatchState>>,
}

impl MixerWatch {
    pub fn new(
        desired: RecordingSource,
        mic_level_db: Option<f32>,
        mic_monitoring: Option<MicMonitoringChoice>,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(WatchState {
                desired,
                mic_level_db,
                mic_monitoring,
                paused: false,
                reapplies: VecDeque::new(),
                last_error: None,
            })),
        }
    }

    /// Start following the card for the life of the app.
    pub fn start(&self, app: AppHandle) {
        let state = self.state.clone();
        thread::spawn(move || run(&state, &app));
    }

    /// Keep `source` selected from now on, and select it now if the G6 is
    /// present. An absent G6 is not an error: the watch applies it when the
    /// card appears.
    pub fn set_recording_source(&self, source: RecordingSource) -> Result<(), String> {
        {
            let mut state = self.state.lock().unwrap();
            state.desired = source;
            state.paused = false;
            state.reapplies.clear();
            state.last_error = None;
        }
        let Ok(mixer) = open_mixer() else {
            info!(
                "RECORDING_SOURCE {} stored; the G6 sound card is not present yet",
                alsa_item(source)
            );
            return Ok(());
        };
        match apply(&mixer, source)? {
            Some(previous) => info!(
                "RECORDING_SOURCE set to {} (was {previous}; chosen by the user)",
                alsa_item(source)
            ),
            None => info!("RECORDING_SOURCE {} already selected", alsa_item(source)),
        }
        Ok(())
    }

    /// The G6's current mic monitoring state (read from the card).
    pub fn mic_monitoring(&self) -> Result<MicMonitoring, String> {
        mic_monitoring::read(&open_mixer()?)
    }

    /// Keep mic monitoring at `choice` from now on and apply it now. Returns
    /// what the controls took.
    pub fn set_mic_monitoring(&self, choice: MicMonitoringChoice) -> Result<MicMonitoring, String> {
        let monitoring = mic_monitoring::apply(&open_mixer()?, choice.enabled, choice.level_db)?;
        self.state.lock().unwrap().mic_monitoring = Some(MicMonitoringChoice {
            enabled: monitoring.enabled,
            level_db: monitoring.level_db,
        });
        info!(
            "MIC_MONITORING {} at {} dB (chosen by the user)",
            if monitoring.enabled { "on" } else { "off" },
            monitoring.level_db
        );
        Ok(monitoring)
    }

    /// The G6's current mic level and range (read from the card).
    pub fn mic_level(&self) -> Result<MicLevel, String> {
        mic_level::read(&open_mixer()?)
    }

    /// Keep the mic level at `level_db` from now on and set it now. Returns
    /// the level the control took.
    pub fn set_mic_level(&self, level_db: f32) -> Result<MicLevel, String> {
        let level = mic_level::apply(&open_mixer()?, level_db)?;
        self.state.lock().unwrap().mic_level_db = Some(level.level_db);
        info!(
            "MIC_LEVEL set to {} dB (chosen by the user)",
            level.level_db
        );
        Ok(level)
    }
}

fn run(state: &Mutex<WatchState>, app: &AppHandle) {
    info!("RECORDING_SOURCE watch started");
    let mut card_present = false;
    loop {
        let mixer = match open_mixer() {
            Ok(mixer) => mixer,
            Err(_) => {
                if card_present {
                    info!("RECORDING_SOURCE G6 sound card removed");
                    card_present = false;
                }
                thread::sleep(CARD_RETRY);
                continue;
            }
        };
        if !card_present {
            info!("RECORDING_SOURCE G6 sound card found");
            card_present = true;
        }

        enforce(state, &mixer, app, Reason::CardAppeared);
        apply_stored_mic_level(state, &mixer, app);
        apply_stored_mic_monitoring(state, &mixer, app);
        if let Err(error) = follow_changes(state, &mixer, app) {
            info!("RECORDING_SOURCE stopped following the card: {error}");
        }
        drop(mixer);
        thread::sleep(CARD_RETRY);
    }
}

/// Wait for control changes and re-apply once they settle. WirePlumber
/// writes the selector several times in a row when it starts or re-activates
/// the G6's input, so re-applying after each write would fight that burst;
/// waiting for `SETTLE_MS` of quiet turns it into one re-apply. Returns when
/// the card goes away or the mixer fails.
fn follow_changes(state: &Mutex<WatchState>, mixer: &Mixer, app: &AppHandle) -> Result<(), String> {
    loop {
        if wait_for_change(mixer, POLL_TIMEOUT_MS)? {
            while wait_for_change(mixer, SETTLE_MS)? {}
            enforce(state, mixer, app, Reason::ChangedElsewhere);
        }
    }
}

/// Whether a control changed within `timeout_ms`. Fails when the card is
/// gone.
fn wait_for_change(mixer: &Mixer, timeout_ms: i32) -> Result<bool, String> {
    let mut fds = mixer.get().map_err(|error| error.to_string())?;
    alsa::poll::poll(&mut fds, timeout_ms).map_err(|error| error.to_string())?;
    let flags = mixer.revents(&fds).map_err(|error| error.to_string())?;
    if flags.intersects(Flags::ERR | Flags::HUP | Flags::NVAL) {
        return Err("the card was disconnected".to_string());
    }
    if flags.is_empty() {
        return Ok(false);
    }
    mixer.handle_events().map_err(|error| error.to_string())?;
    Ok(true)
}

fn enforce(state: &Mutex<WatchState>, mixer: &Mixer, app: &AppHandle, reason: Reason) {
    let desired = {
        let state = state.lock().unwrap();
        if state.paused {
            return;
        }
        state.desired
    };

    match apply(mixer, desired) {
        Ok(None) => {}
        Ok(Some(previous)) => {
            let why = match reason {
                Reason::CardAppeared => "the G6 sound card appeared",
                Reason::ChangedElsewhere => "another program changed it",
            };
            info!(
                "RECORDING_SOURCE set to {} (was {previous}; {why})",
                alsa_item(desired)
            );
            if reason == Reason::ChangedElsewhere {
                record_reapply(state, app, desired);
            }
        }
        Err(message) => {
            error!("RECORDING_SOURCE {message}");
            report_once(
                state,
                app,
                format!("Could not set the G6 recording source: {message}"),
            );
        }
    }
}

/// Apply the stored mic level, if the user set one.
fn apply_stored_mic_level(state: &Mutex<WatchState>, mixer: &Mixer, app: &AppHandle) {
    let Some(level_db) = state.lock().unwrap().mic_level_db else {
        return;
    };
    match mic_level::apply(mixer, level_db) {
        Ok(level) => info!(
            "MIC_LEVEL set to {} dB (the G6 sound card appeared)",
            level.level_db
        ),
        Err(message) => {
            error!("MIC_LEVEL {message}");
            report_once(
                state,
                app,
                format!("Could not set the G6 mic level: {message}"),
            );
        }
    }
}

/// Apply the stored mic monitoring choice, if the user made one.
fn apply_stored_mic_monitoring(state: &Mutex<WatchState>, mixer: &Mixer, app: &AppHandle) {
    let Some(choice) = state.lock().unwrap().mic_monitoring else {
        return;
    };
    match mic_monitoring::apply(mixer, choice.enabled, choice.level_db) {
        Ok(monitoring) => info!(
            "MIC_MONITORING {} at {} dB (the G6 sound card appeared)",
            if monitoring.enabled { "on" } else { "off" },
            monitoring.level_db
        ),
        Err(message) => {
            error!("MIC_MONITORING {message}");
            report_once(
                state,
                app,
                format!("Could not set the G6 mic monitoring: {message}"),
            );
        }
    }
}

/// Count re-applies; pause when another program keeps switching back.
fn record_reapply(state: &Mutex<WatchState>, app: &AppHandle, desired: RecordingSource) {
    let now = Instant::now();
    let fighting = {
        let mut state = state.lock().unwrap();
        state.reapplies.push_back(now);
        while state
            .reapplies
            .front()
            .is_some_and(|time| now.duration_since(*time) > REAPPLY_WINDOW)
        {
            state.reapplies.pop_front();
        }
        let fighting = state.reapplies.len() > MAX_REAPPLIES;
        if fighting {
            state.paused = true;
        }
        fighting
    };
    if fighting {
        warn!(
            "RECORDING_SOURCE another program switched the source back {} times in {} s; \
             no longer re-applying {} until a source is chosen again",
            MAX_REAPPLIES + 1,
            REAPPLY_WINDOW.as_secs(),
            alsa_item(desired)
        );
        report_once(
            state,
            app,
            format!(
                "Another program keeps changing the G6 recording source, so Rusty G6 stopped \
                 setting it to {}. Choose the source again on the Input page to retry.",
                alsa_item(desired)
            ),
        );
    }
}

fn report_once(state: &Mutex<WatchState>, app: &AppHandle, message: String) {
    {
        let mut state = state.lock().unwrap();
        if state.last_error.as_deref() == Some(message.as_str()) {
            return;
        }
        state.last_error = Some(message.clone());
    }
    if let Err(error) = app.emit(ERROR_EVENT, &message) {
        error!("RECORDING_SOURCE could not notify the UI: {error}");
    }
}
