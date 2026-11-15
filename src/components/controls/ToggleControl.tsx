import { ControlLabel } from "./ControlLabel";

interface ToggleControlProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
}

export function ToggleControl({ label, checked, onChange, tooltip, description }: ToggleControlProps) {
  const handleToggle = (e: Event) => {
    const newChecked = (e.currentTarget as HTMLInputElement).checked;
    onChange(newChecked);
  };

  return (
    <div class="control-row control-row-toggle">
      <ControlLabel label={label} tooltip={tooltip} description={description} />
      <span class="slider-value">{checked ? "On" : "Off"}</span>
      <label class="toggle-switch">
        <input type="checkbox" checked={checked} onChange={handleToggle} />
        <span class="toggle-slider"></span>
      </label>
    </div>
  );
}
