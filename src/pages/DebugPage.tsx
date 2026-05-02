import { CollapsibleCardSection } from "../components/ui/CollapsibleCardSection";
import { ToggleControl } from "../components/controls/ToggleControl";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import { Tooltip } from "../components/ui/Tooltip";
import type { G6Settings } from "../types/g6";
import { useState } from "preact/hooks";

interface DebugPageProps {
  settings: G6Settings | null;
  appVersion: string;
  autostartEnabled: boolean;
  showExperimental: boolean;
  checkingUpdates: boolean;
  updateAvailable: boolean;
  latestVersion: string | null;
  lastCheckedLabel: string;
  logSeparatorMessage: string;
  onNavigateUiLab: () => void;
  onReadDeviceState: () => void;
  onResetConnection: () => void;
  onToggleAutostart: (enabled: boolean) => void;
  onToggleExperimental: (enabled: boolean) => void;
  onCheckForUpdates: () => void;
  onOpenUpdateModal: () => void;
  onSetLogSeparatorMessage: (value: string) => void;
  onClearTerminal: () => void;
  onCopySessionLog: () => void;
  onOpenSessionLog: () => void;
}

export function DebugPage({
  settings,
  appVersion,
  autostartEnabled,
  showExperimental,
  checkingUpdates,
  updateAvailable,
  latestVersion,
  lastCheckedLabel,
  logSeparatorMessage,
  onNavigateUiLab,
  onReadDeviceState,
  onResetConnection,
  onToggleAutostart,
  onToggleExperimental,
  onCheckForUpdates,
  onOpenUpdateModal,
  onSetLogSeparatorMessage,
  onClearTerminal,
  onCopySessionLog,
  onOpenSessionLog,
}: DebugPageProps) {
  const [activeSection, setActiveSection] = useState<string | null>(null);
  const toggleSection = (section: string) => {
    setActiveSection((current) => (current === section ? null : section));
  };

  return (
    <>
      <TabPageHeader
        title="Settings"
        subtitle="Application controls and update preferences, with advanced debug tools grouped separately below."
      />

      <div class="settings-accordion">
      <CollapsibleCardSection
        title="General Settings"
        isOpen={activeSection === "general"}
        onToggle={() => toggleSection("general")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="debug-toggle-grid">
              <ToggleControl
                label="Auto-start with system"
                checked={autostartEnabled}
                onChange={onToggleAutostart}
              />
            </div>
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="Updates"
        isOpen={activeSection === "updates"}
        onToggle={() => toggleSection("updates")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="device-details">
              <div class="read-only-item">
                <span class="readonly-label">Current Version:</span>
                <span class="readonly-value">v{appVersion || "1.0.x"}</span>
              </div>
              <div class="read-only-item">
                <span class="readonly-label">Latest Version:</span>
                <span class="readonly-value">{latestVersion ? `v${latestVersion}` : "Unknown"}</span>
              </div>
              <div class="read-only-item">
                <span class="readonly-label">Last Checked:</span>
                <span class="readonly-value">{lastCheckedLabel}</span>
              </div>
            </div>
            <div class="debug-log-actions">
              <button onClick={onCheckForUpdates} class="btn-compact btn-secondary" disabled={checkingUpdates}>
                {checkingUpdates ? "Checking..." : "Check for Updates"}
              </button>
              {updateAvailable && (
                <button onClick={onOpenUpdateModal} class="btn-compact btn-secondary">
                  View Update
                </button>
              )}
            </div>
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="Logging"
        isOpen={activeSection === "logging"}
        onToggle={() => toggleSection("logging")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
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
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="Device Debug"
        isOpen={activeSection === "device-debug"}
        onToggle={() => toggleSection("device-debug")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="debug-action-grid">
              <button onClick={onReadDeviceState} class="btn-compact btn-secondary">
                Read Device State
              </button>
              <button onClick={onResetConnection} class="btn-compact btn-secondary">
                Reset Connection
              </button>
            </div>
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
          </div>
        </div>
      </CollapsibleCardSection>

      <CollapsibleCardSection
        title="UI Debug"
        isOpen={activeSection === "ui-debug"}
        onToggle={() => toggleSection("ui-debug")}
      >
        <div class="debug-log-tools">
          <div class="debug-log-card">
            <div class="debug-action-grid">
              <button onClick={onNavigateUiLab} class="btn-compact btn-secondary">
                Open UI Lab
              </button>
            </div>
            <div class="debug-toggle-grid">
              <ToggleControl
                label="Experimental Features"
                checked={showExperimental}
                onChange={onToggleExperimental}
              />
            </div>
          </div>
        </div>
      </CollapsibleCardSection>
      </div>
    </>
  );
}
