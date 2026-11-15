import { EffectControl } from "../components/controls/EffectControl";
import { EqualizerControl } from "../components/controls/EqualizerControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { Tooltip } from "../components/ui/Tooltip";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import { DIGITAL_FILTERS, SBX_EFFECTS, type DigitalFilter, type G6Settings } from "../types/g6";

interface OutputPageProps {
  connected: boolean;
  settings: G6Settings | null;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

const HEADER_SUBTITLE = "Playback routing and SBX effect controls for your current listening setup.";

export function OutputPage({ connected, settings, actions, onGoToStatus }: OutputPageProps) {
  if (!connected || !settings) {
    return (
      <>
        <TabPageHeader title="Output" subtitle={HEADER_SUBTITLE} />

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
      <TabPageHeader title="Output" subtitle={HEADER_SUBTITLE} />

      <section class="output-section compact surface-card">
        <div class="section-line">
          <span class="section-label">Output:</span>
          <span class="section-value">{settings.output}</span>
          <Tooltip text="Switches the active playback route between speakers and headphones on the G6.">
            <button onClick={actions.toggleOutput} class="btn-compact">
              Toggle Output
            </button>
          </Tooltip>
        </div>

        <div class="effects-list">
          <h3>Audio Effects</h3>
          <div class="output-controls-grid">
            <ToggleControl
              label="Scout Mode"
              checked={settings.scout_mode_enabled}
              tooltip="Highlights positional audio cues such as footsteps and directional movement in games."
              onChange={actions.setScoutModeEnabled}
            />

            <ToggleControl
              label="SBX Mode"
              checked={settings.sbx_enabled}
              tooltip="Master switch for SBX processing. Disable it to bypass SBX effect controls below."
              onChange={actions.setSbxEnabled}
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
          </div>
        </div>

        <div class="effects-list">
          <h3>DAC</h3>
          <div class="output-controls-grid">
            <div class="control-row">
              <span class="control-label">
                <Tooltip text="The DAC's digital reconstruction filter. Differences are subtle; NOS skips oversampling entirely.">
                  Filter
                </Tooltip>
              </span>
              <select
                class="select-control"
                value={settings.digital_filter ?? ""}
                onChange={(event) =>
                  actions.setDacFilter((event.currentTarget as HTMLSelectElement).value as DigitalFilter)
                }
              >
                {settings.digital_filter === null && (
                  <option value="" disabled>
                    Unknown
                  </option>
                )}
                {DIGITAL_FILTERS.map((filter) => (
                  <option key={filter.id} value={filter.id}>
                    {filter.name}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        <EqualizerControl
          equalizer={settings.equalizer}
          onSetEnabled={actions.setEqEnabled}
          onSetPreamp={actions.setEqPreamp}
          onSetBand={actions.setEqBand}
          onSetBands={actions.setEqBands}
        />
      </section>
    </>
  );
}
