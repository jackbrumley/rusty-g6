import { EQ_GAIN_STEP_DB, EQ_PREAMP_LIMIT_DB, type EqualizerSettings } from "../../types/g6";
import { EqualizerGraph } from "./EqualizerGraph";
import { EQ_PRESETS, matchingPreset } from "./eqPresets";
import { SliderControl } from "./SliderControl";
import { ToggleControl } from "./ToggleControl";

interface EqualizerControlProps {
  equalizer: EqualizerSettings;
  onSetEnabled: (enabled: "Enabled" | "Disabled") => void;
  onSetPreamp: (gainDb: number) => void;
  onSetBand: (band: number, gainDb: number) => void;
  onSetBands: (gainsDb: number[]) => void;
}

const formatDb = (gainDb: number) => `${gainDb > 0 ? "+" : ""}${gainDb.toFixed(1)} dB`;

/// The G6's 10-band graphic EQ for the active output: on/off, a preset
/// picker, a draggable curve and the preamp.
export function EqualizerControl({
  equalizer,
  onSetEnabled,
  onSetPreamp,
  onSetBand,
  onSetBands,
}: EqualizerControlProps) {
  const enabled = equalizer.enabled === "Enabled";
  const preset = matchingPreset(equalizer.band_gains_db);

  const handlePresetChange = (event: Event) => {
    const chosen = EQ_PRESETS.find(
      (candidate) => candidate.id === (event.currentTarget as HTMLSelectElement).value
    );
    if (chosen) {
      onSetBands(chosen.gainsDb);
    }
  };

  return (
    <div class="effects-list">
      <h3>Equalizer</h3>
      <div class="output-controls-grid">
        <ToggleControl
          label="Equalizer"
          checked={enabled}
          tooltip="10-band graphic EQ, stored on the G6 for the current output."
          onChange={(checked) => onSetEnabled(checked ? "Enabled" : "Disabled")}
        />

        <div class="control-row">
          <span class="control-label">Preset</span>
          <select class="eq-preset-select" value={preset?.id ?? "custom"} onChange={handlePresetChange}>
            {!preset && (
              <option value="custom" disabled>
                Custom
              </option>
            )}
            {EQ_PRESETS.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>
                {candidate.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      <EqualizerGraph gainsDb={equalizer.band_gains_db} dimmed={!enabled} onSetBand={onSetBand} />
      <p class="info-note">Drag a point to change that band. Double-click a point to reset it.</p>

      <div class="output-controls-grid">
        <SliderControl
          label="Preamp"
          value={equalizer.preamp_db}
          min={-EQ_PREAMP_LIMIT_DB}
          max={EQ_PREAMP_LIMIT_DB}
          step={EQ_GAIN_STEP_DB}
          formatValue={formatDb}
          tooltip="Lowers or raises the level before the EQ, to avoid clipping when bands are boosted."
          onChange={onSetPreamp}
        />
      </div>
    </div>
  );
}
