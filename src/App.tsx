import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";

interface G6Settings {
  output: "Speakers" | "Headphones";
  surround_enabled: "Enabled" | "Disabled";
  surround_value: number;
  crystalizer_enabled: "Enabled" | "Disabled";
  crystalizer_value: number;
  bass_enabled: "Enabled" | "Disabled";
  bass_value: number;
  smart_volume_enabled: "Enabled" | "Disabled";
  smart_volume_value: number;
  smart_volume_preset: "Night" | "Loud" | null;
  dialog_plus_enabled: "Enabled" | "Disabled";
  dialog_plus_value: number;
}

interface UpdateCheckResult {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  releaseUrl: string;
  notesUrl?: string;
}

type AppRoute = "output" | "input" | "debug" | "ui-lab";

const DEFAULT_ROUTE: AppRoute = "output";

const routeFromHash = (hash: string): AppRoute => {
  const normalized = hash.replace(/^#\/?/, "").split("/")[0].trim().toLowerCase();
  if (normalized === "output" || normalized === "debug" || normalized === "ui-lab") {
    return normalized as AppRoute;
  }
  if (normalized === "input") {
    return "input";
  }
  if (normalized === "main") {
    return "output";
  }
  if (normalized === "microphone") {
    return "input";
  }
  return DEFAULT_ROUTE;
};

function App() {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState("Disconnected");
  const [settings, setSettings] = useState<G6Settings | null>(null);
  const [activeTab, setActiveTab] = useState<AppRoute>(routeFromHash(window.location.hash));
  const [micStatus, setMicStatus] = useState<string>("");
  const [micSetupInProgress, setMicSetupInProgress] = useState(false);
  const [appVersion, setAppVersion] = useState("");
  const [checkingUpdates, setCheckingUpdates] = useState(false);
  const [updateResult, setUpdateResult] = useState<UpdateCheckResult | null>(null);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [lastCheckedAt, setLastCheckedAt] = useState<number | null>(null);
  const hasTauriRuntime =
    typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

  // Check connection status on mount
  useEffect(() => {
    const syncRouteFromHash = () => {
      setActiveTab(routeFromHash(window.location.hash));
    };

    window.addEventListener("hashchange", syncRouteFromHash);
    syncRouteFromHash();

    checkConnection();
    // List all USB devices for debugging
    listUsbDevices();

    if (hasTauriRuntime) {
      getVersion().then(setAppVersion).catch(() => {});
      void checkForUpdates(false);
    }

    return () => {
      window.removeEventListener("hashchange", syncRouteFromHash);
    };
  }, []);

  async function listUsbDevices() {
    try {
      const devices = await invoke<string[]>("list_usb_devices");
      console.log("=== All USB HID Devices ===");
      devices.forEach(device => console.log(device));
      console.log("===========================");
    } catch (error) {
      console.error("Failed to list USB devices:", error);
    }
  }

  async function checkConnection() {
    try {
      const isConnected = await invoke<boolean>("is_device_connected");
      setConnected(isConnected);
      if (isConnected) {
        setStatus("Connected");
        await loadSettings();
      } else {
        setStatus("Disconnected");
      }
    } catch (error) {
      console.error("Error checking connection:", error);
      setStatus("Error checking connection");
    }
  }

  function navigate(route: AppRoute) {
    const nextHash = `#/${route}`;
    if (window.location.hash === nextHash) {
      setActiveTab(route);
      return;
    }
    window.location.hash = nextHash;
  }

  async function checkForUpdates(showModalWhenUpToDate: boolean) {
    if (!hasTauriRuntime || checkingUpdates) {
      return;
    }

    setCheckingUpdates(true);
    try {
      const result = await invoke<UpdateCheckResult>("check_for_updates");
      setUpdateResult(result);
      setLastCheckedAt(Date.now());

      if (result.updateAvailable || showModalWhenUpToDate) {
        setShowUpdateModal(true);
      }
    } catch (error) {
      if (showModalWhenUpToDate) {
        setStatus(`Update check failed: ${error}`);
      }
    } finally {
      setCheckingUpdates(false);
    }
  }

  function getLastCheckedLabel() {
    if (!lastCheckedAt) {
      return "Not checked yet";
    }

    const elapsed = Date.now() - lastCheckedAt;
    if (elapsed < 60_000) {
      return "Just now";
    }

    const minutes = Math.floor(elapsed / 60_000);
    return `${minutes} min ago`;
  }

  function openReleasePage() {
    const releaseUrl = updateResult?.releaseUrl || "https://github.com/jackbrumley/rusty-g6/releases/latest";
    window.open(releaseUrl, "_blank", "noopener,noreferrer");
  }

  async function loadSettings() {
    try {
      const deviceSettings = await invoke<G6Settings>("get_device_settings");
      setSettings(deviceSettings);
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  }

  async function connectDevice() {
    try {
      console.log("Attempting to connect to G6 device...");
      setStatus("Connecting...");
      const result = await invoke("connect_device");
      console.log("Connection result:", result);
      setConnected(true);
      setStatus("Connected");
      await loadSettings();
    } catch (error) {
      console.error("Connection failed:", error);
      setStatus(`Connection failed: ${error}`);
      setConnected(false);
    }
  }

  async function disconnectDevice() {
    try {
      await invoke("disconnect_device");
      setConnected(false);
      setStatus("Disconnected");
      setSettings(null);
    } catch (error) {
      setStatus(`Disconnect failed: ${error}`);
    }
  }

  async function toggleOutput() {
    try {
      await invoke("toggle_output");
      await loadSettings();
      setStatus("Output toggled");
    } catch (error) {
      setStatus(`Failed to toggle output: ${error}`);
    }
  }

  async function setEffect(
    effectName: string,
    enabled: "Enabled" | "Disabled",
    value: number
  ) {
    try {
      console.log(`Setting ${effectName}:`, { enabled, value });
      const result = await invoke(`set_${effectName}`, { enabled, value });
      console.log(`${effectName} result:`, result);
      await loadSettings();
      setStatus(`${effectName} updated`);
    } catch (error) {
      console.error(`Failed to set ${effectName}:`, error);
      setStatus(`Failed to set ${effectName}: ${error}`);
    }
  }

  async function setupMicrophone() {
    setMicSetupInProgress(true);
    setMicStatus("Setting up microphone...");
    
    try {
      const result = await invoke<string>("setup_microphone");
      setMicStatus(result);
      console.log("Microphone setup successful:", result);
    } catch (error) {
      const errorMsg = `Failed to setup microphone: ${error}`;
      setMicStatus(errorMsg);
      console.error(errorMsg);
    } finally {
      setMicSetupInProgress(false);
    }
  }

  async function getMicrophoneStatus() {
    try {
      const status = await invoke<string>("get_microphone_status");
      setMicStatus(status);
    } catch (error) {
      setMicStatus(`Failed to get status: ${error}`);
    }
  }

  async function handleCloseApp() {
    if (!hasTauriRuntime) {
      return;
    }

    try {
      await invoke("quit_application");
    } catch (error) {
      console.error("Failed to quit application:", error);
      await getCurrentWindow().close();
    }
  }

  async function handleMinimizeApp() {
    if (!hasTauriRuntime) {
      return;
    }

    try {
      await invoke("minimize_to_tray_or_taskbar");
    } catch (error) {
      console.error("Failed to minimize via backend:", error);
      await getCurrentWindow().minimize();
    }
  }

  async function handleTitleBarMouseDown(event: MouseEvent) {
    if (!hasTauriRuntime) {
      return;
    }

    const target = event.target as HTMLElement;
    if (event.detail > 1) {
      event.preventDefault();
      return;
    }

    if (event.buttons === 1 && !target.closest("button")) {
      event.preventDefault();
      await getCurrentWindow().startDragging();
    }
  }

  async function handleToggleMaximize() {
    if (!hasTauriRuntime) {
      return;
    }

    try {
      const win = getCurrentWindow();
      if (await win.isMaximized()) {
        await win.unmaximize();
      } else {
        await win.maximize();
      }
    } catch (error) {
      console.error("Failed to toggle maximize:", error);
    }
  }

  async function handleTitleBarDoubleClick(event: MouseEvent) {
    if (!hasTauriRuntime) {
      return;
    }

    const target = event.target as HTMLElement;
    if (target.closest("button")) {
      return;
    }

    event.preventDefault();
    await handleToggleMaximize();
  }

  return (
    <div class="app-shell">
      <div class="titlebar" onMouseDown={handleTitleBarMouseDown} onDblClick={handleTitleBarDoubleClick}>
        <div class="titlebar-title">Rusty G6</div>
        <div class="titlebar-controls">
          <button class="titlebar-btn" onClick={handleMinimizeApp} aria-label="Minimize">
            &#x2500;
          </button>
          <button class="titlebar-btn" onClick={handleToggleMaximize} aria-label="Maximize or restore">
            &#x25A1;
          </button>
          <button class="titlebar-btn titlebar-btn-close" onClick={handleCloseApp} aria-label="Close">
            &#x2715;
          </button>
        </div>
      </div>

      <main class="container">

      <header>
        <h1>Rusty G6</h1>
        <p class="subtitle">SoundBlaster X G6 Control Panel</p>
        <div class="meta-row">
          <span class="app-version">v{appVersion || "0.2.x"}</span>
          {updateResult?.updateAvailable && (
            <button class="update-pill" onClick={() => setShowUpdateModal(true)}>
              <span class="update-pill-icon" aria-hidden="true">!</span>
              <span>Update available</span>
            </button>
          )}
        </div>
      </header>

      <section class="status-section">
        <div class="status-bar">
          <span class={`status-indicator ${connected ? "connected" : "disconnected"}`}>
            {connected ? "●" : "○"}
          </span>
          <span class="status-text">{status}</span>
        </div>
        <div class="connection-buttons">
          {!connected ? (
            <button onClick={connectDevice} class="btn-primary">
              Connect Device
            </button>
          ) : (
            <button onClick={disconnectDevice} class="btn-secondary">
              Disconnect
            </button>
          )}
        </div>
      </section>

      <nav class="tab-nav">
        <button
          class={`tab-button ${activeTab === "output" ? "active" : ""}`}
          onClick={() => navigate("output")}
        >
          Output
        </button>
        <button
          class={`tab-button ${activeTab === "input" ? "active" : ""}`}
          onClick={() => navigate("input")}
        >
          Input
        </button>
        <button
          class={`tab-button ${activeTab === "debug" ? "active" : ""}`}
          onClick={() => navigate("debug")}
        >
          Debug
        </button>
      </nav>

      {activeTab === "output" && connected && settings && (
        <>
          <section class="output-section">
            <h2>Output</h2>
            <div class="output-control">
              <div class="current-output">
                Current: <strong>{settings.output}</strong>
              </div>
              <button onClick={toggleOutput} class="btn-toggle">
                Toggle Output
              </button>
            </div>
          </section>

          <section class="effects-section">
            <h2>Audio Effects</h2>

            <EffectControl
              name="Surround"
              enabled={settings.surround_enabled === "Enabled"}
              value={settings.surround_value}
              onChange={(enabled, value) =>
                setEffect(
                  "surround",
                  enabled ? "Enabled" : "Disabled",
                  value
                )
              }
            />

            <EffectControl
              name="Crystalizer"
              enabled={settings.crystalizer_enabled === "Enabled"}
              value={settings.crystalizer_value}
              onChange={(enabled, value) =>
                setEffect(
                  "crystalizer",
                  enabled ? "Enabled" : "Disabled",
                  value
                )
              }
            />

            <EffectControl
              name="Bass"
              enabled={settings.bass_enabled === "Enabled"}
              value={settings.bass_value}
              onChange={(enabled, value) =>
                setEffect("bass", enabled ? "Enabled" : "Disabled", value)
              }
            />

            <EffectControl
              name="Smart Volume"
              enabled={settings.smart_volume_enabled === "Enabled"}
              value={settings.smart_volume_value}
              onChange={(enabled, value) =>
                setEffect(
                  "smart_volume",
                  enabled ? "Enabled" : "Disabled",
                  value
                )
              }
            />

            <EffectControl
              name="Dialog Plus"
              enabled={settings.dialog_plus_enabled === "Enabled"}
              value={settings.dialog_plus_value}
              onChange={(enabled, value) =>
                setEffect(
                  "dialog_plus",
                  enabled ? "Enabled" : "Disabled",
                  value
                )
              }
            />
          </section>
        </>
      )}

      {activeTab === "output" && !connected && (
        <div class="info-panel">
          <p>Connect your SoundBlaster X G6 device to begin.</p>
          <p class="info-note">
            Make sure the device is plugged in and drivers are installed.
          </p>
        </div>
      )}

      {activeTab === "input" && (
        <MicrophoneTab 
          micStatus={micStatus}
          micSetupInProgress={micSetupInProgress}
          onSetupMicrophone={setupMicrophone}
          onGetStatus={getMicrophoneStatus}
        />
      )}

      {activeTab === "debug" && (
        <DebugTab
          connected={connected}
          checkingUpdates={checkingUpdates}
          onCheckForUpdates={() => void checkForUpdates(true)}
          onOpenUiLab={() => navigate("ui-lab")}
        />
      )}

      {activeTab === "ui-lab" && <UiLabPage appVersion={appVersion} onBack={() => navigate("debug")} />}

      {showUpdateModal && (
        <div class="modal-overlay" onClick={() => setShowUpdateModal(false)}>
          <section class="update-modal" onClick={(event) => event.stopPropagation()}>
            <h2>{updateResult?.updateAvailable ? "Update Available" : "Rusty G6 is Up to Date"}</h2>
            <p>
              {updateResult?.updateAvailable
                ? `A newer Rusty G6 version is available. Current: v${updateResult.currentVersion} -> Latest: v${updateResult.latestVersion}.`
                : `You are on the latest version (v${updateResult?.currentVersion || appVersion}).`}
            </p>
            <p class="update-last-checked">Last checked: {getLastCheckedLabel()}</p>
            <div class="update-modal-actions">
              <button class="btn-secondary" onClick={() => setShowUpdateModal(false)}>Later</button>
              <button class="btn-primary" onClick={openReleasePage}>Download Latest</button>
            </div>
          </section>
        </div>
      )}
      </main>
    </div>
  );
}

interface EffectControlProps {
  name: string;
  enabled: boolean;
  value: number;
  onChange: (enabled: boolean, value: number) => void;
}

function EffectControl({ name, enabled, value, onChange }: EffectControlProps) {
  const [localValue, setLocalValue] = useState(value);
  const [localEnabled, setLocalEnabled] = useState(enabled);

  useEffect(() => {
    setLocalValue(value);
    setLocalEnabled(enabled);
  }, [value, enabled]);

  const handleToggle = (e: Event) => {
    const newEnabled = (e.currentTarget as HTMLInputElement).checked;
    setLocalEnabled(newEnabled);
    // Auto-apply when toggle changes
    onChange(newEnabled, localValue);
  };

  const handleSliderChange = (e: Event) => {
    const newValue = parseInt((e.currentTarget as HTMLInputElement).value);
    setLocalValue(newValue);
    // Auto-apply when slider is released
    if (localEnabled) {
      onChange(localEnabled, newValue);
    }
  };

  const handleSliderInput = (e: Event) => {
    // Just update local value for visual feedback while dragging
    setLocalValue(parseInt((e.currentTarget as HTMLInputElement).value));
  };

  return (
    <div class="effect-control">
      <div class="effect-header">
        <h3>{name}</h3>
        <label class="toggle-switch">
          <input
            type="checkbox"
            checked={localEnabled}
            onChange={handleToggle}
          />
          <span class="toggle-slider"></span>
        </label>
      </div>
      <div class="effect-slider">
        <input
          type="range"
          min="0"
          max="100"
          value={localValue}
          onInput={handleSliderInput}
          onChange={handleSliderChange}
          disabled={!localEnabled}
          class="slider"
        />
        <span class="slider-value">{localValue}</span>
      </div>
    </div>
  );
}

interface MicrophoneTabProps {
  micStatus: string;
  micSetupInProgress: boolean;
  onSetupMicrophone: () => void;
  onGetStatus: () => void;
}

function MicrophoneTab({ micStatus, micSetupInProgress, onSetupMicrophone, onGetStatus }: MicrophoneTabProps) {
  return (
    <section class="microphone-section">
      <h2>Microphone Setup (Linux Only)</h2>
      
      <div class="info-panel">
        <p><strong>About Microphone Setup:</strong></p>
        <p>
          On Linux, the SoundBlaster X G6 requires manual ALSA configuration to enable 
          the external microphone input. This needs to be done each time after rebooting.
        </p>
        <p class="info-note">
          This feature automates the following steps:
        </p>
        <ol>
          <li>Enable External Mic capture in ALSA</li>
          <li>Set PCM Capture Source to External Mic</li>
          <li>Set reasonable input gain (50%)</li>
        </ol>
      </div>

      <div class="microphone-controls">
        <button 
          onClick={onSetupMicrophone} 
          disabled={micSetupInProgress}
          class="btn-primary"
        >
          {micSetupInProgress ? "Setting up..." : "Setup Microphone"}
        </button>
        
        <button 
          onClick={onGetStatus}
          class="btn-secondary"
        >
          Check Status
        </button>
      </div>

      {micStatus && (
        <div class="microphone-result">
          <h3>Result:</h3>
          <pre>{micStatus}</pre>
        </div>
      )}

      <div class="info-panel" style="margin-top: 20px;">
        <p class="info-note">
          <strong>Note:</strong> These settings are temporary and will reset after a reboot. 
          Once we verify this works reliably, we can add an option to auto-configure on app startup.
        </p>
      </div>
    </section>
  );
}

interface DebugTabProps {
  connected: boolean;
  checkingUpdates: boolean;
  onCheckForUpdates: () => void;
  onOpenUiLab: () => void;
}

function DebugTab({ connected, checkingUpdates, onCheckForUpdates, onOpenUiLab }: DebugTabProps) {
  const [reading, setReading] = useState(false);
  const [readResult, setReadResult] = useState<string>("");
  const [fullReadResults, setFullReadResults] = useState<Array<[string, string]>>([]);

  async function readDeviceState() {
    if (!connected) {
      setReadResult("❌ Device not connected");
      return;
    }

    setReading(true);
    setReadResult("Reading...");
    
    try {
      const response = await invoke<string>("read_device_state");
      setReadResult(`✅ Response (0x05):\n${response}`);
    } catch (error) {
      setReadResult(`❌ Error: ${error}`);
    } finally {
      setReading(false);
    }
  }

  async function readFullDeviceState() {
    if (!connected) {
      setReadResult("❌ Device not connected");
      return;
    }

    setReading(true);
    setReadResult("Reading all commands...");
    setFullReadResults([]);
    
    try {
      const results = await invoke<Array<[string, string]>>("read_full_device_state");
      setFullReadResults(results);
      setReadResult(`✅ Read ${results.length} command responses`);
    } catch (error) {
      setReadResult(`❌ Error: ${error}`);
    } finally {
      setReading(false);
    }
  }

  return (
    <section class="debug-section">
      <h2>Device Read Commands (Reverse Engineering)</h2>
      
      {!connected && (
        <div class="info-panel">
          <p>⚠️ Connect the device first to test read commands</p>
        </div>
      )}

      <div class="debug-controls">
        <button 
          onClick={readDeviceState} 
          disabled={!connected || reading}
          class="btn-primary"
        >
          {reading ? "Reading..." : "Read Status (0x05)"}
        </button>
        
        <button 
          onClick={readFullDeviceState} 
          disabled={!connected || reading}
          class="btn-secondary"
        >
          {reading ? "Reading..." : "Read All Commands"}
        </button>
      </div>

      <div class="debug-controls">
        <button onClick={onCheckForUpdates} class="btn-secondary" disabled={checkingUpdates}>
          {checkingUpdates ? "Checking..." : "Check for Updates"}
        </button>
        <button onClick={onOpenUiLab} class="btn-secondary">Open UI Lab</button>
      </div>

      {readResult && (
        <div class="debug-result">
          <h3>Result:</h3>
          <pre>{readResult}</pre>
        </div>
      )}

      {fullReadResults.length > 0 && (
        <div class="debug-responses">
          <h3>All Command Responses:</h3>
          {fullReadResults.map(([name, data]) => (
            <div key={name} class="response-item">
              <h4>{name}</h4>
              <pre class="hex-data">{data}</pre>
            </div>
          ))}
        </div>
      )}

      <div class="debug-info">
        <h3>Discovered Commands:</h3>
        <ul>
          <li><code>0x05</code> - Main status query (HIGH PRIORITY)</li>
          <li><code>0x10</code> - Unknown</li>
          <li><code>0x15</code> - Unknown (params: 01)</li>
          <li><code>0x20</code> - Unknown</li>
          <li><code>0x30</code> - Unknown</li>
          <li><code>0x39</code> - Unknown (params: 01 04)</li>
          <li><code>0x3a</code> - Parameterized query (HIGH PRIORITY)</li>
        </ul>
        <p class="info-note">
          These commands were discovered by capturing USB traffic when Creative software reads device state.
          The responses will help us parse device settings.
        </p>
      </div>
    </section>
  );
}

interface UiLabPageProps {
  appVersion: string;
  onBack: () => void;
}

function UiLabPage({ appVersion, onBack }: UiLabPageProps) {
  const [enabled, setEnabled] = useState(true);
  const [value, setValue] = useState(45);
  const [showUpdateBadge, setShowUpdateBadge] = useState(true);
  const [showUpdatePreviewModal, setShowUpdatePreviewModal] = useState(false);

  return (
    <section class="debug-section">
      <h2>UI Lab (internal)</h2>
      <p class="info-note">Hidden hash route preview for UI states. Access via #/ui-lab.</p>

      <div class="debug-controls">
        <button onClick={onBack} class="btn-secondary">Back to Debug</button>
        <button onClick={() => setShowUpdatePreviewModal(true)} class="btn-primary">Open Update Modal Preview</button>
      </div>

      <div class="info-panel">
        <p><strong>Header Update Pill Preview</strong></p>
        <div class="meta-row" style="justify-content: flex-start; margin-top: 10px;">
          <span class="app-version">v{appVersion || "0.2.x"}</span>
          {showUpdateBadge && (
            <button class="update-pill" onClick={() => setShowUpdatePreviewModal(true)}>
              <span class="update-pill-icon" aria-hidden="true">!</span>
              <span>Update available</span>
            </button>
          )}
        </div>
        <div class="debug-controls" style="margin-top: 10px;">
          <button class="btn-secondary" onClick={() => setShowUpdateBadge((prev) => !prev)}>
            {showUpdateBadge ? "Hide" : "Show"} Update Pill
          </button>
        </div>
      </div>

      <div class="effect-control">
        <div class="effect-header">
          <h3>Effect Control Preview</h3>
          <label class="toggle-switch">
            <input type="checkbox" checked={enabled} onChange={(e) => setEnabled((e.currentTarget as HTMLInputElement).checked)} />
            <span class="toggle-slider"></span>
          </label>
        </div>
        <div class="effect-slider">
          <input
            type="range"
            min="0"
            max="100"
            value={value}
            onInput={(e) => setValue(parseInt((e.currentTarget as HTMLInputElement).value))}
            class="slider"
          />
          <span class="slider-value">{value}</span>
        </div>
      </div>

      <div class="debug-controls">
        <button class="btn-primary">Primary Button</button>
        <button class="btn-secondary">Secondary Button</button>
        <button class="btn-toggle">Toggle Button</button>
      </div>

      {showUpdatePreviewModal && (
        <div class="modal-overlay" onClick={() => setShowUpdatePreviewModal(false)}>
          <section class="update-modal" onClick={(event) => event.stopPropagation()}>
            <h2>Update Available</h2>
            <p>A newer Rusty G6 version is available. Current: v{appVersion || "0.2.x"} -&gt; Latest: v0.2.1.</p>
            <p class="update-last-checked">Last checked: Just now</p>
            <div class="update-modal-actions">
              <button class="btn-secondary" onClick={() => setShowUpdatePreviewModal(false)}>Later</button>
              <button class="btn-primary" onClick={() => setShowUpdatePreviewModal(false)}>Download Latest</button>
            </div>
          </section>
        </div>
      )}
    </section>
  );
}

export default App;
