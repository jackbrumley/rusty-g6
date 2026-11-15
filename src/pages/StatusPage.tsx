import type { G6Settings } from "../types/g6";
import { Tooltip } from "../components/ui/Tooltip";

interface StatusPageProps {
  connected: boolean;
  status: string;
  appVersion: string;
  permissionError: boolean;
  settings: G6Settings | null;
  onReadDeviceState: () => void;
  onSetupPermissions: () => void;
  showUpdatePill: boolean;
  onOpenUpdateModal: () => void;
}

const onOff = (enabled: boolean | undefined) =>
  enabled === undefined ? "Unknown" : enabled ? "On" : "Off";

export function StatusPage({
  connected,
  status,
  appVersion,
  permissionError,
  settings,
  onReadDeviceState,
  onSetupPermissions,
  showUpdatePill,
  onOpenUpdateModal,
}: StatusPageProps) {
  return (
    <>
      <header class="status-header">
        <h1>Rusty G6</h1>
        <p class="subtitle">SoundBlaster X G6 Control Panel</p>
        <p class="version-text">v{appVersion || "1.0.x"}</p>
        {showUpdatePill && (
          <button class="update-pill" onClick={onOpenUpdateModal}>
            <span class="update-pill-icon" aria-hidden="true">
              !
            </span>
            <span>Update available</span>
          </button>
        )}
      </header>

      <div class="status-line status-line-centered">
        <span class={`status-indicator ${connected ? "connected" : "disconnected"}`}>
          {connected ? "●" : "○"}
        </span>
        <span class="status-text">{status}</span>
        {permissionError && (
          <Tooltip text="Automatically sets up Linux USB permissions for the G6. Root authentication is required.">
            <button onClick={onSetupPermissions} class="btn-compact btn-warning">
              Fix Permissions
            </button>
          </Tooltip>
        )}
      </div>

      <dl class="status-details">
        <div class="status-detail">
          <dt>Firmware</dt>
          <dd>{settings?.firmware_version ?? "Unknown"}</dd>
        </div>
        <div class="status-detail">
          <dt>Output</dt>
          <dd>{settings?.output ?? "Unknown"}</dd>
        </div>
        <div class="status-detail">
          <dt>SBX</dt>
          <dd>{onOff(settings?.sbx_enabled)}</dd>
        </div>
        <div class="status-detail">
          <dt>Scout Mode</dt>
          <dd>{onOff(settings?.scout_mode_enabled)}</dd>
        </div>
        <div class="status-detail">
          <dt>Last read</dt>
          <dd>
            {settings?.last_read_time
              ? new Date(settings.last_read_time * 1000).toLocaleTimeString()
              : "Not yet"}
          </dd>
        </div>
      </dl>

      {connected && (
        <div class="status-actions">
          <button onClick={onReadDeviceState} class="btn-compact btn-secondary">
            Read Device State
          </button>
        </div>
      )}
    </>
  );
}
