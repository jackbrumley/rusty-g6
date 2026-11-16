import { EffectControl } from "../components/controls/EffectControl";
import { SelectControl } from "../components/controls/SelectControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { SettingsPage } from "../components/ui/SettingsPage";
import { SettingsSection } from "../components/ui/SettingsSection";
import { Tooltip } from "../components/ui/Tooltip";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import { DECODER_MODES, SBX_EFFECTS, type G6Settings } from "../types/g6";

interface SoundPageProps {
  settings: G6Settings | null;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function SoundPage({ settings, actions, onGoToStatus }: SoundPageProps) {
  return (
    <>
      <DeviceRequired settings={settings} onGoToStatus={onGoToStatus}>
        {settings && (
          <SettingsPage>
            <SettingsSection title="Output">
              <div class="control-row">
                <span class="control-label">Playing on</span>
                <span class="control-value">{settings.output}</span>
                <Tooltip text={`Switches the G6 to its ${settings.output === "Speakers" ? "headphone" : "speaker"} output.`}>
                  <button onClick={actions.toggleOutput} class="btn-compact">
                    Switch
                  </button>
                </Tooltip>
              </div>
            </SettingsSection>

            <SettingsSection title="Decoder">
              <SelectControl
                label="Dynamic range"
                description="For Dolby and DTS sound. Night evens out loud and quiet parts for late listening."
                value={settings.decoder_mode}
                options={DECODER_MODES}
                onChange={actions.setDecoderMode}
              />
            </SettingsSection>

            <SettingsSection title="SBX">
              <ToggleControl
                label="SBX"
                checked={settings.sbx_enabled}
                description="Master switch for the effects below."
                onChange={actions.setSbxEnabled}
              />
              <ToggleControl
                label="Scout Mode"
                checked={settings.scout_mode_enabled}
                description="Brings out footsteps and other game cues. Turns SBX off."
                onChange={actions.setScoutModeEnabled}
              />
              {SBX_EFFECTS.map((effect) => (
                <EffectControl
                  key={effect.id}
                  name={effect.name}
                  description={effect.description}
                  enabled={settings.effects[effect.id].enabled}
                  level={settings.effects[effect.id].level}
                  onToggle={(enabled) => actions.setEffectEnabled(effect.id, enabled)}
                  onLevelChange={(level) => actions.setEffectLevel(effect.id, level)}
                  disabled={!settings.sbx_enabled}
                />
              ))}
            </SettingsSection>
          </SettingsPage>
        )}
      </DeviceRequired>
    </>
  );
}
