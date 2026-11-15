import { ControlLabel } from "./ControlLabel";
import { useEffect, useState } from "preact/hooks";

interface SliderControlProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
  formatValue?: (value: number) => string;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
}

export function SliderControl({
  label,
  value,
  min = 0,
  max = 100,
  step = 1,
  unit = "",
  onChange,
  formatValue,
  tooltip,
  description,
}: SliderControlProps) {
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  const handleSliderChange = (e: Event) => {
    const newValue = parseFloat((e.currentTarget as HTMLInputElement).value);
    setLocalValue(newValue);
    onChange(newValue);
  };

  const handleSliderInput = (e: Event) => {
    setLocalValue(parseFloat((e.currentTarget as HTMLInputElement).value));
  };

  const displayValue = formatValue ? formatValue(localValue) : `${localValue}${unit}`;

  return (
    <div class="control-row">
      <ControlLabel label={label} tooltip={tooltip} description={description} />
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={localValue}
        onInput={handleSliderInput}
        onChange={handleSliderChange}
        class="slider"
      />
      <span class="slider-value">{displayValue}</span>
    </div>
  );
}
