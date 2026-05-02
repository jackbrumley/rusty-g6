import { ToggleControl } from "../components/controls/ToggleControl";
import type { G6Settings } from "../types/g6";

interface DebugPageProps {
  appVersion: string;
  settings: G6Settings | null;
  autostartEnabled: boolean;
  showExperimental: boolean;
  logSeparatorMessage: string;
  onNavigateUiLab: () => void;
  onReadDeviceState: () => void;
  onToggleAutostart: (enabled: boolean) => void;
  onToggleExperimental: (enabled: boolean) => void;
  onSetLogSeparatorMessage: (value: string) => void;
  onClearTerminal: () => void;
}

export function DebugPage({
  appVersion,
  settings,
  autostartEnabled,
  showExperimental,
  logSeparatorMessage,
  onNavigateUiLab,
  onReadDeviceState,
  onToggleAutostart,
  onToggleExperimental,
  onSetLogSeparatorMessage,
  onClearTerminal,
}: DebugPageProps) {
  return (
    <section class="debug-section compact">
      <div class="read-only-item">
        <span class="readonly-label">App Version:</span>
        <span class="readonly-value">{appVersion || "Unknown"}</span>
      </div>

      <div class="read-only-item">
        <span class="readonly-label">Firmware:</span>
        <span class="readonly-value">{settings?.firmware_info ? settings.firmware_info.version : "Unknown"}</span>
      </div>

      {(settings?.equalizer || settings?.extended_params) && (
        <div class="device-details">
          {settings.equalizer && (
            <div class="read-only-item">
              <span class="readonly-label">Equalizer:</span>
              <span class="readonly-value">
                {settings.equalizer.enabled} • {settings.equalizer.bands.length} bands (Read-only)
              </span>
            </div>
          )}

          {settings.extended_params && (
            <div class="read-only-item">
              <span class="readonly-label">Extended Params:</span>
              <span class="readonly-value">
                {Object.values(settings.extended_params).filter((v) => v !== null).length}/15 detected
                (Read-only)
              </span>
            </div>
          )}

          {settings.last_read_time && (
            <div class="read-only-item">
              <span class="readonly-label">Last Read:</span>
              <span class="readonly-value">
                {new Date(settings.last_read_time * 1000).toLocaleTimeString()}
              </span>
            </div>
          )}
        </div>
      )}

      <div class="debug-controls">
        <button onClick={onNavigateUiLab} class="btn-compact btn-secondary">
          Open UI Lab
        </button>
        <button onClick={onReadDeviceState} class="btn-compact btn-secondary">
          Read Device State
        </button>
        <ToggleControl
          label="Auto-start with system"
          checked={autostartEnabled}
          onChange={onToggleAutostart}
        />
        <ToggleControl
          label="Experimental Features"
          checked={showExperimental}
          onChange={onToggleExperimental}
        />
        <input
          type="text"
          class="log-message-input"
          placeholder="Optional: Add a note to the log separator..."
          value={logSeparatorMessage}
          onInput={(e) => onSetLogSeparatorMessage(e.currentTarget.value)}
          onKeyPress={(e) => {
            if (e.key === "Enter") {
              onClearTerminal();
            }
          }}
        />
        <button
          onClick={onClearTerminal}
          class="btn-compact btn-full-width"
          title="Add a visual separator marker in the terminal logs with optional message"
        >
          Add Log Separator
        </button>
      </div>
    </section>
  );
}
