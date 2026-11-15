import { SliderControl } from "../components/controls/SliderControl";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { Tooltip } from "../components/ui/Tooltip";
import { IS_LINUX } from "../app/platform";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import { RECORDING_SOURCES, type G6Settings, type RecordingSource } from "../types/g6";

interface InputPageProps {
  connected: boolean;
  settings: G6Settings | null;
  recordingSource: RecordingSource | null;
  onSetRecordingSource: (source: RecordingSource) => void;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function InputPage({
  connected,
  settings,
  recordingSource,
  onSetRecordingSource,
  actions,
  onGoToStatus,
}: InputPageProps) {
  if (!connected || !settings) {
    return (
      <>
        <TabPageHeader
          title="Input"
          subtitle="Microphone setup and gain controls for voice capture tuning."
        />

        <div class="info-panel surface-card">
          <p>Connect your SoundBlaster X G6 from the Status tab to begin.</p>
          <p class="info-note">This page only shows controls once a device session is active.</p>
          <button class="btn-compact btn-secondary" onClick={onGoToStatus}>
            Go to Status
          </button>
        </div>
      </>
    );
  }

  return (
    <>
      <TabPageHeader
        title="Input"
        subtitle="Microphone setup and gain controls for voice capture tuning."
      />

      <section class="input-section compact surface-card">
        {IS_LINUX ? (
          <div class="control-row">
            <span class="control-label">
              <Tooltip text="What the G6 records. Rusty G6 keeps this selected, also after reboots and replugs, because Linux resets it to Line In.">
                Recording source
              </Tooltip>
            </span>
            <select
              class="select-control"
              value={recordingSource ?? ""}
              disabled={recordingSource === null}
              onChange={(event) =>
                onSetRecordingSource((event.currentTarget as HTMLSelectElement).value as RecordingSource)
              }
            >
              {RECORDING_SOURCES.map((source) => (
                <option key={source.id} value={source.id}>
                  {source.name}
                </option>
              ))}
            </select>
          </div>
        ) : (
          <p class="info-note">Windows selects the G6 recording source itself.</p>
        )}

        <SliderControl
          label="Mic Boost"
          value={settings.microphone_boost_db ?? 0}
          tooltip="Microphone input gain in 10 dB steps. Higher values increase sensitivity and background pickup."
          min={0}
          max={30}
          step={10}
          onChange={actions.setMicrophoneBoost}
          formatValue={(dbValue) => (settings.microphone_boost_db === null ? "Unknown" : `+${dbValue} dB`)}
        />
      </section>
    </>
  );
}
