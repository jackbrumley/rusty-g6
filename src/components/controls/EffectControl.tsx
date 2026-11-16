import { ControlLabel } from "./ControlLabel";
import { RangeSlider } from "./RangeSlider";
import { type ToggleRequest, useRequestedToggle } from "./useRequestedToggle";

interface EffectControlProps {
  name: string;
  enabled: boolean;
  level: number;
  onToggle: ToggleRequest;
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
  const { shown, toggle } = useRequestedToggle(enabled, onToggle);
  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <ControlLabel label={name} tooltip={tooltip} description={description} />
      <label class="toggle-switch">
        <input
          type="checkbox"
          checked={shown}
          onChange={(e) => toggle((e.currentTarget as HTMLInputElement).checked)}
          disabled={disabled}
        />
        <span class="toggle-slider"></span>
      </label>
      <RangeSlider
        value={level}
        min={0}
        max={100}
        step={1}
        disabled={!shown || disabled}
        formatValue={(current) => String(current)}
        onCommit={onLevelChange}
      />
    </div>
  );
}
