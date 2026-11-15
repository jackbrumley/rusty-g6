/// `g6_spec::SbxEffect`: also the keys of `G6Settings.effects`.
export type SbxEffect = "surround" | "crystalizer" | "bass" | "smart_volume" | "dialog_plus";

/// `g6_spec::EffectSettings`.
export interface EffectSettings {
  enabled: boolean;
  /// 0-100.
  level: number;
}

export const SBX_EFFECTS: { id: SbxEffect; name: string; tooltip: string }[] = [
  {
    id: "surround",
    name: "Surround",
    tooltip: "Expands stereo imaging into a wider virtual surround field.",
  },
  {
    id: "crystalizer",
    name: "Crystalizer",
    tooltip: "Restores perceived detail and clarity in compressed or dull source audio.",
  },
  { id: "bass", name: "Bass", tooltip: "Boosts low-end presence and punch for fuller bass response." },
  {
    id: "smart_volume",
    name: "Smart Volume",
    tooltip: "Smooths loudness swings so quieter and louder moments stay more consistent.",
  },
  {
    id: "dialog_plus",
    name: "Dialog Plus",
    tooltip: "Brings spoken dialogue forward so voices remain clear over background audio.",
  },
];

/// `g6_spec::MicSwitch`: also the keys of `MicProcessing`.
export type MicSwitch = "noise_reduction" | "echo_cancellation" | "smart_volume" | "equalizer";

/// `g6_spec::MicProcessing`.
export interface MicProcessing {
  noise_reduction: boolean;
  /// 0-100.
  noise_reduction_level: number;
  echo_cancellation: boolean;
  smart_volume: boolean;
  equalizer: boolean;
}

/// The on/off mic features shown as plain switches (noise reduction has a
/// level too and is shown separately).
export const MIC_SWITCHES: { id: Exclude<MicSwitch, "noise_reduction">; name: string; tooltip: string }[] = [
  {
    id: "echo_cancellation",
    name: "Echo cancellation",
    tooltip: "Removes speaker sound that the mic picks up, for calls without headphones.",
  },
  {
    id: "smart_volume",
    name: "Smart volume",
    tooltip: "Evens out your voice level, so quiet and loud speech come through alike.",
  },
  { id: "equalizer", name: "Mic EQ", tooltip: "The mic's own 8-band EQ, as last set on the G6." },
];

/// `g6_spec::MICROPHONE_BOOST_STEPS_DB`.
export const MICROPHONE_BOOST_STEPS_DB = [0, 10, 20, 30];

/// `g6_spec::EqualizerSettings`: the graphic EQ of the active output.
export interface EqualizerSettings {
  enabled: boolean;
  preamp_db: number;
  /// Gains in dB of the bands in `EQ_BAND_FREQUENCIES`.
  band_gains_db: number[];
}

/// Band labels, in the order of `band_gains_db` (G6 bands 0x0b-0x14).
export const EQ_BAND_FREQUENCIES = ["31", "62", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];

/// Ranges and step in dB (device range dump; `g6_spec.rs` mirrors them).
export const EQ_PREAMP_LIMIT_DB = 6;
export const EQ_BAND_LIMIT_DB = 12;
export const EQ_GAIN_STEP_DB = 0.5;

/// `g6_spec::DigitalFilter`: the DAC's reconstruction filter.
export type DigitalFilter =
  | "FastRollOffMinimumPhase"
  | "SlowRollOffMinimumPhase"
  | "NonOversampling"
  | "FastRollOffLinearPhase"
  | "SlowRollOffLinearPhase";

export const DIGITAL_FILTERS: { id: DigitalFilter; name: string }[] = [
  { id: "FastRollOffMinimumPhase", name: "Fast roll-off, minimum phase" },
  { id: "SlowRollOffMinimumPhase", name: "Slow roll-off, minimum phase" },
  { id: "FastRollOffLinearPhase", name: "Fast roll-off, linear phase" },
  { id: "SlowRollOffLinearPhase", name: "Slow roll-off, linear phase" },
  { id: "NonOversampling", name: "NOS (non-oversampling)" },
];

/// `g6_spec::G6Settings`: what the backend knows of the device's state.
export interface G6Settings {
  output: "Speakers" | "Headphones";
  sbx_enabled: boolean;
  scout_mode_enabled: boolean;
  effects: Record<SbxEffect, EffectSettings>;
  equalizer: EqualizerSettings;
  digital_filter: DigitalFilter | null;
  /// `null` until read from the device.
  microphone_boost_db: number | null;
  mic_processing: MicProcessing;
  firmware_version: string | null;
  last_read_time: number | null;
}

export type ToastType = "success" | "error" | "info";

export interface ToastMessage {
  id: number;
  message: string;
  type: ToastType;
  durationMs: number;
}

export interface UpdateCheckResult {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  releaseUrl: string;
  notesUrl: string | null;
}

/// Backend settings (`app::settings::AppSettings`).
export interface AppSettings {
  close_to_tray: boolean;
  log_level: string;
  recording_source: RecordingSource;
  mic_level_db: number | null;
}

/// `g6_spec::MicLevel`: the G6's mic level (Linux) and its range, in dB.
export interface MicLevel {
  level_db: number;
  min_db: number;
  max_db: number;
}

/// `g6_spec::RecordingSource`: the G6's `PCM Capture Source` on Linux.
export type RecordingSource = "LineIn" | "ExternalMic" | "SpdifIn" | "WhatUHear";

export const RECORDING_SOURCES: { id: RecordingSource; name: string }[] = [
  { id: "ExternalMic", name: "Microphone (front jack)" },
  { id: "LineIn", name: "Line In" },
  { id: "SpdifIn", name: "S/PDIF In" },
  { id: "WhatUHear", name: "What U Hear (playback)" },
];

/// `app::system_info::SystemInfo`: the session log header and the Logs block.
export interface SystemInfo {
  appName: string;
  appVersion: string;
  buildProfile: string;
  executable: string;
  os: string;
  distribution: string;
  architecture: string;
  desktop: string;
  sessionType: string;
}
