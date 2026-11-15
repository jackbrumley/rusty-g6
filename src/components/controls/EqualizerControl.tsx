import { EQ_GAIN_STEP_DB, EQ_PREAMP_LIMIT_DB, type EqualizerSettings } from "../../types/g6";
import { EqualizerGraph } from "./EqualizerGraph";
import { EQ_PRESETS, matchingPreset } from "./eqPresets";
import { SettingsPage } from "../ui/SettingsPage";
import { SettingsSection } from "../ui/SettingsSection";
import { SelectControl } from "./SelectControl";
import { SliderControl } from "./SliderControl";
import { ToggleControl } from "./ToggleControl";

interface EqualizerControlProps {
  equalizer: EqualizerSettings;
  onSetEnabled: (enabled: boolean) => void;
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
  const enabled = equalizer.enabled;
  const preset = matchingPreset(equalizer.band_gains_db);

  const choosePreset = (id: string) => {
    const chosen = EQ_PRESETS.find((candidate) => candidate.id === id);
    if (chosen) {
      onSetBands(chosen.gainsDb);
    }
  };

  return (
    <SettingsPage>
      <SettingsSection title="Equalizer">
        <ToggleControl
          label="Equalizer"
          checked={enabled}
          onChange={onSetEnabled}
        />
        <SelectControl
          label="Preset"
          value={preset?.id ?? null}
          placeholder="Custom"
          options={EQ_PRESETS.map(({ id, name }) => ({ id, name }))}
          onChange={choosePreset}
        />
        <SliderControl
          label="Preamp"
          value={equalizer.preamp_db}
          min={-EQ_PREAMP_LIMIT_DB}
          max={EQ_PREAMP_LIMIT_DB}
          step={EQ_GAIN_STEP_DB}
          formatValue={formatDb}
          description="Turn this down if boosted bands distort."
          onChange={onSetPreamp}
        />
      </SettingsSection>

      <SettingsSection title="Curve" wide>
        <EqualizerGraph gainsDb={equalizer.band_gains_db} dimmed={!enabled} onSetBand={onSetBand} />
        <p class="info-note eq-hint">Drag a point to change that band. Double-click a point to reset it.</p>
      </SettingsSection>
    </SettingsPage>
  );
}
