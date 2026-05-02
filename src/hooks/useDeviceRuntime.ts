import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  disable as disableAutostart,
  enable as enableAutostart,
  isEnabled as isAutostartEnabled,
} from "@tauri-apps/plugin-autostart";
import { useEffect, useRef, useState } from "preact/hooks";
import { routeFromHash, type AppRoute } from "../app/routes";
import type { G6Settings, ToastType } from "../types/g6";

interface UseDeviceRuntimeProps {
  showToast: (message: string, type: ToastType, durationMs?: number) => void;
}

export function useDeviceRuntime({ showToast }: UseDeviceRuntimeProps) {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState("Disconnected");
  const [settings, setSettings] = useState<G6Settings | null>(null);
  const [activeTab, setActiveTab] = useState<AppRoute>(routeFromHash(window.location.hash));
  const [appVersion, setAppVersion] = useState("");
  const [isLinux, setIsLinux] = useState(true);
  const [logSeparatorMessage, setLogSeparatorMessage] = useState("");
  const [micBoost, setMicBoost] = useState(0);
  const [permissionError, setPermissionError] = useState(false);
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [showExperimental, setShowExperimental] = useState(
    () => localStorage.getItem("rusty-g6-experimental") === "true"
  );

  const pollEnabledRef = useRef(false);

  const logUiEvent = async (message: string) => {
    try {
      await invoke("log_ui_event", { message });
    } catch {
      // no-op
    }
  };

  const toggleExperimental = (enabled: boolean) => {
    setShowExperimental(enabled);
    localStorage.setItem("rusty-g6-experimental", String(enabled));
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

  const navigate = (route: AppRoute) => {
    const nextHash = `#/${route}`;
    if (window.location.hash === nextHash) {
      setActiveTab(route);
      return;
    }
    window.location.hash = nextHash;
  };

  const loadVersion = async () => {
    try {
      const version = await invoke<string>("get_app_version");
      setAppVersion(version);
      await logUiEvent(`Session metadata: app_version=${version}`);
    } catch (error) {
      console.error("Failed to get app version:", error);
    }
  };

  const listUsbDevices = async () => {
    try {
      const devices = await invoke<string[]>("list_usb_devices");
      console.log("=== All USB HID Devices ===");
      devices.forEach((device) => console.log(device));
      console.log("===========================");
      await logUiEvent(`USB scan completed: ${devices.length} device entries observed`);
    } catch (error) {
      console.error("Failed to list USB devices:", error);
    }
  };

  const loadSettings = async () => {
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
      if (deviceSettings.firmware_info) {
        await logUiEvent(
          `Device snapshot: firmware=${deviceSettings.firmware_info.version} output=${deviceSettings.output}`
        );
      }
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  };

  const readDeviceState = async () => {
    try {
      const deviceSettings = await invoke<G6Settings>("read_device_state");
      setSettings(deviceSettings);
      if (deviceSettings.firmware_info) {
        await logUiEvent(
          `Read device state: firmware=${deviceSettings.firmware_info.version} output=${deviceSettings.output} sbx=${deviceSettings.sbx_enabled}`
        );
      }
      showToast(
        "Device state read successfully! All settings now reflect actual device values.",
        "success"
      );
    } catch (error) {
      console.error("Failed to read device state:", error);
      showToast(`Failed to read device state: ${error}`, "error");
    }
  };

  const connectDevice = async (silent = false) => {
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
      await logUiEvent(`Connection attempt result: ${String(result)}`);
      setConnected(true);
      setStatus("Connected");
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
        await logUiEvent("Connection failed: USB permission denied");
      } else {
        setStatus("Disconnected");
        await logUiEvent(`Connection failed: ${errorMsg}`);
      }
    }
  };

  const disconnectDevice = async () => {
    try {
      await invoke("disconnect_device");
      await logUiEvent("Device disconnected by user action");
      setConnected(false);
      setStatus("Disconnected");
      setSettings(null);
    } catch (error) {
      showToast(`Failed to disconnect: ${error}`, "error");
    }
  };

  const resetConnection = async () => {
    try {
      await invoke("disconnect_device");
      await logUiEvent("Connection reset requested from debug tools");
      setConnected(false);
      setStatus("Searching for device...");
      setSettings(null);
      showToast("Connection reset requested. Reconnecting...", "info", 2500);
      await connectDevice(true);
    } catch (error) {
      showToast(`Failed to reset connection: ${error}`, "error");
    }
  };

  const handleSetupUsbPermissions = async () => {
    try {
      showToast("Setting up USB permissions...", "info");
      const result = await invoke<string>("setup_udev_rules");
      await logUiEvent("USB permissions setup command succeeded");
      showToast(result, "success", 5000);
      setPermissionError(false);
    } catch (error) {
      console.error("Failed to setup permissions:", error);
      showToast(`Setup failed: ${error}`, "error");
    }
  };

  const checkPermissionsAndSetup = async () => {
    try {
      const hasPermissions = await invoke<boolean>("check_usb_permissions");
      if (!hasPermissions) {
        console.log("USB permissions missing, prompting for setup...");
        await handleSetupUsbPermissions();
      }
    } catch (error) {
      console.error("Failed to check USB permissions:", error);
    }
  };

  const toggleOutput = async () => {
    try {
      await invoke("toggle_output");
      await logUiEvent("Output toggled");
      await loadSettings();
    } catch (error) {
      showToast(`Failed to toggle output: ${error}`, "error");
    }
  };

  const setSbxMode = async (enabled: "Enabled" | "Disabled") => {
    try {
      console.log("Setting SBX Mode:", enabled);
      await logUiEvent(`SBX mode requested: ${enabled}`);
      await invoke("set_sbx_mode", { enabled });
    } catch (error) {
      console.error("Failed to set SBX Mode:", error);
      showToast(`Failed to set SBX Mode: ${error}`, "error");
    }
  };

  const setScoutMode = async (enabled: "Enabled" | "Disabled") => {
    try {
      console.log("Setting Scout Mode:", enabled);
      await logUiEvent(`Scout mode requested: ${enabled}`);
      await invoke("set_scout_mode", { enabled });
    } catch (error) {
      console.error("Failed to set Scout Mode:", error);
      showToast(`Failed to set Scout Mode: ${error}`, "error");
    }
  };

  const setEffect = async (
    effectName: string,
    enabled: "Enabled" | "Disabled",
    value: number
  ) => {
    try {
      console.log(`Setting ${effectName}:`, { enabled, value });
      await logUiEvent(`Effect change requested: ${effectName} enabled=${enabled} value=${value}`);
      const result = await invoke(`set_${effectName}`, { enabled, value });
      console.log(`${effectName} result:`, result);
    } catch (error) {
      console.error(`Failed to set ${effectName}:`, error);
      showToast(`Failed to set ${effectName}: ${error}`, "error", 3000);
    }
  };

  const setMicrophoneBoost = async (dbValue: number) => {
    try {
      console.log("Setting Microphone Boost:", dbValue);
      await logUiEvent(`Microphone boost requested: ${dbValue} dB`);
      await invoke("set_microphone_boost", { dbValue });
      setMicBoost(dbValue);
      showToast(`Microphone boost set to ${dbValue}dB`, "success", 2000);
    } catch (error) {
      console.error("Failed to set microphone boost:", error);
      showToast(`Failed to set mic boost: ${error}`, "error", 3000);
    }
  };

  const configureMicrophone = async () => {
    try {
      showToast("Configuring microphone...", "info");
      await invoke<string>("configure_microphone");
      showToast(
        'Microphone configured! Now set your system default input device to "Digital Input (S/PDIF) Sound BlasterX G6"',
        "info",
        8000
      );
    } catch (error) {
      showToast(`Failed to configure microphone: ${error}`, "error");
    }
  };

  const showWindowsMicrophoneGuidance = () => {
    showToast(
      "Microphone setup is not required on Windows - it works automatically",
      "info",
      4000
    );
  };

  const handleSetupMicClick = () => {
    if (isLinux) {
      configureMicrophone();
    } else {
      showWindowsMicrophoneGuidance();
    }
  };

  const clearTerminal = async () => {
    try {
      await invoke("clear_terminal", {
        message: logSeparatorMessage || null,
      });
      await logUiEvent("Log separator inserted from debug tools");
      showToast("Log separator added - check terminal for marker", "success", 2000);
      setLogSeparatorMessage("");
    } catch (error) {
      console.error("Failed to add log separator:", error);
      showToast(`Failed to add log separator: ${error}`, "error", 3000);
    }
  };

  const copySessionLog = async () => {
    try {
      const text = await invoke<string>("get_session_log_text");
      await invoke("plugin:clipboard-manager|write_text", { text });
      await logUiEvent(`Session log copied to clipboard (${text.length} chars)`);
      showToast("Session log copied to clipboard.", "success");
    } catch (error) {
      showToast(`Failed to copy session log: ${error}`, "error");
    }
  };

  const openSessionLog = async () => {
    try {
      await invoke("open_session_log");
      await logUiEvent("Session log file open requested");
    } catch (error) {
      showToast(`Failed to open session log: ${error}`, "error");
    }
  };

  useEffect(() => {
    const syncRouteFromHash = () => {
      setActiveTab(routeFromHash(window.location.hash));
    };

    window.addEventListener("hashchange", syncRouteFromHash);
    syncRouteFromHash();

    const userAgent = navigator.userAgent.toLowerCase();
    setIsLinux(userAgent.includes("linux"));

    if (userAgent.includes("linux")) {
      checkPermissionsAndSetup();
    }
    listUsbDevices();
    loadVersion();
    isAutostartEnabled().then(setAutostartEnabled).catch(console.error);

    const unlistenPromise = listen("device-update", () => {
      console.log("Device update event received - refreshing state from memory");
      loadSettings();
    });

    return () => {
      window.removeEventListener("hashchange", syncRouteFromHash);
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    pollEnabledRef.current = connected;
  }, [connected]);

  useEffect(() => {
    if (settings) {
      setMicBoost(settings.microphone_boost);
    }
  }, [settings]);

  useEffect(() => {
    let interval: ReturnType<typeof setInterval> | undefined;
    if (!connected) {
      connectDevice(true);
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

  return {
    connected,
    status,
    settings,
    activeTab,
    appVersion,
    isLinux,
    logSeparatorMessage,
    micBoost,
    permissionError,
    autostartEnabled,
    showExperimental,
    navigate,
    connectDevice,
    disconnectDevice,
    resetConnection,
    handleSetupUsbPermissions,
    toggleOutput,
    setSbxMode,
    setScoutMode,
    setEffect,
    readDeviceState,
    toggleAutostart,
    toggleExperimental,
    setLogSeparatorMessage,
    clearTerminal,
    copySessionLog,
    openSessionLog,
    handleSetupMicClick,
    setMicrophoneBoost,
  };
}
