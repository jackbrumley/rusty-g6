import { EffectControl } from "../components/controls/EffectControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { SettingsSection } from "../components/ui/SettingsSection";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { Tooltip } from "../components/ui/Tooltip";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import { SBX_EFFECTS, type G6Settings } from "../types/g6";

interface SoundPageProps {
  settings: G6Settings | null;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function SoundPage({ settings, actions, onGoToStatus }: SoundPageProps) {
  return (
    <>
      <TabPageHeader title="Sound" subtitle="Output and SBX effects for what you're listening to." />
      <DeviceRequired connected={settings !== null} onGoToStatus={onGoToStatus}>
        {settings && (
          <>
            <SettingsSection title="Output">
              <div class="control-row">
                <span class="control-label">Playing on</span>
                <span class="control-value">{settings.output}</span>
                <Tooltip text="Switches the G6 between its speaker and headphone outputs.">
                  <button onClick={actions.toggleOutput} class="btn-compact">
                    Switch to {settings.output === "Speakers" ? "Headphones" : "Speakers"}
                  </button>
                </Tooltip>
              </div>
            </SettingsSection>

            <SettingsSection title="SBX">
              <ToggleControl
                label="SBX"
                checked={settings.sbx_enabled}
                tooltip="Master switch for the SBX effects below."
                onChange={actions.setSbxEnabled}
              />
              <ToggleControl
                label="Scout Mode"
                checked={settings.scout_mode_enabled}
                tooltip="Highlights positional cues such as footsteps in games. Switches SBX off."
                onChange={actions.setScoutModeEnabled}
              />
              {SBX_EFFECTS.map((effect) => (
                <EffectControl
                  key={effect.id}
                  name={effect.name}
                  tooltip={effect.tooltip}
                  enabled={settings.effects[effect.id].enabled}
                  level={settings.effects[effect.id].level}
                  onToggle={(enabled) => actions.setEffectEnabled(effect.id, enabled)}
                  onLevelChange={(level) => actions.setEffectLevel(effect.id, level)}
                  disabled={!settings.sbx_enabled}
                />
              ))}
            </SettingsSection>
          </>
        )}
      </DeviceRequired>
    </>
  );
}
