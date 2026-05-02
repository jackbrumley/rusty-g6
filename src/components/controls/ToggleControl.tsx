import { Tooltip } from "../ui/Tooltip";

interface ToggleControlProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  tooltip?: string;
}

export function ToggleControl({ label, checked, onChange, tooltip }: ToggleControlProps) {
  const handleToggle = (e: Event) => {
    const newChecked = (e.currentTarget as HTMLInputElement).checked;
    onChange(newChecked);
  };

  return (
    <div class="control-row control-row-toggle">
      <span class="control-label">
        {tooltip ? <Tooltip text={tooltip}>{label}</Tooltip> : label}
      </span>
      <span class="slider-value">{checked ? "On" : "Off"}</span>
      <label class="toggle-switch">
        <input type="checkbox" checked={checked} onChange={handleToggle} />
        <span class="toggle-slider"></span>
      </label>
    </div>
  );
}
