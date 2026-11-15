import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "preact/hooks";
import { IS_LINUX } from "../app/platform";
import type { AppSettings, MicLevel, RecordingSource, ToastType } from "../types/g6";

export type Mixer = ReturnType<typeof useMixer>;

/// The G6's ALSA mixer settings on Linux: the recording source and the mic
/// level. The backend stores both and keeps them applied; it reports
/// trouble with a "mixer-error" event.
export function useMixer(showToast: (message: string, type: ToastType, durationMs?: number) => void) {
  const [recordingSource, setRecordingSourceState] = useState<RecordingSource | null>(null);
  const [micLevel, setMicLevelState] = useState<MicLevel | null>(null);

  const loadMicLevel = () =>
    invoke<MicLevel>("get_mic_level")
      .then(setMicLevelState)
      .catch((error) => console.error("Failed to read the mic level:", error));

  useEffect(() => {
    if (!IS_LINUX) {
      return;
    }
    invoke<RecordingSource>("get_recording_source")
      .then(setRecordingSourceState)
      .catch((error) => showToast(`Failed to read the recording source: ${error}`, "error", 4000));
    void loadMicLevel();

    const unlistenError = listen<string>("mixer-error", (event) => showToast(event.payload, "error", 8000));
    // The G6's sound card comes and goes with the device.
    const unlistenDevice = listen("device-update", () => void loadMicLevel());
    return () => {
      unlistenError.then((unlisten) => unlisten());
      unlistenDevice.then((unlisten) => unlisten());
    };
  }, []);

  const setRecordingSource = async (source: RecordingSource) => {
    try {
      const updated = await invoke<AppSettings>("set_recording_source", { source });
      setRecordingSourceState(updated.recording_source);
    } catch (error) {
      console.error("Failed to set the recording source:", error);
      showToast(`Failed to set the recording source: ${error}`, "error", 4000);
    }
  };

  const setMicLevel = async (levelDb: number) => {
    try {
      setMicLevelState(await invoke<MicLevel>("set_mic_level", { levelDb }));
    } catch (error) {
      console.error("Failed to set the mic level:", error);
      showToast(`Failed to set the mic level: ${error}`, "error", 4000);
    }
  };

  return { recordingSource, setRecordingSource, micLevel, setMicLevel };
}
