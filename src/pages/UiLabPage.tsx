import { useState } from "preact/hooks";
import { EffectControl } from "../components/controls/EffectControl";
import { SliderControl } from "../components/controls/SliderControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import type { ToastType } from "../types/g6";

interface UiLabPageProps {
  onBack: () => void;
  onToast: (type: ToastType, message: string, durationMs?: number) => void;
}

export function UiLabPage({ onBack, onToast }: UiLabPageProps) {
  const [previewToggle, setPreviewToggle] = useState(true);
  const [previewValue, setPreviewValue] = useState(42);
  const [previewEnabled, setPreviewEnabled] = useState(true);
  const [previewConnected, setPreviewConnected] = useState(true);
  const [previewPermissionError, setPreviewPermissionError] = useState(false);
  const [previewSbxEnabled, setPreviewSbxEnabled] = useState(true);
  const [showUpdatePill, setShowUpdatePill] = useState(true);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateState, setUpdateState] = useState<"available" | "uptodate" | "error">("available");

  const updateTitle =
    updateState === "available"
      ? "Update Available"
      : updateState === "uptodate"
        ? "Rusty G6 is Up to Date"
        : "Update Check Failed";

  const updateMessage =
    updateState === "available"
      ? "A newer Rusty G6 version is available. Current: v1.0.4 -> Latest: v1.0.5."
      : updateState === "uptodate"
        ? "You are already running the latest version (v1.0.4)."
        : "Unable to contact update endpoint. Please try again in a few minutes.";

  return (
    <section class="debug-section compact">
      <div class="ui-lab-header-row">
        <span class="section-label">UI Lab:</span>
        <button onClick={onBack} class="btn-compact btn-secondary">
          Back to Debug
        </button>
      </div>

      <p class="info-note" style={{ marginTop: "8px" }}>
        Hidden route for visual tuning. Open directly with <code>#/ui-lab</code>.
      </p>

      <div class="ui-lab-section ui-lab-checklist">
        <h3>Release QA Order</h3>
        <ol>
          <li>Check titlebar and tab shell spacing/alignment.</li>
          <li>Validate Status states: connected, disconnected, and permission warning.</li>
          <li>Review control interactions: toggles, sliders, and disabled SBX-dependent controls.</li>
          <li>Review update pill and open each update modal state variant.</li>
          <li>Trigger success, info, and error toasts for final visual pass.</li>
        </ol>
      </div>

      <div class="ui-lab-section">
        <h3>Shell Preview</h3>
        <div class="ui-lab-shell-preview">
          <div class="title-bar" style={{ cursor: "default" }}>
            <div class="title-bar-title">Rusty G6</div>
            <div class="title-bar-controls">
              <button class="title-bar-button">
                <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M5 12h14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                </svg>
              </button>
              <button class="title-bar-button">
                <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <rect x="5" y="5" width="14" height="14" rx="1.5" ry="1.5" fill="none" stroke="currentColor" strokeWidth="2" />
                </svg>
              </button>
              <button class="title-bar-button close">
                <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M7 7l10 10" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                  <path d="M17 7L7 17" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                </svg>
              </button>
            </div>
          </div>
          <div class="tab-nav-shell">
            <nav class="tab-nav">
              <button class="tab-button">Status</button>
              <button class="tab-button active">Output</button>
              <button class="tab-button">Input</button>
              <button class="tab-button">Debug</button>
            </nav>
          </div>
        </div>
      </div>

      <div class="ui-lab-section">
        <h3>Status Elements</h3>
        <div class="ui-lab-controls-inline">
          <button class="btn-compact btn-secondary" onClick={() => setPreviewConnected((v) => !v)}>
            {previewConnected ? "Show Disconnected" : "Show Connected"}
          </button>
          <button
            class="btn-compact btn-secondary"
            onClick={() => setPreviewPermissionError((v) => !v)}
          >
            Toggle Permission Warning
          </button>
        </div>
        <section class="status-section">
          <div class="status-line">
            <span class={`status-indicator ${previewConnected ? "connected" : "disconnected"}`}>
              {previewConnected ? "●" : "○"}
            </span>
            <span class="status-text">{previewConnected ? "Connected" : "Disconnected"}</span>
            <button class={`btn-compact ${previewConnected ? "btn-secondary" : ""}`}>
              {previewConnected ? "Disconnect" : "Connect Device"}
            </button>
            {previewPermissionError && <button class="btn-compact btn-warning">Fix Permissions</button>}
          </div>
        </section>
      </div>

      <div class="ui-lab-section effects-list">
        <h3>Control Previews</h3>
        <div class="ui-lab-controls-inline">
          <button class="btn-compact btn-secondary" onClick={() => setPreviewSbxEnabled((v) => !v)}>
            SBX: {previewSbxEnabled ? "Enabled" : "Disabled"}
          </button>
          <button class="btn-compact" onClick={() => setPreviewValue(64)}>
            Set Slider 64
          </button>
        </div>

        <ToggleControl label="Toggle Preview" checked={previewToggle} onChange={setPreviewToggle} />

        <SliderControl
          label="Slider Preview"
          value={previewValue}
          min={0}
          max={100}
          onChange={setPreviewValue}
        />

        <EffectControl
          name="Effect Preview"
          enabled={previewEnabled}
          value={previewValue}
          disabled={!previewSbxEnabled}
          onChange={(enabled, value) => {
            setPreviewEnabled(enabled);
            setPreviewValue(value);
          }}
        />
      </div>

      <div class="ui-lab-section effects-list">
        <h3>Update UX Preview</h3>
        <div class="ui-lab-controls-inline">
          <button class="btn-compact btn-secondary" onClick={() => setShowUpdatePill((v) => !v)}>
            {showUpdatePill ? "Hide" : "Show"} Update Pill
          </button>
          <button class="btn-compact btn-secondary" onClick={() => setUpdateState("available")}>
            Available
          </button>
          <button class="btn-compact btn-secondary" onClick={() => setUpdateState("uptodate")}>
            Up To Date
          </button>
          <button class="btn-compact btn-secondary" onClick={() => setUpdateState("error")}>
            Error
          </button>
          <button class="btn-compact" onClick={() => setShowUpdateModal(true)}>
            Open Update Modal
          </button>
        </div>
        <div style={{ marginTop: "10px", display: "flex", alignItems: "center", gap: "10px" }}>
          <span class="version-text">v1.0.4</span>
          {showUpdatePill && (
            <button class="update-pill" onClick={() => setShowUpdateModal(true)}>
              <span class="update-pill-icon" aria-hidden="true">
                !
              </span>
              <span>Update available</span>
            </button>
          )}
        </div>
      </div>

      <div class="ui-lab-section effects-list">
        <h3>Toast Preview</h3>
        <div class="ui-lab-controls-inline">
          <button
            class="btn-compact"
            onClick={() => onToast("success", "Settings applied successfully.", 3500)}
          >
            Show Success Toast
          </button>
          <button
            class="btn-compact btn-secondary"
            onClick={() => onToast("info", "Update check started.", 3500)}
          >
            Show Info Toast
          </button>
          <button
            class="btn-compact btn-warning"
            onClick={() => onToast("error", "Failed to apply setting.", 3500)}
          >
            Show Error Toast
          </button>
        </div>
      </div>

      {showUpdateModal && (
        <div class="modal-overlay" onClick={() => setShowUpdateModal(false)}>
          <section class="update-modal" onClick={(event) => event.stopPropagation()}>
            <h2>{updateTitle}</h2>
            <p>{updateMessage}</p>
            <p class="update-last-checked">Last checked: Just now</p>
            <div class="update-modal-actions">
              <button class="btn-compact btn-secondary" onClick={() => setShowUpdateModal(false)}>
                Later
              </button>
              <button class="btn-compact" onClick={() => setShowUpdateModal(false)}>
                {updateState === "available" ? "Download Latest" : "Close"}
              </button>
            </div>
          </section>
        </div>
      )}
    </section>
  );
}
