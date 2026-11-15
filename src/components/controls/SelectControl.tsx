import { Tooltip } from "../ui/Tooltip";
import { Dropdown } from "./Dropdown";

interface SelectControlProps<T extends string> {
  label: string;
  tooltip?: string;
  /// `null` shows `placeholder` (e.g. "Unknown") until a value is known.
  value: T | null;
  placeholder?: string;
  options: { id: T; name: string }[];
  onChange: (value: T) => void;
  disabled?: boolean;
}

/// A settings row with a dropdown.
export function SelectControl<T extends string>({
  label,
  tooltip,
  value,
  placeholder = "Unknown",
  options,
  onChange,
  disabled = false,
}: SelectControlProps<T>) {
  return (
    <div class="control-row">
      <span class="control-label">{tooltip ? <Tooltip text={tooltip}>{label}</Tooltip> : label}</span>
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
