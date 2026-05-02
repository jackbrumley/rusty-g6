interface StatusPageProps {
  connected: boolean;
  status: string;
  appVersion: string;
  permissionError: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onSetupPermissions: () => void;
}

export function StatusPage({
  connected,
  status,
  appVersion,
  permissionError,
  onConnect,
  onDisconnect,
  onSetupPermissions,
}: StatusPageProps) {
  return (
    <>
      <header class="status-header">
        <h1>Rusty G6</h1>
        <p class="subtitle">SoundBlaster X G6 Control Panel</p>
        <p class="version-text">v{appVersion || "1.0.x"}</p>
      </header>

      <section class="status-section">
        <div class="status-line">
          <span class={`status-indicator ${connected ? "connected" : "disconnected"}`}>
            {connected ? "●" : "○"}
          </span>
          <span class="status-text">{status}</span>
          {!connected ? (
            <button onClick={onConnect} class="btn-compact">
              Connect Device
            </button>
          ) : (
            <button onClick={onDisconnect} class="btn-compact btn-secondary">
              Disconnect
            </button>
          )}
          {permissionError && (
            <button
              onClick={onSetupPermissions}
              class="btn-compact btn-warning"
              title="Automatically set up Linux udev rules for the G6 device. Requires root password."
            >
              Fix Permissions
            </button>
          )}
        </div>
      </section>
    </>
  );
}
