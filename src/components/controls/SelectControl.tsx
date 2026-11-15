import { ControlLabel } from "./ControlLabel";
import { Dropdown } from "./Dropdown";

interface SelectControlProps<T extends string> {
  label: string;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
  /// `null` shows `placeholder` (e.g. "Unknown") until a value is known.
  value: T | null;
  placeholder?: string;
  options: { id: T; name: string; description?: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
}

/// A settings row with a dropdown.
export function SelectControl<T extends string>({
  label,
  tooltip,
  description,
  value,
  placeholder = "Unknown",
  options,
  onChange,
  disabled = false,
}: SelectControlProps<T>) {
  return (
    <div class="control-row">
      <ControlLabel label={label} tooltip={tooltip} description={description} />
      <Dropdown
        value={value}
        placeholder={placeholder}
        options={options}
        onChange={onChange}
        disabled={disabled}
        ariaLabel={label}
      />
    </div>
  );
}
