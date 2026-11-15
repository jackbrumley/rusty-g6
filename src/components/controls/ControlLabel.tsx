import { Tooltip } from "../ui/Tooltip";

interface ControlLabelProps {
  label: string;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
}

/// The label of a settings row: the name (with an optional tooltip), and an
/// optional description that wraps onto its own line under the whole row.
export function ControlLabel({ label, tooltip, description }: ControlLabelProps) {
  return (
    <>
      <span class="control-label">{tooltip ? <Tooltip text={tooltip}>{label}</Tooltip> : label}</span>
      {description && <span class="control-description">{description}</span>}
    </>
  );
}
