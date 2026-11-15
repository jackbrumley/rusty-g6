//! Keeps the stored recording source selected: applies it whenever the G6's
//! ALSA card appears, and again whenever another program changes it.
use super::{alsa_item, apply, open_mixer};
use crate::g6_spec::RecordingSource;
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
const ERROR_EVENT: &str = "recording-source-error";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Reason {
    CardAppeared,
    ChangedElsewhere,
}

struct WatchState {
    desired: RecordingSource,
    /// Set when another program kept switching the source back.
    paused: bool,
    reapplies: VecDeque<Instant>,
    /// The last error reported to the UI, so a lasting error is shown once.
    last_error: Option<String>,
}

#[derive(Clone)]
pub struct RecordingSourceWatch {
    state: Arc<Mutex<WatchState>>,
}

impl RecordingSourceWatch {
    pub fn new(desired: RecordingSource) -> Self {
        Self {
            state: Arc::new(Mutex::new(WatchState {
                desired,
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
    pub fn set(&self, source: RecordingSource) -> Result<(), String> {
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
