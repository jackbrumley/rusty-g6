import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "preact/hooks";
import { IS_LINUX } from "../app/platform";
import type { AppSettings, RecordingSource, ToastType } from "../types/g6";

interface UseRecordingSourceProps {
  showToast: (message: string, type: ToastType, durationMs?: number) => void;
}

/// The G6 recording source on Linux. The backend stores it and keeps it
/// selected; it reports trouble with a "recording-source-error" event.
export function useRecordingSource({ showToast }: UseRecordingSourceProps) {
  const [recordingSource, setRecordingSourceState] = useState<RecordingSource | null>(null);

  useEffect(() => {
    if (!IS_LINUX) {
      return;
    }
    invoke<RecordingSource>("get_recording_source")
      .then(setRecordingSourceState)
      .catch((error) => showToast(`Failed to read the recording source: ${error}`, "error", 4000));

    const unlistenPromise = listen<string>("recording-source-error", (event) => {
      showToast(event.payload, "error", 8000);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
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

  return { recordingSource, setRecordingSource };
}
