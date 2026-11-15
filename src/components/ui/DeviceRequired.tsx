import type { ComponentChildren } from "preact";

interface DeviceRequiredProps {
  connected: boolean;
  onGoToStatus: () => void;
  children: ComponentChildren;
}

/// Shows `children` while the G6 is connected, and a pointer to the Status
/// page otherwise.
export function DeviceRequired({ connected, onGoToStatus, children }: DeviceRequiredProps) {
  if (connected) {
    return <>{children}</>;
  }
  return (
    <div class="info-panel">
      <p>Connect your Sound BlasterX G6 to use these controls.</p>
      <button class="btn-compact" onClick={onGoToStatus}>
        Go to Status
      </button>
    </div>
  );
}
