import { ControlLabel } from "./ControlLabel";
import { RangeSlider } from "./RangeSlider";

interface EffectControlProps {
  name: string;
  enabled: boolean;
  level: number;
  onToggle: (enabled: boolean) => void;
  onLevelChange: (level: number) => void;
  disabled?: boolean;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
}

export function EffectControl({
  name,
  enabled,
  level,
  onToggle,
  onLevelChange,
  disabled = false,
  tooltip,
  description,
}: EffectControlProps) {
  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <ControlLabel label={name} tooltip={tooltip} description={description} />
      <label class="toggle-switch">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => onToggle((e.currentTarget as HTMLInputElement).checked)}
          disabled={disabled}
        />
        <span class="toggle-slider"></span>
      </label>
      <RangeSlider
        value={level}
        min={0}
        max={100}
        step={1}
        disabled={!enabled || disabled}
        formatValue={(current) => String(current)}
        onCommit={onLevelChange}
      />
    </div>
  );
}
