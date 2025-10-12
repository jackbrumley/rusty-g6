import { useState, useEffect } from "preact/hooks";
import { invoke } from "@tauri-apps/api/core";
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

function App() {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState("Disconnected");
  const [settings, setSettings] = useState<G6Settings | null>(null);
  const [activeTab, setActiveTab] = useState<"main" | "microphone" | "debug">("main");
  const [micStatus, setMicStatus] = useState<string>("");
  const [micSetupInProgress, setMicSetupInProgress] = useState(false);

  // Check connection status on mount
  useEffect(() => {
    checkConnection();
    // List all USB devices for debugging
    listUsbDevices();
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

  return (
    <main class="container">
      <header>
        <h1>Rusty G6</h1>
        <p class="subtitle">SoundBlaster X G6 Control Panel</p>
      </header>

      <nav class="tab-nav">
        <button 
          class={`tab-button ${activeTab === "main" ? "active" : ""}`}
          onClick={() => setActiveTab("main")}
        >
          Main
        </button>
        <button 
          class={`tab-button ${activeTab === "microphone" ? "active" : ""}`}
          onClick={() => setActiveTab("microphone")}
        >
          Microphone (Linux)
        </button>
        <button 
          class={`tab-button ${activeTab === "debug" ? "active" : ""}`}
          onClick={() => setActiveTab("debug")}
        >
          Debug
        </button>
      </nav>

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

      {activeTab === "main" && connected && settings && (
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

      {activeTab === "main" && !connected && (
        <div class="info-panel">
          <p>Connect your SoundBlaster X G6 device to begin.</p>
          <p class="info-note">
            Make sure the device is plugged in and drivers are installed.
          </p>
        </div>
      )}

      {activeTab === "microphone" && (
        <MicrophoneTab 
          micStatus={micStatus}
          micSetupInProgress={micSetupInProgress}
          onSetupMicrophone={setupMicrophone}
          onGetStatus={getMicrophoneStatus}
        />
      )}

      {activeTab === "debug" && (
        <DebugTab connected={connected} />
      )}
    </main>
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
}

function DebugTab({ connected }: DebugTabProps) {
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

export default App;
