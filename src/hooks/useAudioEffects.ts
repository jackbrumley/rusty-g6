import { invoke } from "@tauri-apps/api/core";
import type { ToastType } from "../types/g6";

interface UseAudioEffectsProps {
  showToast: (message: string, type: ToastType, durationMs?: number) => void;
  logUiEvent: (message: string) => Promise<void>;
}

/// Playback effect writes: SBX and Scout mode, the SBX effects and the
/// graphic EQ. Each one reports a failure with a toast.
export function useAudioEffects({ showToast, logUiEvent }: UseAudioEffectsProps) {
  const setSbxMode = async (enabled: "Enabled" | "Disabled") => {
    try {
      console.log("Setting SBX Mode:", enabled);
      await logUiEvent(`SBX mode requested: ${enabled}`);
      await invoke("set_sbx_mode", { enabled });
    } catch (error) {
      console.error("Failed to set SBX Mode:", error);
      showToast(`Failed to set SBX Mode: ${error}`, "error");
    }
  };

  const setScoutMode = async (enabled: "Enabled" | "Disabled") => {
    try {
      console.log("Setting Scout Mode:", enabled);
      await logUiEvent(`Scout mode requested: ${enabled}`);
      await invoke("set_scout_mode", { enabled });
    } catch (error) {
      console.error("Failed to set Scout Mode:", error);
      showToast(`Failed to set Scout Mode: ${error}`, "error");
    }
  };

  const setEffect = async (
    effectName: string,
    enabled: "Enabled" | "Disabled",
    value: number
  ) => {
    try {
      console.log(`Setting ${effectName}:`, { enabled, value });
      await logUiEvent(`Effect change requested: ${effectName} enabled=${enabled} value=${value}`);
      const result = await invoke(`set_${effectName}`, { enabled, value });
      console.log(`${effectName} result:`, result);
    } catch (error) {
      console.error(`Failed to set ${effectName}:`, error);
      showToast(`Failed to set ${effectName}: ${error}`, "error", 3000);
    }
  };

  const setEqEnabled = async (enabled: "Enabled" | "Disabled") => {
    try {
      await logUiEvent(`EQ requested: ${enabled}`);
      await invoke("set_eq_enabled", { enabled });
    } catch (error) {
      console.error("Failed to set EQ:", error);
      showToast(`Failed to set EQ: ${error}`, "error", 3000);
    }
  };

  const setEqPreamp = async (gainDb: number) => {
    try {
      await logUiEvent(`EQ preamp requested: ${gainDb} dB`);
      await invoke<number>("set_eq_preamp", { gainDb });
    } catch (error) {
      console.error("Failed to set EQ preamp:", error);
      showToast(`Failed to set EQ preamp: ${error}`, "error", 3000);
    }
  };

  const setEqBand = async (band: number, gainDb: number) => {
    try {
      await logUiEvent(`EQ band ${band} requested: ${gainDb} dB`);
      await invoke<number>("set_eq_band", { band, gainDb });
    } catch (error) {
      console.error(`Failed to set EQ band ${band}:`, error);
      showToast(`Failed to set EQ band: ${error}`, "error", 3000);
    }
  };

  const setEqBands = async (gainsDb: number[]) => {
    try {
      await logUiEvent(`EQ preset requested: ${gainsDb.join(", ")} dB`);
      await invoke<number[]>("set_eq_bands", { gainsDb });
    } catch (error) {
      console.error("Failed to set EQ preset:", error);
      showToast(`Failed to set EQ preset: ${error}`, "error", 3000);
    }
  };

  return {
    setSbxMode,
    setScoutMode,
    setEffect,
    setEqEnabled,
    setEqPreamp,
    setEqBand,
    setEqBands,
  };
}
