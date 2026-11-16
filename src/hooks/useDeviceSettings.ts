import { invoke } from "@tauri-apps/api/core";
import type { DecoderMode, DeviceFlag, DigitalFilter, MicSwitch, RgbColor, SbxEffect, ToastType } from "../types/g6";

export type DeviceSettingsActions = ReturnType<typeof useDeviceSettings>;

interface UseDeviceSettingsProps {
  showToast: (message: string, type: ToastType, durationMs?: number) => void;
}

/// Every device setting change. The backend logs each request, applies the
/// device's answer to its cached settings and emits "device-update", which
/// reloads `settings`; this hook only reports failures.
export function useDeviceSettings({ showToast }: UseDeviceSettingsProps) {
  const change = async (command: string, args: Record<string, unknown>, failure: string) => {
    try {
      await invoke(command, args);
    } catch (error) {
      console.error(`${failure}:`, error);
      showToast(`${failure}: ${error}`, "error", 4000);
    }
  };

  return {
    toggleOutput: () => change("toggle_output", {}, "Failed to switch the output"),
    setSbxEnabled: (enabled: boolean) =>
      change("set_sbx_enabled", { enabled }, "Failed to switch SBX"),
    setScoutModeEnabled: (enabled: boolean) =>
      change("set_scout_mode_enabled", { enabled }, "Failed to switch Scout Mode"),
    setEffectEnabled: (effect: SbxEffect, enabled: boolean) =>
      change("set_effect_enabled", { effect, enabled }, "Failed to switch the effect"),
    setEffectLevel: (effect: SbxEffect, level: number) =>
      change("set_effect_level", { effect, level }, "Failed to set the effect level"),
    setEqEnabled: (enabled: boolean) => change("set_eq_enabled", { enabled }, "Failed to switch the EQ"),
    setEqPreamp: (gainDb: number) => change("set_eq_preamp", { gainDb }, "Failed to set the EQ preamp"),
    setEqBand: (band: number, gainDb: number) =>
      change("set_eq_band", { band, gainDb }, "Failed to set the EQ band"),
    setEqBands: (gainsDb: number[]) => change("set_eq_bands", { gainsDb }, "Failed to apply the EQ preset"),
    setDeviceFlag: (flag: DeviceFlag, enabled: boolean) =>
      change("set_device_flag", { flag, enabled }, "Failed to switch the device feature"),
    setDecoderMode: (mode: DecoderMode) =>
      change("set_decoder_mode", { mode }, "Failed to set the decoder mode"),
    setLightingEnabled: (enabled: boolean) =>
      change("set_lighting_enabled", { enabled }, "Failed to switch the lighting"),
    setLightingColor: (color: RgbColor) =>
      change("set_lighting_color", { color }, "Failed to set the lighting colour"),
    setDacFilter: (filter: DigitalFilter) =>
      change("set_dac_filter", { filter }, "Failed to set the DAC filter"),
    setMicSwitch: (micSwitch: MicSwitch, enabled: boolean) =>
      change("set_mic_switch", { switch: micSwitch, enabled }, "Failed to switch the mic feature"),
    setMicEqBands: (gainsDb: number[]) =>
      change("set_mic_eq_bands", { gainsDb }, "Failed to apply the mic EQ preset"),
    setNoiseReductionLevel: (level: number) =>
      change("set_noise_reduction_level", { level }, "Failed to set the noise reduction level"),
    setMicrophoneBoost: (dbValue: number) =>
      change("set_microphone_boost", { dbValue }, "Failed to set the mic boost"),
  };
}
