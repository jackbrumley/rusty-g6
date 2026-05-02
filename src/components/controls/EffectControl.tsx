import { useEffect, useState } from "preact/hooks";
import { Tooltip } from "../ui/Tooltip";

interface EffectControlProps {
  name: string;
  enabled: boolean;
  value: number;
  onChange: (enabled: boolean, value: number) => void;
  disabled?: boolean;
  tooltip?: string;
}

export function EffectControl({
  name,
  enabled,
  value,
  onChange,
  disabled = false,
  tooltip,
}: EffectControlProps) {
  const [localValue, setLocalValue] = useState(value);
  const [localEnabled, setLocalEnabled] = useState(enabled);

  useEffect(() => {
    setLocalValue(value);
    setLocalEnabled(enabled);
  }, [value, enabled]);

  const handleToggle = (e: Event) => {
    const newEnabled = (e.currentTarget as HTMLInputElement).checked;
    setLocalEnabled(newEnabled);
    onChange(newEnabled, localValue);
  };

  const handleSliderChange = (e: Event) => {
    const newValue = parseInt((e.currentTarget as HTMLInputElement).value);
    setLocalValue(newValue);
    if (localEnabled) {
      onChange(localEnabled, newValue);
    }
  };

  const handleSliderInput = (e: Event) => {
    setLocalValue(parseInt((e.currentTarget as HTMLInputElement).value));
  };

  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <span class="control-label">
        {tooltip ? <Tooltip text={tooltip}>{name}</Tooltip> : name}
      </span>
      <label class="toggle-switch">
        <input
          type="checkbox"
          checked={localEnabled}
          onChange={handleToggle}
          disabled={disabled}
        />
        <span class="toggle-slider"></span>
      </label>
      <input
        type="range"
        min="0"
        max="100"
        value={localValue}
        onInput={handleSliderInput}
        onChange={handleSliderChange}
        disabled={!localEnabled || disabled}
        class="slider"
      />
      <span class="slider-value">{localValue}</span>
    </div>
  );
}
