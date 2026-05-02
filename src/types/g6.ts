export interface FirmwareInfo {
  version: string;
  build: string | null;
}

export interface EqualizerBand {
  frequency: number;
  gain: number;
}

export interface EqualizerConfig {
  enabled: "Enabled" | "Disabled";
  bands: EqualizerBand[];
}

export interface ExtendedAudioParams {
  param_0x0a: number | null;
  param_0x0b: number | null;
  param_0x0c: number | null;
  param_0x0d: number | null;
  param_0x0e: number | null;
  param_0x0f: number | null;
  param_0x10: number | null;
  param_0x11: number | null;
  param_0x12: number | null;
  param_0x13: number | null;
  param_0x14: number | null;
  param_0x1a: number | null;
  param_0x1b: number | null;
  param_0x1c: number | null;
  param_0x1d: number | null;
}

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
  equalizer: EqualizerConfig | null;
  extended_params: ExtendedAudioParams | null;
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
}
