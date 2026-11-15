import { SelectControl } from "../components/controls/SelectControl";
import { SliderControl } from "../components/controls/SliderControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { SettingsSection } from "../components/ui/SettingsSection";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { IS_LINUX } from "../app/platform";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import { RECORDING_SOURCES, type G6Settings, type RecordingSource } from "../types/g6";

interface MicPageProps {
  settings: G6Settings | null;
  recordingSource: RecordingSource | null;
  onSetRecordingSource: (source: RecordingSource) => void;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function MicPage({
  settings,
  recordingSource,
  onSetRecordingSource,
  actions,
  onGoToStatus,
}: MicPageProps) {
  return (
    <>
      <TabPageHeader title="Microphone" subtitle="What the G6 records, and how loud." />
      <DeviceRequired connected={settings !== null} onGoToStatus={onGoToStatus}>
        {settings && (
          <SettingsSection title="Input">
            {IS_LINUX ? (
              <SelectControl
                label="Source"
                tooltip="What the G6 records. Rusty G6 keeps this selected, also after reboots and replugs, because Linux resets it to Line In."
                value={recordingSource}
                placeholder="Loading..."
                options={RECORDING_SOURCES}
                onChange={onSetRecordingSource}
                disabled={recordingSource === null}
              />
            ) : (
              <p class="info-note">Windows selects the G6 recording source itself.</p>
            )}
            <SliderControl
              label="Mic boost"
              value={settings.microphone_boost_db ?? 0}
              tooltip="Microphone gain in 10 dB steps. Higher values also pick up more background noise."
              min={0}
              max={30}
              step={10}
              onChange={actions.setMicrophoneBoost}
              formatValue={(dbValue) => (settings.microphone_boost_db === null ? "Unknown" : `+${dbValue} dB`)}
            />
          </SettingsSection>
        )}
      </DeviceRequired>
    </>
  );
}
