import { EffectControl } from "../components/controls/EffectControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import { Tooltip } from "../components/ui/Tooltip";
import type { G6Settings } from "../types/g6";

interface OutputPageProps {
  settings: G6Settings;
  onToggleOutput: () => void;
  onSetScoutMode: (enabled: "Enabled" | "Disabled") => void;
  onSetSbxMode: (enabled: "Enabled" | "Disabled") => void;
  onSetEffect: (
    effectName: string,
    enabled: "Enabled" | "Disabled",
    value: number
  ) => void;
}

export function OutputPage({
  settings,
  onToggleOutput,
  onSetScoutMode,
  onSetSbxMode,
  onSetEffect,
}: OutputPageProps) {
  return (
    <section class="output-section compact surface-card">
      <div class="section-line">
        <span class="section-label">Output:</span>
        <span class="section-value">{settings.output}</span>
        <Tooltip text="Switches the active playback route between speakers and headphones on the G6.">
          <button onClick={onToggleOutput} class="btn-compact">
            Toggle Output
          </button>
        </Tooltip>
      </div>

      <div class="effects-list">
        <h3>Audio Effects</h3>
        <div class="output-controls-grid">
          <ToggleControl
            label="Scout Mode"
            checked={settings.scout_mode === "Enabled"}
            tooltip="Highlights positional audio cues such as footsteps and directional movement in games."
            onChange={(enabled) => onSetScoutMode(enabled ? "Enabled" : "Disabled")}
          />

          <ToggleControl
            label="SBX Mode"
            checked={settings.sbx_enabled === "Enabled"}
            tooltip="Master switch for SBX processing. Disable it to bypass SBX effect controls below."
            onChange={(enabled) => onSetSbxMode(enabled ? "Enabled" : "Disabled")}
          />

          <EffectControl
            name="Surround"
            enabled={settings.surround_enabled === "Enabled"}
            value={settings.surround_value}
            tooltip="Expands stereo imaging into a wider virtual surround field."
            onChange={(enabled, value) => onSetEffect("surround", enabled ? "Enabled" : "Disabled", value)}
            disabled={settings.sbx_enabled === "Disabled"}
          />

          <EffectControl
            name="Crystalizer"
            enabled={settings.crystalizer_enabled === "Enabled"}
            value={settings.crystalizer_value}
            tooltip="Restores perceived detail and clarity in compressed or dull source audio."
            onChange={(enabled, value) => onSetEffect("crystalizer", enabled ? "Enabled" : "Disabled", value)}
            disabled={settings.sbx_enabled === "Disabled"}
          />

          <EffectControl
            name="Bass"
            enabled={settings.bass_enabled === "Enabled"}
            value={settings.bass_value}
            tooltip="Boosts low-end presence and punch for fuller bass response."
            onChange={(enabled, value) => onSetEffect("bass", enabled ? "Enabled" : "Disabled", value)}
            disabled={settings.sbx_enabled === "Disabled"}
          />

          <EffectControl
            name="Smart Volume"
            enabled={settings.smart_volume_enabled === "Enabled"}
            value={settings.smart_volume_value}
            tooltip="Smooths loudness swings so quieter and louder moments stay more consistent."
            onChange={(enabled, value) =>
              onSetEffect("smart_volume", enabled ? "Enabled" : "Disabled", value)
            }
            disabled={settings.sbx_enabled === "Disabled"}
          />

          <EffectControl
            name="Dialog Plus"
            enabled={settings.dialog_plus_enabled === "Enabled"}
            value={settings.dialog_plus_value}
            tooltip="Brings spoken dialogue forward so voices remain clear over background audio."
            onChange={(enabled, value) => onSetEffect("dialog_plus", enabled ? "Enabled" : "Disabled", value)}
            disabled={settings.sbx_enabled === "Disabled"}
          />
        </div>
      </div>
    </section>
  );
}
