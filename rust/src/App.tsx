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

  async function configureMicrophone() {
    try {
      setStatus("Configuring microphone...");
      const result = await invoke<string>("configure_microphone");
      setStatus(result);
    } catch (error) {
      setStatus(`Failed to configure microphone: ${error}`);
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

  return (
    <main class="container">
      <header>
        <h1>Rusty G6</h1>
        <p class="subtitle">SoundBlaster X G6 Control Panel</p>
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

      {connected && settings && (
        <>
          <section class="input-section">
            <h2>Input</h2>
            <div class="input-control">
              <p class="input-description">Configure microphone capture settings</p>
              <button onClick={configureMicrophone} class="btn-toggle">
                Configure Microphone
              </button>
            </div>
          </section>

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

            <div class="effects-list">
              <h3>Audio Effects</h3>

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

export default App;
