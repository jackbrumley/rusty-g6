import { useState, useEffect, useRef } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  enable as enableAutostart,
  disable as disableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import "./App.css";

interface FirmwareInfo {
  version: string;
  build: string | null;
}

interface EqualizerBand {
  frequency: number;
  gain: number;
}

interface EqualizerConfig {
  enabled: "Enabled" | "Disabled";
  bands: EqualizerBand[];
}

interface ExtendedAudioParams {
  param_0x0a: number | null;
  param_0x0b: number | null;
  param_0x0c: number | null;
  param_0x0d: number | null;
  param_0x0e: number | null;
  param_0x0f: number | null;
  param_0x10: number | null;
  param_0x11: number | null;
  param_0x12: number | null;
  param_0x13: number | null;
  param_0x14: number | null;
  param_0x1a: number | null;
  param_0x1b: number | null;
  param_0x1c: number | null;
  param_0x1d: number | null;
}

interface G6Settings {
  // Controllable settings (read-write)
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

  // Microphone boost (0, 10, 20, or 30 dB)
  microphone_boost: number;

  // Global SBX processing switch
  sbx_enabled: "Enabled" | "Disabled";

  // Read-only device information
  firmware_info: FirmwareInfo | null;
  scout_mode: "Enabled" | "Disabled";
  equalizer: EqualizerConfig | null;
  extended_params: ExtendedAudioParams | null;

  // Device connection state
  is_connected: boolean;
  last_read_time: number | null;
}

interface ToastMessage {
  id: number;
  message: string;
  type: "success" | "error" | "info";
  durationMs: number;
}

type AppRoute = "status" | "output" | "input" | "debug" | "ui-lab";

const DEFAULT_ROUTE: AppRoute = "status";

const routeFromHash = (hash: string): AppRoute => {
  const normalized = hash.replace(/^#\/?/, "").split("/")[0].trim().toLowerCase();
  if (
    normalized === "status" ||
    normalized === "output" ||
    normalized === "input" ||
    normalized === "debug" ||
    normalized === "ui-lab"
  ) {
    return normalized as AppRoute;
  }
  if (normalized === "main") {
    return "status";
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
  const [toast, setToast] = useState<ToastMessage | null>(null);
  const toastIdRef = useRef(0);
  const [appVersion, setAppVersion] = useState<string>("");
  const [isLinux, setIsLinux] = useState(true);
  const [logSeparatorMessage, setLogSeparatorMessage] = useState<string>("");
  const [micBoost, setMicBoost] = useState<number>(0);
  const [permissionError, setPermissionError] = useState(false);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [showExperimental, setShowExperimental] = useState(() => {
    return localStorage.getItem("rusty-g6-experimental") === "true";
  });

  const toggleExperimental = (enabled: boolean) => {
    setShowExperimental(enabled);
    localStorage.setItem("rusty-g6-experimental", String(enabled));
  };

  const showToast = (
    message: string,
    type: "success" | "error" | "info",
    durationMs = type === "success" ? 3000 : 5000
  ) => {
    toastIdRef.current += 1;
    setToast({
      id: toastIdRef.current,
      message,
      type,
      durationMs,
    });
  };

  const toggleAutostart = async (enabled: boolean) => {
    try {
      if (enabled) {
        await enableAutostart();
      } else {
        await disableAutostart();
      }
      setAutostartEnabled(enabled);
      showToast(`Auto-start ${enabled ? "enabled" : "disabled"}`, "success");
    } catch (error) {
      console.error("Failed to toggle autostart:", error);
      showToast(`Failed to toggle auto-start: ${error}`, "error");
    }
  };

  // Use ref to control polling logic if needed (mostly replaced by events now)
  const pollEnabledRef = useRef(false);

  // Check connection status on mount
  useEffect(() => {
    const syncRouteFromHash = () => {
      setActiveTab(routeFromHash(window.location.hash));
    };

    window.addEventListener("hashchange", syncRouteFromHash);
    syncRouteFromHash();

    // Detect OS from user agent
    const userAgent = navigator.userAgent.toLowerCase();
    setIsLinux(userAgent.includes("linux"));

    // Check USB permissions on launch (Linux only)
    if (userAgent.includes("linux")) {
      checkPermissionsAndSetup();
    }
    // List all USB devices for debugging
    listUsbDevices();
    // Load app version
    loadVersion();
    // Check autostart status
    isAutostartEnabled().then(setAutostartEnabled).catch(console.error);

    // Listen for device updates (from listener thread)
    const unlistenPromise = listen("device-update", () => {
      console.log(
        "Device update event received - refreshing state from memory"
      );
      // Don't query the device - just read the already-updated internal state
      loadSettings();
    });

    return () => {
      window.removeEventListener("hashchange", syncRouteFromHash);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Update polling status when connected state changes
  useEffect(() => {
    pollEnabledRef.current = connected;
  }, [connected]);

  // Sync microphone boost from settings
  useEffect(() => {
    if (settings) {
      setMicBoost(settings.microphone_boost);
    }
  }, [settings]);

  // Auto-connect loop
  useEffect(() => {
    let interval: ReturnType<typeof setInterval> | undefined;
    if (!connected) {
      // Initial check
      connectDevice(true);

      // Periodic check every 3 seconds
      interval = setInterval(() => {
        connectDevice(true);
      }, 3000);
    }
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [connected]);

  async function checkPermissionsAndSetup() {
    try {
      const hasPermissions = await invoke<boolean>("check_usb_permissions");
      if (!hasPermissions) {
        console.log("USB permissions missing, prompting for setup...");
        await handleSetupUsbPermissions();
      }
    } catch (error) {
      console.error("Failed to check USB permissions:", error);
    }
  }

  async function loadVersion() {
    try {
      const version = await invoke<string>("get_app_version");
      setAppVersion(version);
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  }

  async function listUsbDevices() {
    try {
      const devices = await invoke<string[]>("list_usb_devices");
      console.log("=== All USB HID Devices ===");
      devices.forEach((device) => console.log(device));
      console.log("===========================");
    } catch (error) {
      console.error("Failed to list USB devices:", error);
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

  async function loadSettings() {
    try {
      const isConnected = await invoke<boolean>("is_device_connected");
      if (!isConnected) {
        setConnected(false);
        setStatus("Disconnected");
        setSettings(null);
        return;
      }

      setConnected(true);
      setStatus("Connected");
      const deviceSettings = await invoke<G6Settings>("get_device_settings");
      setSettings(deviceSettings);
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  }

  async function readDeviceState() {
    try {
      const deviceSettings = await invoke<G6Settings>("read_device_state");
      setSettings(deviceSettings);
      showToast(
        "Device state read successfully! All settings now reflect actual device values.",
        "success"
      );
    } catch (error) {
      console.error("Failed to read device state:", error);
      showToast(`Failed to read device state: ${error}`, "error");
    }
  }

  async function connectDevice(silent = false) {
    try {
      if (!silent) {
        console.log("Attempting to connect to G6 device...");
        setStatus("Connecting...");
      } else {
        setStatus("Searching for device...");
      }

      setPermissionError(false);
      const result = await invoke("connect_device");
      console.log("Connection result:", result);
      setConnected(true);
      setStatus("Connected");
      // Read full device state on connect (includes firmware, equalizer, etc.)
      await readDeviceState();
    } catch (error) {
      if (!silent) {
        console.error("Connection failed:", error);
      }
      const errorMsg = String(error);
      setConnected(false);

      if (errorMsg.includes("Permission denied")) {
        setPermissionError(true);
        setStatus("USB Permission Denied (Linux)");
      } else {
        setStatus("Disconnected");
      }
    }
  }

  async function disconnectDevice() {
    try {
      await invoke("disconnect_device");
      setConnected(false);
      setStatus("Disconnected");
      setSettings(null);
    } catch (error) {
      showToast(`Failed to disconnect: ${error}`, "error");
    }
  }

  async function handleSetupUsbPermissions() {
    try {
      showToast("Setting up USB permissions...", "info");
      const result = await invoke<string>("setup_udev_rules");
      showToast(result, "success", 5000);
      setPermissionError(false);
    } catch (error) {
      console.error("Failed to setup permissions:", error);
      showToast(`Setup failed: ${error}`, "error");
    }
  }

  async function toggleOutput() {
    try {
      await invoke("toggle_output");
      await loadSettings();
    } catch (error) {
      showToast(`Failed to toggle output: ${error}`, "error");
    }
  }

  async function setSbxMode(enabled: "Enabled" | "Disabled") {
    try {
      console.log("Setting SBX Mode:", enabled);
      await invoke("set_sbx_mode", { enabled });
    } catch (error) {
      console.error("Failed to set SBX Mode:", error);
      showToast(`Failed to set SBX Mode: ${error}`, "error");
    }
  }

  async function setScoutMode(enabled: "Enabled" | "Disabled") {
    try {
      console.log("Setting Scout Mode:", enabled);
      await invoke("set_scout_mode", { enabled });
    } catch (error) {
      console.error("Failed to set Scout Mode:", error);
      showToast(`Failed to set Scout Mode: ${error}`, "error");
    }
  }

  async function configureMicrophone() {
    try {
      showToast("Configuring microphone...", "info");
      await invoke<string>("configure_microphone");

      // Show toast with instructions
      showToast(
        'Microphone configured! Now set your system default input device to "Digital Input (S/PDIF) Sound BlasterX G6"',
        "info",
        8000
      );
    } catch (error) {
      showToast(`Failed to configure microphone: ${error}`, "error");
    }
  }

  async function clearTerminal() {
    try {
      await invoke("clear_terminal", {
        message: logSeparatorMessage || null,
      });
      showToast("Log separator added - check terminal for marker", "success", 2000);
      // Clear the input after sending
      setLogSeparatorMessage("");
    } catch (error) {
      console.error("Failed to add log separator:", error);
      showToast(`Failed to add log separator: ${error}`, "error", 3000);
    }
  }

  function showWindowsMicrophoneGuidance() {
    showToast(
      "Microphone setup is not required on Windows - it works automatically",
      "info",
      4000
    );
  }

  function handleSetupMicClick() {
    if (isLinux) {
      configureMicrophone();
    } else {
      showWindowsMicrophoneGuidance();
    }
  }

  async function setMicrophoneBoost(dbValue: number) {
    try {
      console.log("Setting Microphone Boost:", dbValue);
      await invoke("set_microphone_boost", { dbValue });
      setMicBoost(dbValue);
      showToast(`Microphone boost set to ${dbValue}dB`, "success", 2000);
    } catch (error) {
      console.error("Failed to set microphone boost:", error);
      showToast(`Failed to set mic boost: ${error}`, "error", 3000);
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
    } catch (error) {
      console.error(`Failed to set ${effectName}:`, error);
      showToast(`Failed to set ${effectName}: ${error}`, "error", 3000);
    }
  }

  const handleMinimize = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.minimize();
    } catch (error) {
      console.error("Failed to minimize window:", error);
    }
  };

  const handleToggleMaximize = async () => {
    try {
      const appWindow = getCurrentWindow();
      if (await appWindow.isMaximized()) {
        await appWindow.unmaximize();
      } else {
        await appWindow.maximize();
      }
    } catch (error) {
      console.error("Failed to toggle maximize:", error);
    }
  };

  const handleClose = async () => {
    try {
      await invoke("quit_application");
    } catch (error) {
      console.error("Failed to close window:", error);
    }
  };

  const handleTitleBarMouseDown = async (e: MouseEvent) => {
    if (e.detail > 1) {
      e.preventDefault();
      return;
    }

    if (
      e.button === 0 &&
      !(e.target as HTMLElement).closest(".title-bar-button")
    ) {
      try {
        const appWindow = getCurrentWindow();
        await appWindow.startDragging();
      } catch (error) {
        console.error("Failed to start dragging:", error);
      }
    }
  };

  const handleTitleBarDoubleClick = async (e: MouseEvent) => {
    if (!(e.target as HTMLElement).closest(".title-bar-button")) {
      e.preventDefault();
      await handleToggleMaximize();
    }
  };

  return (
    <div class="app-shell">
      {toast && (
        <Toast
          key={toast.id}
          message={toast.message}
          type={toast.type}
          durationMs={toast.durationMs}
          onDismiss={() => setToast(null)}
        />
      )}

      <div
        class="title-bar"
        onMouseDown={handleTitleBarMouseDown}
        onDblClick={handleTitleBarDoubleClick}
      >
        <div class="title-bar-title">Rusty G6</div>
        <div class="title-bar-controls">
          <button
            class="title-bar-button minimize"
            onClick={handleMinimize}
            title="Minimize"
          >
            <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M5 12h14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
            </svg>
          </button>
          <button
            class="title-bar-button"
            onClick={handleToggleMaximize}
            title="Maximize or Restore"
          >
            <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
              <rect
                x="5"
                y="5"
                width="14"
                height="14"
                rx="1.5"
                ry="1.5"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              />
            </svg>
          </button>
          <button
            class="title-bar-button close"
            onClick={handleClose}
            title="Close"
          >
            <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M7 7l10 10" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
              <path d="M17 7L7 17" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>

      <div class="tab-nav-shell">
        <nav class="tab-nav">
          <button
            class={`tab-button ${activeTab === "status" ? "active" : ""}`}
            onClick={() => navigate("status")}
          >
            Status
          </button>
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
      </div>

      <main class="container">
        {activeTab === "status" && (
          <>
            <header class="status-header">
              <h1>Rusty G6</h1>
              <p class="subtitle">SoundBlaster X G6 Control Panel</p>
              <p class="version-text">v{appVersion || "1.0.x"}</p>
            </header>

            <section class="status-section">
              <div class="status-line">
                <span
                  class={`status-indicator ${
                    connected ? "connected" : "disconnected"
                  }`}
                >
                  {connected ? "●" : "○"}
                </span>
                <span class="status-text">{status}</span>
                {!connected ? (
                  <button onClick={() => connectDevice(false)} class="btn-compact">
                    Connect Device
                  </button>
                ) : (
                  <button onClick={disconnectDevice} class="btn-compact btn-secondary">
                    Disconnect
                  </button>
                )}
                {permissionError && (
                  <button
                    onClick={handleSetupUsbPermissions}
                    class="btn-compact btn-warning"
                    title="Automatically set up Linux udev rules for the G6 device. Requires root password."
                  >
                    Fix Permissions
                  </button>
                )}
              </div>
            </section>
          </>
        )}

        {activeTab === "output" && connected && settings && (
          <section class="output-section compact">
            <div class="section-line">
              <span class="section-label">Output:</span>
              <span class="section-value">{settings.output}</span>
              <button onClick={toggleOutput} class="btn-compact">
                Toggle Output
              </button>
            </div>

            <div class="effects-list">
              <h3>Audio Effects</h3>

              <ToggleControl
                label="Scout Mode"
                checked={settings.scout_mode === "Enabled"}
                onChange={(enabled) =>
                  setScoutMode(enabled ? "Enabled" : "Disabled")
                }
              />

              <ToggleControl
                label="SBX Mode"
                checked={settings.sbx_enabled === "Enabled"}
                onChange={(enabled) =>
                  setSbxMode(enabled ? "Enabled" : "Disabled")
                }
              />

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
                disabled={settings.sbx_enabled === "Disabled"}
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
                disabled={settings.sbx_enabled === "Disabled"}
              />

              <EffectControl
                name="Bass"
                enabled={settings.bass_enabled === "Enabled"}
                value={settings.bass_value}
                onChange={(enabled, value) =>
                  setEffect("bass", enabled ? "Enabled" : "Disabled", value)
                }
                disabled={settings.sbx_enabled === "Disabled"}
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
                disabled={settings.sbx_enabled === "Disabled"}
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
                disabled={settings.sbx_enabled === "Disabled"}
              />
            </div>
          </section>
        )}

        {activeTab === "input" && connected && settings && (
          <section class="input-section compact">
            <div class="section-line">
              <span class="section-label">Input:</span>
              {showExperimental ? (
                <button
                  onClick={handleSetupMicClick}
                  class="btn-compact"
                  title={
                    isLinux
                      ? "Configure ALSA mixer for microphone input"
                      : undefined
                  }
                >
                  Setup Mic
                </button>
              ) : (
                <span class="info-note" style={{ fontSize: "0.75rem" }}>
                  Experimental on Linux. Recommended to use motherboard input
                  for reliability.
                </span>
              )}
            </div>

            {showExperimental && (
              <SliderControl
                label="Mic Boost:"
                value={micBoost}
                min={0}
                max={30}
                step={10}
                onChange={setMicrophoneBoost}
                formatValue={(val) => (val > 0 ? `+${val}dB` : `${val}dB`)}
              />
            )}
          </section>
        )}

        {activeTab === "debug" && (
          <section class="debug-section compact">
            <div class="read-only-item">
              <span class="readonly-label">App Version:</span>
              <span class="readonly-value">{appVersion || "Unknown"}</span>
            </div>

            <div class="read-only-item">
              <span class="readonly-label">Firmware:</span>
              <span class="readonly-value">
                {settings?.firmware_info
                  ? settings.firmware_info.version
                  : "Unknown"}
              </span>
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
                      {Object.values(settings.extended_params).filter((v) => v !== null).length}/15 detected (Read-only)
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
              <button
                onClick={() => navigate("ui-lab")}
                class="btn-compact btn-secondary"
              >
                Open UI Lab
              </button>
              <button onClick={readDeviceState} class="btn-compact btn-secondary">
                Read Device State
              </button>
              <ToggleControl
                label="Auto-start with system"
                checked={autostartEnabled}
                onChange={toggleAutostart}
              />
              <ToggleControl
                label="Experimental Features"
                checked={showExperimental}
                onChange={toggleExperimental}
              />
              <input
                type="text"
                class="log-message-input"
                placeholder="Optional: Add a note to the log separator..."
                value={logSeparatorMessage}
                onInput={(e) => setLogSeparatorMessage(e.currentTarget.value)}
                onKeyPress={(e) => {
                  if (e.key === "Enter") {
                    clearTerminal();
                  }
                }}
              />
              <button
                onClick={clearTerminal}
                class="btn-compact btn-full-width"
                title="Add a visual separator marker in the terminal logs with optional message"
              >
                Add Log Separator
              </button>
            </div>
          </section>
        )}

        {activeTab === "ui-lab" && <UiLabPage onBack={() => navigate("debug")} />}

        {(activeTab === "output" || activeTab === "input") && !connected && (
          <div class="info-panel">
            <p>Connect your SoundBlaster X G6 from the Status tab to begin.</p>
            <p class="info-note">
              This page only shows controls once a device session is active.
            </p>
            <button class="btn-compact btn-secondary" onClick={() => navigate("status")}>Go to Status</button>
          </div>
        )}
      </main>
    </div>
  );
}

interface UiLabPageProps {
  onBack: () => void;
}

function UiLabPage({ onBack }: UiLabPageProps) {
  const [previewToggle, setPreviewToggle] = useState(true);
  const [previewValue, setPreviewValue] = useState(42);
  const [previewEnabled, setPreviewEnabled] = useState(true);
  const [previewConnected, setPreviewConnected] = useState(true);
  const [previewPermissionError, setPreviewPermissionError] = useState(false);
  const [previewSbxEnabled, setPreviewSbxEnabled] = useState(true);
  const [showUpdatePill, setShowUpdatePill] = useState(true);
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [updateState, setUpdateState] = useState<"available" | "uptodate" | "error">("available");
  const [toastType, setToastType] = useState<"success" | "error" | "info" | null>(null);

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
          <button class="btn-compact btn-secondary" onClick={() => setPreviewPermissionError((v) => !v)}>
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
            {previewPermissionError && (
              <button class="btn-compact btn-warning">Fix Permissions</button>
            )}
          </div>
        </section>
      </div>

      <div class="ui-lab-section effects-list">
        <h3>Control Previews</h3>
        <div class="ui-lab-controls-inline">
          <button class="btn-compact btn-secondary" onClick={() => setPreviewSbxEnabled((v) => !v)}>
            SBX: {previewSbxEnabled ? "Enabled" : "Disabled"}
          </button>
          <button class="btn-compact" onClick={() => setPreviewValue(64)}>Set Slider 64</button>
        </div>

        <ToggleControl
          label="Toggle Preview"
          checked={previewToggle}
          onChange={setPreviewToggle}
        />

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
          <button class="btn-compact btn-secondary" onClick={() => setUpdateState("available")}>Available</button>
          <button class="btn-compact btn-secondary" onClick={() => setUpdateState("uptodate")}>Up To Date</button>
          <button class="btn-compact btn-secondary" onClick={() => setUpdateState("error")}>Error</button>
          <button class="btn-compact" onClick={() => setShowUpdateModal(true)}>Open Update Modal</button>
        </div>
        <div style={{ marginTop: "10px", display: "flex", alignItems: "center", gap: "10px" }}>
          <span class="version-text">v1.0.4</span>
          {showUpdatePill && (
            <button class="update-pill" onClick={() => setShowUpdateModal(true)}>
              <span class="update-pill-icon" aria-hidden="true">!</span>
              <span>Update available</span>
            </button>
          )}
        </div>
      </div>

      <div class="ui-lab-section effects-list">
        <h3>Toast Preview</h3>
        <div class="ui-lab-controls-inline">
          <button class="btn-compact" onClick={() => setToastType("success")}>Show Success Toast</button>
          <button class="btn-compact btn-secondary" onClick={() => setToastType("info")}>Show Info Toast</button>
          <button class="btn-compact btn-warning" onClick={() => setToastType("error")}>Show Error Toast</button>
        </div>
      </div>

      {showUpdateModal && (
        <div class="modal-overlay" onClick={() => setShowUpdateModal(false)}>
          <section class="update-modal" onClick={(event) => event.stopPropagation()}>
            <h2>{updateTitle}</h2>
            <p>{updateMessage}</p>
            <p class="update-last-checked">Last checked: Just now</p>
            <div class="update-modal-actions">
              <button class="btn-compact btn-secondary" onClick={() => setShowUpdateModal(false)}>Later</button>
              <button class="btn-compact" onClick={() => setShowUpdateModal(false)}>
                {updateState === "available" ? "Download Latest" : "Close"}
              </button>
            </div>
          </section>
        </div>
      )}

      {toastType && (
        <Toast
          message={
            toastType === "success"
              ? "Settings applied successfully."
              : toastType === "info"
                ? "Update check started."
                : "Failed to apply setting."
          }
          type={toastType}
          durationMs={3500}
          onDismiss={() => setToastType(null)}
        />
      )}
    </section>
  );
}

// ============================================================================
// REUSABLE CONTROL COMPONENTS
// ============================================================================

interface SliderControlProps {
  label: string;
  value: number;
  min?: number;
  max?: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
  formatValue?: (value: number) => string;
}

function SliderControl({
  label,
  value,
  min = 0,
  max = 100,
  step = 1,
  unit = "",
  onChange,
  formatValue,
}: SliderControlProps) {
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  const handleSliderChange = (e: Event) => {
    const newValue = parseInt((e.currentTarget as HTMLInputElement).value);
    setLocalValue(newValue);
    onChange(newValue);
  };

  const handleSliderInput = (e: Event) => {
    setLocalValue(parseInt((e.currentTarget as HTMLInputElement).value));
  };

  const displayValue = formatValue
    ? formatValue(localValue)
    : `${localValue}${unit}`;

  return (
    <div class="control-row">
      <span class="control-label">{label}</span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={localValue}
        onInput={handleSliderInput}
        onChange={handleSliderChange}
        class="slider"
      />
      <span class="slider-value">{displayValue}</span>
    </div>
  );
}

interface ToggleControlProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function ToggleControl({ label, checked, onChange }: ToggleControlProps) {
  const handleToggle = (e: Event) => {
    const newChecked = (e.currentTarget as HTMLInputElement).checked;
    onChange(newChecked);
  };

  return (
    <div class="control-row">
      <span class="control-label">{label}</span>
      <label class="toggle-switch">
        <input type="checkbox" checked={checked} onChange={handleToggle} />
        <span class="toggle-slider"></span>
      </label>
      <span class="slider-value">{checked ? "On" : "Off"}</span>
    </div>
  );
}

interface EffectControlProps {
  name: string;
  enabled: boolean;
  value: number;
  onChange: (enabled: boolean, value: number) => void;
  disabled?: boolean;
}

function EffectControl({
  name,
  enabled,
  value,
  onChange,
  disabled = false,
}: EffectControlProps) {
  const [localValue, setLocalValue] = useState(value);
  const [localEnabled, setLocalEnabled] = useState(enabled);

  useEffect(() => {
    setLocalValue(value);
    setLocalEnabled(enabled);
  }, [value, enabled]);

  const handleToggle = (e: Event) => {
    const newEnabled = (e.currentTarget as HTMLInputElement).checked;
    setLocalEnabled(newEnabled);
    onChange(newEnabled, localValue);
  };

  const handleSliderChange = (e: Event) => {
    const newValue = parseInt((e.currentTarget as HTMLInputElement).value);
    setLocalValue(newValue);
    if (localEnabled) {
      onChange(localEnabled, newValue);
    }
  };

  const handleSliderInput = (e: Event) => {
    setLocalValue(parseInt((e.currentTarget as HTMLInputElement).value));
  };

  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <span class="control-label">{name}</span>
      <label class="toggle-switch">
        <input
          type="checkbox"
          checked={localEnabled}
          onChange={handleToggle}
          disabled={disabled}
        />
        <span class="toggle-slider"></span>
      </label>
      <input
        type="range"
        min="0"
        max="100"
        value={localValue}
        onInput={handleSliderInput}
        onChange={handleSliderChange}
        disabled={!localEnabled || disabled}
        class="slider"
      />
      <span class="slider-value">{localValue}</span>
    </div>
  );
}

interface ToastProps {
  message: string;
  type: "success" | "error" | "info";
  durationMs: number;
  onDismiss: () => void;
}

function Toast({ message, type, durationMs, onDismiss }: ToastProps) {
  const timerRef = useRef<number | null>(null);
  const startedAtRef = useRef<number>(0);
  const remainingRef = useRef<number>(durationMs);

  const clearTimer = () => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const startTimer = () => {
    clearTimer();
    startedAtRef.current = Date.now();
    timerRef.current = window.setTimeout(() => {
      onDismiss();
    }, remainingRef.current);
  };

  useEffect(() => {
    remainingRef.current = durationMs;
    startTimer();
    return () => {
      clearTimer();
    };
  }, [durationMs]);

  const handleMouseEnter = () => {
    const elapsed = Date.now() - startedAtRef.current;
    remainingRef.current = Math.max(0, remainingRef.current - elapsed);
    clearTimer();
  };

  const handleMouseLeave = () => {
    if (remainingRef.current > 0) {
      startTimer();
    }
  };

  return (
    <div
      class={`toast toast-${type}`}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      <div class="toast-content">
        <p class="toast-message">{message}</p>
        <button class="toast-close" onClick={onDismiss}>
          ×
        </button>
      </div>
    </div>
  );
}

export default App;
