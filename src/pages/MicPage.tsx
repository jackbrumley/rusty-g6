import { EffectControl } from "../components/controls/EffectControl";
import { SelectControl } from "../components/controls/SelectControl";
import { SliderControl } from "../components/controls/SliderControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { SettingsPage } from "../components/ui/SettingsPage";
import { SettingsSection } from "../components/ui/SettingsSection";
import { IS_LINUX } from "../app/platform";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import type { Mixer } from "../hooks/useMixer";
import { MIC_SWITCHES, RECORDING_SOURCES, type G6Settings } from "../types/g6";

interface MicPageProps {
  settings: G6Settings | null;
  mixer: Mixer;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function MicPage({
  settings,
  mixer,
  actions,
  onGoToStatus,
}: MicPageProps) {
  return (
    <>
      <DeviceRequired connected={settings !== null} onGoToStatus={onGoToStatus}>
        {settings && (
          <SettingsPage>
            <SettingsSection title="Input">
              {IS_LINUX ? (
                <SelectControl
                  label="Source"
                  description="What the G6 records. Kept after reboots and replugs."
                  value={mixer.recordingSource}
                  placeholder="Loading..."
                  options={RECORDING_SOURCES}
                  onChange={mixer.setRecordingSource}
                  disabled={mixer.recordingSource === null}
                />
              ) : (
                <p class="info-note">
                  Windows selects the G6 recording source itself.
                </p>
              )}
              {IS_LINUX && mixer.micLevel && (
                <SliderControl
                  label="Mic level"
                  value={mixer.micLevel.level_db}
                  description="Your mic's volume. Raise this before adding boost."
                  min={mixer.micLevel.min_db}
                  max={mixer.micLevel.max_db}
                  step={1}
                  onChange={mixer.setMicLevel}
                  formatValue={(levelDb) => `${levelDb > 0 ? "+" : ""}${levelDb.toFixed(1)} dB`}
                />
              )}
              <SliderControl
                label="Mic boost"
                value={settings.microphone_boost_db ?? 0}
                description="Extra gain for quiet mics. Also raises background noise."
                min={0}
                max={30}
                step={10}
                onChange={actions.setMicrophoneBoost}
                formatValue={(dbValue) =>
                  settings.microphone_boost_db === null
                    ? "Unknown"
                    : `+${dbValue} dB`
                }
              />
            </SettingsSection>

            <SettingsSection title="Processing">
              <EffectControl
                name="Noise reduction"
                description="Removes steady background noise, like fans."
                enabled={settings.mic_processing.noise_reduction}
                level={settings.mic_processing.noise_reduction_level}
                onToggle={(enabled) =>
                  actions.setMicSwitch("noise_reduction", enabled)
                }
                onLevelChange={actions.setNoiseReductionLevel}
              />
              {MIC_SWITCHES.map((micSwitch) => (
                <ToggleControl
                  key={micSwitch.id}
                  label={micSwitch.name}
                  description={micSwitch.description}
                  checked={settings.mic_processing[micSwitch.id]}
                  onChange={(enabled) =>
                    actions.setMicSwitch(micSwitch.id, enabled)
                  }
                />
              ))}
            </SettingsSection>
          </SettingsPage>
        )}
      </DeviceRequired>
    </>
  );
}
