import { ToggleControl } from "../components/controls/ToggleControl";
import { Tooltip } from "../components/ui/Tooltip";
import type { G6Settings } from "../types/g6";

interface DebugPageProps {
  settings: G6Settings | null;
  autostartEnabled: boolean;
  showExperimental: boolean;
  logSeparatorMessage: string;
  onNavigateUiLab: () => void;
  onReadDeviceState: () => void;
  onResetConnection: () => void;
  onToggleAutostart: (enabled: boolean) => void;
  onToggleExperimental: (enabled: boolean) => void;
  onSetLogSeparatorMessage: (value: string) => void;
  onClearTerminal: () => void;
  onCopySessionLog: () => void;
  onOpenSessionLog: () => void;
}

export function DebugPage({
  settings,
  autostartEnabled,
  showExperimental,
  logSeparatorMessage,
  onNavigateUiLab,
  onReadDeviceState,
  onResetConnection,
  onToggleAutostart,
  onToggleExperimental,
  onSetLogSeparatorMessage,
  onClearTerminal,
  onCopySessionLog,
  onOpenSessionLog,
}: DebugPageProps) {
  return (
    <>
      <section class="debug-section compact surface-card">
        {settings?.extended_params && (
          <div class="device-details">
            <div class="read-only-item">
              <span class="readonly-label">Extended Params:</span>
              <span class="readonly-value">
                {Object.values(settings.extended_params).filter((v) => v !== null).length}/15 detected
                (Read-only)
              </span>
            </div>
          </div>
        )}

        <div class="debug-controls">
          <div class="debug-action-grid">
            <button onClick={onNavigateUiLab} class="btn-compact btn-secondary">
              Open UI Lab
            </button>
            <button onClick={onReadDeviceState} class="btn-compact btn-secondary">
              Read Device State
            </button>
            <button onClick={onResetConnection} class="btn-compact btn-secondary">
              Reset Connection
            </button>
          </div>

          <div class="debug-toggle-grid">
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
          </div>
        </div>
      </section>

      <section class="debug-section compact surface-card">
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <h3>Logging</h3>
            <div class="debug-log-row">
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
              <Tooltip text="Inserts a visible separator marker in terminal logs using your optional note.">
                <button onClick={onClearTerminal} class="btn-compact btn-secondary debug-log-button">
                  Add Log Separator
                </button>
              </Tooltip>
            </div>
            <div class="debug-log-actions">
              <button onClick={onCopySessionLog} class="btn-compact btn-secondary">
                Copy Session Log
              </button>
              <button onClick={onOpenSessionLog} class="btn-compact btn-secondary">
                Open Log File
              </button>
            </div>
          </div>
        </div>
      </section>
    </>
  );
}
