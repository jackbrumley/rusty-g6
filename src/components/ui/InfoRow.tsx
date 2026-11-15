import type { ComponentChildren } from "preact";

/// A settings row that shows a value, with optional actions on the right.
export function InfoRow({
  label,
  value,
  muted = false,
  children,
}: {
  label: string;
  value?: string;
  /// Show the value as secondary text.
  muted?: boolean;
  children?: ComponentChildren;
}) {
  return (
    <div class="control-row">
      <span class="control-label">{label}</span>
      {value !== undefined && <span class={`control-value ${muted ? "control-value-muted" : ""}`}>{value}</span>}
      {children && <span class="control-actions">{children}</span>}
    </div>
  );
}
