import { ControlLabel } from "./ControlLabel";
import { type ToggleRequest, useRequestedToggle } from "./useRequestedToggle";

interface ToggleControlProps {
  label: string;
  checked: boolean;
  onChange: ToggleRequest;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
}

export function ToggleControl({ label, checked, onChange, tooltip, description }: ToggleControlProps) {
  const { shown, toggle } = useRequestedToggle(checked, onChange);

  return (
    <div class="control-row control-row-toggle">
      <ControlLabel label={label} tooltip={tooltip} description={description} />
      <span class="slider-value">{shown ? "On" : "Off"}</span>
      <label class="toggle-switch">
        <input
          type="checkbox"
          checked={shown}
          onChange={(e) => toggle((e.currentTarget as HTMLInputElement).checked)}
        />
        <span class="toggle-slider"></span>
      </label>
    </div>
  );
}
