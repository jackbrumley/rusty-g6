import type { G6Settings } from "../types/g6";
import { Tooltip } from "../components/ui/Tooltip";

interface StatusPageProps {
  connected: boolean;
  status: string;
  appVersion: string;
  permissionError: boolean;
  settings: G6Settings | null;
  isLinux: boolean;
  retryInSeconds: number | null;
  onReadDeviceState: () => void;
  onSetupPermissions: () => void;
  showUpdatePill: boolean;
  onOpenUpdateModal: () => void;
}

export function StatusPage({
  connected,
  status,
  appVersion,
  permissionError,
  settings,
  isLinux,
  retryInSeconds,
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

      <section class="status-section surface-card">
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
        {!connected && !permissionError && (
          <p class="info-note" style={{ marginTop: "8px" }}>
            Auto-connect is enabled. Rusty G6 will keep searching for your device.
            {retryInSeconds !== null ? ` Next retry in ${retryInSeconds}s.` : ""}
            If this persists, try unplugging and reconnecting the G6 USB cable.
          </p>
        )}
      </section>

      <section class="debug-section compact surface-card">
        <div class="status-info-grid">
          <div class="status-info-tile">
            <span class="status-info-label">Firmware</span>
            <span class="status-info-value">
              {settings?.firmware_info
                ? `${settings.firmware_info.version}${settings.firmware_info.build ? ` (${settings.firmware_info.build})` : ""}`
                : "Unknown"}
            </span>
          </div>
          <div class="status-info-tile">
            <span class="status-info-label">Output</span>
            <span class="status-info-value">{settings?.output ?? "Unknown"}</span>
          </div>
          <div class="status-info-tile">
            <span class="status-info-label">SBX Mode</span>
            <span class="status-info-value">{settings?.sbx_enabled ?? "Unknown"}</span>
          </div>
          <div class="status-info-tile">
            <span class="status-info-label">Scout Mode</span>
            <span class="status-info-value">{settings?.scout_mode ?? "Unknown"}</span>
          </div>
          <div class="status-info-tile">
            <span class="status-info-label">Platform</span>
            <span class="status-info-value">{isLinux ? "Linux" : "Windows"}</span>
          </div>
          <div class="status-info-tile">
            <span class="status-info-label">Last Read</span>
            <span class="status-info-value">
              {settings?.last_read_time
                ? new Date(settings.last_read_time * 1000).toLocaleTimeString()
                : "Not yet"}
            </span>
          </div>
        </div>
        {connected && (
          <div class="status-action-grid">
            <button onClick={onReadDeviceState} class="btn-compact btn-secondary">
              Read Device State
            </button>
          </div>
        )}
      </section>
    </>
  );
}
