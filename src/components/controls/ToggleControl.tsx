interface ToggleControlProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function ToggleControl({ label, checked, onChange }: ToggleControlProps) {
  const handleToggle = (e: Event) => {
    const newChecked = (e.currentTarget as HTMLInputElement).checked;
    onChange(newChecked);
  };

  return (
    <div class="control-row">
      <span class="control-label">{label}</span>
      <label class="toggle-switch">
        <input type="checkbox" checked={checked} onChange={handleToggle} />
        <span class="toggle-slider"></span>
      </label>
      <span class="slider-value">{checked ? "On" : "Off"}</span>
    </div>
  );
}
