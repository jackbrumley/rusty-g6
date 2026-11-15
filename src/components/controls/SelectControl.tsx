import { Tooltip } from "../ui/Tooltip";

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
      <select
        class="select-control"
        value={value ?? ""}
        disabled={disabled}
        onChange={(event) => onChange((event.currentTarget as HTMLSelectElement).value as T)}
      >
        {value === null && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((option) => (
          <option key={option.id} value={option.id}>
            {option.name}
          </option>
        ))}
      </select>
    </div>
  );
}
