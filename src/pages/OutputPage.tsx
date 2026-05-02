import { EffectControl } from "../components/controls/EffectControl";
import { ToggleControl } from "../components/controls/ToggleControl";
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
    <section class="output-section compact">
      <div class="section-line">
        <span class="section-label">Output:</span>
        <span class="section-value">{settings.output}</span>
        <button onClick={onToggleOutput} class="btn-compact">
          Toggle Output
        </button>
      </div>

      <div class="effects-list">
        <h3>Audio Effects</h3>

        <ToggleControl
          label="Scout Mode"
          checked={settings.scout_mode === "Enabled"}
          onChange={(enabled) => onSetScoutMode(enabled ? "Enabled" : "Disabled")}
        />

        <ToggleControl
          label="SBX Mode"
          checked={settings.sbx_enabled === "Enabled"}
          onChange={(enabled) => onSetSbxMode(enabled ? "Enabled" : "Disabled")}
        />

        <EffectControl
          name="Surround"
          enabled={settings.surround_enabled === "Enabled"}
          value={settings.surround_value}
          onChange={(enabled, value) => onSetEffect("surround", enabled ? "Enabled" : "Disabled", value)}
          disabled={settings.sbx_enabled === "Disabled"}
        />

        <EffectControl
          name="Crystalizer"
          enabled={settings.crystalizer_enabled === "Enabled"}
          value={settings.crystalizer_value}
          onChange={(enabled, value) => onSetEffect("crystalizer", enabled ? "Enabled" : "Disabled", value)}
          disabled={settings.sbx_enabled === "Disabled"}
        />

        <EffectControl
          name="Bass"
          enabled={settings.bass_enabled === "Enabled"}
          value={settings.bass_value}
          onChange={(enabled, value) => onSetEffect("bass", enabled ? "Enabled" : "Disabled", value)}
          disabled={settings.sbx_enabled === "Disabled"}
        />

        <EffectControl
          name="Smart Volume"
          enabled={settings.smart_volume_enabled === "Enabled"}
          value={settings.smart_volume_value}
          onChange={(enabled, value) =>
            onSetEffect("smart_volume", enabled ? "Enabled" : "Disabled", value)
          }
          disabled={settings.sbx_enabled === "Disabled"}
        />

        <EffectControl
          name="Dialog Plus"
          enabled={settings.dialog_plus_enabled === "Enabled"}
          value={settings.dialog_plus_value}
          onChange={(enabled, value) => onSetEffect("dialog_plus", enabled ? "Enabled" : "Disabled", value)}
          disabled={settings.sbx_enabled === "Disabled"}
        />
      </div>
    </section>
  );
}
