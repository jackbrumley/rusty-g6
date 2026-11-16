import type { ComponentChildren } from "preact";
import type { G6Settings } from "../../types/g6";

interface DeviceRequiredProps {
  settings: G6Settings | null;
  onGoToStatus: () => void;
  children: ComponentChildren;
}

/// Shows `children` only once the G6 has answered a full state read, so a
/// control never shows a guessed value: the device is the master.
export function DeviceRequired({ settings, onGoToStatus, children }: DeviceRequiredProps) {
  if (settings === null) {
    return (
      <div class="info-panel">
        <p>Connect your Sound BlasterX G6 to use these controls.</p>
        <button class="btn-compact" onClick={onGoToStatus}>
          Go to Status
        </button>
      </div>
    );
  }
  if (settings.last_read_time === null) {
    return (
      <div class="info-panel">
        <p>Reading the G6's settings...</p>
      </div>
    );
  }
  return <>{children}</>;
}
