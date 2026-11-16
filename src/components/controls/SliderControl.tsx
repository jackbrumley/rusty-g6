import { ControlLabel } from "./ControlLabel";
import { RangeSlider } from "./RangeSlider";

interface SliderControlProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  /// Called once when a drag or key press ends.
  onChange: (value: number) => void;
  /// Off while the value is unknown.
  disabled?: boolean;
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
  disabled = false,
  formatValue,
  tooltip,
  description,
}: SliderControlProps) {
  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <ControlLabel label={label} tooltip={tooltip} description={description} />
      <RangeSlider
        value={value}
        min={min}
        max={max}
        step={step}
        disabled={disabled}
        formatValue={(current) => (formatValue ? formatValue(current) : `${current}${unit}`)}
        onCommit={onChange}
      />
    </div>
  );
}
