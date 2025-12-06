import { useState, useEffect, useRef } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
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
  message: string;
  type: "success" | "error" | "info";
}

interface ProtocolConsoleMessage {
  timestamp: number;
  level: string;
  text: string;
  details: string | null;
}

function App() {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState("Disconnected");
  const [settings, setSettings] = useState<G6Settings | null>(null);
  const [toast, setToast] = useState<ToastMessage | null>(null);
  const [appVersion, setAppVersion] = useState<string>("");
  const [isLinux, setIsLinux] = useState(true);
  const [logSeparatorMessage, setLogSeparatorMessage] = useState<string>("");

  // Protocol Console state
  const [consoleMessages, setConsoleMessages] = useState<
    ProtocolConsoleMessage[]
  >([]);
  const [consoleAutoScroll, setConsoleAutoScroll] = useState(true);
  const consoleEndRef = useRef<HTMLDivElement>(null);

  // Use ref to control polling logic if needed (mostly replaced by events now)
  const pollEnabledRef = useRef(false);

  // Check connection status on mount
  useEffect(() => {
    // Detect OS from user agent
    const userAgent = navigator.userAgent.toLowerCase();
    setIsLinux(userAgent.includes("linux"));

    checkConnection();
    // List all USB devices for debugging
    listUsbDevices();
    // Load app version
    loadVersion();

    // Listen for device updates (from listener thread)
    const unlistenPromise = listen("device-update", () => {
      console.log(
        "Device update event received - refreshing state from memory"
      );
      // Don't query the device - just read the already-updated internal state
      loadSettings();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Update polling status when connected state changes
  useEffect(() => {
    pollEnabledRef.current = connected;
  }, [connected]);

  // Protocol Console: Load messages and listen for updates
  useEffect(() => {
    loadConsoleMessages();

    const unlistenPromise = listen<ProtocolConsoleMessage>(
      "protocol-console-update",
      (event) => {
        setConsoleMessages((prev) => [...prev, event.payload]);
      }
    );

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Auto-scroll console
  useEffect(() => {
    if (consoleAutoScroll && consoleEndRef.current) {
      consoleEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [consoleMessages, consoleAutoScroll]);

  async function loadConsoleMessages() {
    try {
      const messages = await invoke<ProtocolConsoleMessage[]>(
        "get_protocol_console_messages"
      );
      setConsoleMessages(messages);
    } catch (error) {
      console.error("Failed to load console messages:", error);
    }
  }

  async function clearConsole() {
    try {
      await invoke("clear_protocol_console");
      setConsoleMessages([]);
    } catch (error) {
      console.error("Failed to clear console:", error);
    }
  }

  async function testProtocolV2() {
    try {
      const result = await invoke<string>("test_protocol_v2");
      setToast({
        message: "Protocol V2 test completed - check console below!",
        type: "success",
      });
      setTimeout(() => setToast(null), 3000);
    } catch (error) {
      setToast({
        message: `Protocol V2 test failed: ${error}`,
        type: "error",
      });
      setTimeout(() => setToast(null), 5000);
    }
  }

  async function testOutputToggleV2() {
    try {
      const result = await invoke<string>("test_output_toggle_v2");
      setToast({
        message: "V2 output toggle test sent - check console for details!",
        type: "success",
      });
      setTimeout(() => setToast(null), 3000);
      // Refresh settings after toggle
      await loadSettings();
    } catch (error) {
      setToast({
        message: `V2 toggle test failed: ${error}`,
        type: "error",
      });
      setTimeout(() => setToast(null), 5000);
    }
  }

  function copyConsoleToClipboard() {
    if (consoleMessages.length === 0) {
      setToast({
        message: "Console is empty - nothing to copy",
        type: "info",
      });
      setTimeout(() => setToast(null), 2000);
      return;
    }

    const text = consoleMessages
      .map((msg) => {
        const timestamp = new Date(msg.timestamp * 1000).toLocaleTimeString();
        const header = `[${timestamp}] [${msg.level}] ${msg.text}`;
        if (msg.details) {
          return `${header}\nDetails:\n${msg.details}\n`;
        }
        return header;
      })
      .join("\n");

    navigator.clipboard.writeText(text).then(
      () => {
        setToast({
          message: `Copied ${consoleMessages.length} messages to clipboard!`,
          type: "success",
        });
        setTimeout(() => setToast(null), 2000);
      },
      (err) => {
        console.error("Failed to copy to clipboard:", err);
        setToast({
          message: "Failed to copy to clipboard",
          type: "error",
        });
        setTimeout(() => setToast(null), 3000);
      }
    );
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

  async function loadSettings() {
    try {
      const deviceSettings = await invoke<G6Settings>("get_device_settings");
      setSettings(deviceSettings);
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  }

  async function readDeviceState() {
    try {
      setStatus("Reading device state...");
      const deviceSettings = await invoke<G6Settings>("read_device_state");
      setSettings(deviceSettings);
      setStatus("Device state read successfully");
      setToast({
        message:
          "Device state read successfully! All settings now reflect actual device values.",
        type: "success",
      });
      setTimeout(() => setToast(null), 4000);
    } catch (error) {
      console.error("Failed to read device state:", error);
      setStatus(`Failed to read device state: ${error}`);
      setToast({
        message: `Failed to read device state: ${error}`,
        type: "error",
      });
      setTimeout(() => setToast(null), 5000);
    }
  }

  // Silent version for polling/events
  async function readDeviceStateSilent() {
    try {
      const deviceSettings = await invoke<G6Settings>("read_device_state");
      // Only update if something changed? React does shallow diff, but object reference changes.
      setSettings(deviceSettings);
    } catch (error) {
      console.error("Silent refresh failed:", error);
    }
  }

  async function synchronizeDevice() {
    try {
      setStatus("Synchronizing device...");
      await invoke("synchronize_device");
      await loadSettings();
      setStatus("Device synchronized");
      setToast({
        message: "Device synchronized successfully!",
        type: "success",
      });
      setTimeout(() => setToast(null), 3000);
    } catch (error) {
      console.error("Failed to synchronize device:", error);
      setStatus(`Failed to synchronize device: ${error}`);
      setToast({
        message: `Failed to synchronize device: ${error}`,
        type: "error",
      });
      setTimeout(() => setToast(null), 5000);
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

  async function setSbxMode(enabled: "Enabled" | "Disabled") {
    try {
      console.log("Setting SBX Mode:", enabled);
      await invoke("set_sbx_mode", { enabled });
      // Initially update UI state optimistically or wait for event?
      // Wait for event is safest but might feel laggy (100ms listener + roundtrip)
      // Call readDeviceStateSilent explicitly for responsiveness
      readDeviceStateSilent();
      setStatus(`SBX Mode ${enabled}`);
    } catch (error) {
      console.error("Failed to set SBX Mode:", error);
      setStatus(`Failed to set SBX Mode: ${error}`);
    }
  }

  async function setScoutMode(enabled: "Enabled" | "Disabled") {
    try {
      console.log("Setting Scout Mode:", enabled);
      await invoke("set_scout_mode", { enabled });
      readDeviceStateSilent();
      setStatus(`Scout Mode ${enabled}`);
    } catch (error) {
      console.error("Failed to set Scout Mode:", error);
      setStatus(`Failed to set Scout Mode: ${error}`);
    }
  }

  async function configureMicrophone() {
    try {
      setStatus("Configuring microphone...");
      const result = await invoke<string>("configure_microphone");
      setStatus(result);

      // Show toast with instructions
      setToast({
        message:
          'Microphone configured! Now set your system default input device to "Digital Input (S/PDIF) Sound BlasterX G6"',
        type: "info",
      });

      // Auto-dismiss toast after 8 seconds
      setTimeout(() => setToast(null), 8000);
    } catch (error) {
      setStatus(`Failed to configure microphone: ${error}`);
      setToast({
        message: `Failed to configure microphone: ${error}`,
        type: "error",
      });
      setTimeout(() => setToast(null), 5000);
    }
  }

  async function clearTerminal() {
    try {
      await invoke("clear_terminal", {
        message: logSeparatorMessage || null,
      });
      setToast({
        message: "Log separator added - check terminal for marker",
        type: "success",
      });
      // Clear the input after sending
      setLogSeparatorMessage("");
      setTimeout(() => setToast(null), 2000);
    } catch (error) {
      console.error("Failed to add log separator:", error);
      setToast({
        message: `Failed to add log separator: ${error}`,
        type: "error",
      });
      setTimeout(() => setToast(null), 3000);
    }
  }

  function showWindowsMicrophoneGuidance() {
    setToast({
      message:
        "Microphone setup is not required on Windows - it works automatically",
      type: "info",
    });

    // Auto-dismiss toast after 4 seconds
    setTimeout(() => setToast(null), 4000);
  }

  function handleSetupMicClick() {
    if (isLinux) {
      configureMicrophone();
    } else {
      showWindowsMicrophoneGuidance();
    }
  }

  async function setEffect(
    effectName: string,
    enabled: "Enabled" | "Disabled",
    value: number
  ) {
    try {
      console.log(`Setting ${effectName}:`, { enabled, value });
      // Optimistic update locally?
      // Not strictly needed if readDeviceStateSilent is fast.
      const result = await invoke(`set_${effectName}`, { enabled, value });
      console.log(`${effectName} result:`, result);
      setStatus(`${effectName} updated`);
      // We don't need to force read here if the listener works,
      // the device will send a Confirmation report -> listener -> emit -> read.
      // But for robustness:
      // readDeviceStateSilent();
    } catch (error) {
      console.error(`Failed to set ${effectName}:`, error);
      setStatus(`Failed to set ${effectName}: ${error}`);
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

  const handleClose = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.close();
    } catch (error) {
      console.error("Failed to close window:", error);
    }
  };

  const handleTitleBarMouseDown = async (e: MouseEvent) => {
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

  return (
    <div class="app">
      {toast && (
        <Toast
          message={toast.message}
          type={toast.type}
          onDismiss={() => setToast(null)}
        />
      )}

      {/* Custom Title Bar */}
      <div class="title-bar" onMouseDown={handleTitleBarMouseDown}>
        <div class="title-bar-title">Rusty G6</div>
        <div class="title-bar-subtitle">SoundBlaster X G6 Control Panel</div>
        <div class="title-bar-controls">
          <button
            class="title-bar-button minimize"
            onClick={handleMinimize}
            title="Minimize"
          >
            ─
          </button>
          <button
            class="title-bar-button close"
            onClick={handleClose}
            title="Close"
          >
            ✕
          </button>
        </div>
      </div>

      <main class="container">
        {/* Status Section - Compact horizontal layout */}
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
              <button onClick={connectDevice} class="btn-compact btn-primary">
                Connect
              </button>
            ) : (
              <button
                onClick={disconnectDevice}
                class="btn-compact btn-secondary"
              >
                Disconnect
              </button>
            )}
          </div>
        </section>

        {connected && settings && (
          <>
            {/* Input Section - Horizontal layout */}
            <section class="input-section compact">
              <div class="section-line">
                <span class="section-label">Input:</span>
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
              </div>
            </section>

            {/* Output Section - Horizontal layout */}
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

                <div class="effect-group-row">
                  <div class="effect-control compact main-switch">
                    <span class="effect-name">SBX Mode</span>
                    <label class="toggle-switch">
                      <input
                        type="checkbox"
                        checked={settings.sbx_enabled === "Enabled"}
                        onChange={(e) =>
                          setSbxMode(
                            e.currentTarget.checked ? "Enabled" : "Disabled"
                          )
                        }
                      />
                      <span class="toggle-slider"></span>
                    </label>
                    <span class="slider-value">
                      {settings.sbx_enabled === "Enabled" ? "On" : "Off"}
                    </span>
                  </div>

                  <div class="effect-control compact main-switch">
                    <span class="effect-name">Scout Mode</span>
                    <label class="toggle-switch">
                      <input
                        type="checkbox"
                        checked={settings.scout_mode === "Enabled"}
                        onChange={(e) =>
                          setScoutMode(
                            e.currentTarget.checked ? "Enabled" : "Disabled"
                          )
                        }
                      />
                      <span class="toggle-slider"></span>
                    </label>
                    <span class="slider-value">
                      {settings.scout_mode === "Enabled" ? "On" : "Off"}
                    </span>
                  </div>
                </div>

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
              </div>
            </section>

            {/* Debug Section - Vertical layout */}
            <section class="debug-section compact">
              <div class="section-line">
                <span class="section-label">Debug:</span>
                <button
                  onClick={readDeviceState}
                  class="btn-compact btn-secondary"
                >
                  Read State
                </button>
                <button onClick={synchronizeDevice} class="btn-compact">
                  Sync
                </button>
              </div>

              {/* Device Information */}
              {(settings.firmware_info ||
                settings.equalizer ||
                settings.extended_params) && (
                <div class="device-details">
                  {settings.firmware_info && (
                    <div class="read-only-item">
                      <span class="readonly-label">Firmware:</span>
                      <span class="readonly-value">
                        {settings.firmware_info.version}
                      </span>
                    </div>
                  )}

                  {settings.equalizer && (
                    <div class="read-only-item">
                      <span class="readonly-label">Equalizer:</span>
                      <span class="readonly-value">
                        {settings.equalizer.enabled} •{" "}
                        {settings.equalizer.bands.length} bands (Read-only)
                      </span>
                    </div>
                  )}

                  {settings.extended_params && (
                    <div class="read-only-item">
                      <span class="readonly-label">Extended Params:</span>
                      <span class="readonly-value">
                        {
                          Object.values(settings.extended_params).filter(
                            (v) => v !== null
                          ).length
                        }
                        /15 detected (Read-only)
                      </span>
                    </div>
                  )}

                  {settings.last_read_time && (
                    <div class="read-only-item">
                      <span class="readonly-label">Last Read:</span>
                      <span class="readonly-value">
                        {new Date(
                          settings.last_read_time * 1000
                        ).toLocaleTimeString()}
                      </span>
                    </div>
                  )}
                </div>
              )}

              <div class="debug-controls">
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

              {/* Protocol Console */}
              <div class="protocol-console-wrapper">
                <div class="console-header">
                  <h4>Protocol Console (V2 Debug)</h4>
                  <div class="console-controls">
                    <button
                      onClick={testProtocolV2}
                      class="btn-compact btn-primary"
                    >
                      Test Firmware V2
                    </button>
                    <button
                      onClick={testOutputToggleV2}
                      class="btn-compact btn-primary"
                      title="Test output toggle using V2 protocol (2 commands)"
                    >
                      Test Toggle V2
                    </button>
                    <button
                      onClick={copyConsoleToClipboard}
                      class="btn-compact"
                      title="Copy all console messages to clipboard"
                    >
                      📋 Copy
                    </button>
                    <button onClick={clearConsole} class="btn-compact">
                      Clear
                    </button>
                    <label class="console-checkbox">
                      <input
                        type="checkbox"
                        checked={consoleAutoScroll}
                        onChange={(e) =>
                          setConsoleAutoScroll(e.currentTarget.checked)
                        }
                      />
                      <span>Auto-scroll</span>
                    </label>
                  </div>
                </div>

                <div class="protocol-console">
                  {consoleMessages.length === 0 ? (
                    <div class="console-empty">
                      No messages yet. Click "Test Protocol V2" to generate test
                      commands.
                    </div>
                  ) : (
                    consoleMessages.map((msg, i) => (
                      <div
                        key={i}
                        class={`console-message console-${msg.level}`}
                      >
                        <span class="console-timestamp">
                          {new Date(msg.timestamp * 1000).toLocaleTimeString()}
                        </span>
                        <span class="console-level">[{msg.level}]</span>
                        <span class="console-text">{msg.text}</span>
                        {msg.details && (
                          <details class="console-details">
                            <summary>Show details</summary>
                            <pre>{msg.details}</pre>
                          </details>
                        )}
                      </div>
                    ))
                  )}
                  <div ref={consoleEndRef} />
                </div>
              </div>
            </section>
          </>
        )}

        {!connected && (
          <div class="info-panel">
            <p>Connect your SoundBlaster X G6 device to begin.</p>
            <p class="info-note">
              Make sure the device is plugged in and drivers are installed.
            </p>
          </div>
        )}
      </main>

      {/* Version display */}
      {appVersion && <div class="app-version">v{appVersion}</div>}
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
    <div class="effect-control compact">
      <span class="effect-name">{name}</span>
      <label class="toggle-switch">
        <input type="checkbox" checked={localEnabled} onChange={handleToggle} />
        <span class="toggle-slider"></span>
      </label>
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
  );
}

interface ToastProps {
  message: string;
  type: "success" | "error" | "info";
  onDismiss: () => void;
}

function Toast({ message, type, onDismiss }: ToastProps) {
  return (
    <div class={`toast toast-${type}`}>
      <div class="toast-content">
        <span class="toast-icon">
          {type === "success" && "✓"}
          {type === "error" && "✕"}
          {type === "info" && "ℹ"}
        </span>
        <p class="toast-message">{message}</p>
        <button class="toast-close" onClick={onDismiss}>
          ×
        </button>
      </div>
    </div>
  );
}

export default App;
