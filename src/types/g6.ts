export interface FirmwareInfo {
  version: string;
  build: string | null;
}

/// `g6_spec::EqualizerSettings`: the graphic EQ of the active output.
export interface EqualizerSettings {
  enabled: "Enabled" | "Disabled";
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

export interface G6Settings {
  output: "Speakers" | "Headphones";
  surround_enabled: "Enabled" | "Disabled";
  surround_value: number;
  crystalizer_enabled: "Enabled" | "Disabled";
  crystalizer_value: number;
  bass_enabled: "Enabled" | "Disabled";
  bass_value: number;
  smart_volume_enabled: "Enabled" | "Disabled";
  smart_volume_value: number;
  smart_volume_preset: "Night" | "Loud" | null;
  dialog_plus_enabled: "Enabled" | "Disabled";
  dialog_plus_value: number;
  microphone_boost: number;
  sbx_enabled: "Enabled" | "Disabled";
  firmware_info: FirmwareInfo | null;
  scout_mode: "Enabled" | "Disabled";
  equalizer: EqualizerSettings;
  is_connected: boolean;
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
}

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
