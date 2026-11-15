import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "preact/hooks";
import { IS_LINUX } from "../app/platform";
import { logUiEvent } from "../app/uiLog";
import { withTimeout } from "../app/withTimeout";
import type { G6Settings, ToastType } from "../types/g6";

const CONNECT_TIMEOUT_MS = 1000;
// A read of an unresponsive G6 heals the handle (reopen, then USB reset)
// before it fails, which takes a few seconds; the backend emits
// "device-update" when it finishes either way.
const READ_TIMEOUT_MS = 15000;

export type DeviceConnection = ReturnType<typeof useDeviceConnection>;

/// The G6 connection and the backend's cached settings. Connects when the
/// device appears ("device-availability") and reloads the settings on every
/// "device-update".
export function useDeviceConnection(showToast: (message: string, type: ToastType, durationMs?: number) => void) {
  const [connected, setConnected] = useState(false);
  const [status, setStatus] = useState("Disconnected");
  const [settings, setSettings] = useState<G6Settings | null>(null);
  const [permissionError, setPermissionError] = useState(false);

  const isConnectingRef = useRef(false);
  const connectedRef = useRef(false);
  const connectAttemptRef = useRef(0);

  useEffect(() => {
    connectedRef.current = connected;
  }, [connected]);

  const showDisconnected = (statusText = "Disconnected") => {
    setConnected(false);
    setStatus(statusText);
    setSettings(null);
  };

  const loadSettings = async () => {
    try {
      if (!(await invoke<boolean>("is_device_connected"))) {
        showDisconnected();
        return;
      }
      setConnected(true);
      setStatus("Connected");
      setSettings(await invoke<G6Settings>("get_device_settings"));
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  };

  const readDeviceStateSilently = async () => {
    try {
      setSettings(await withTimeout(invoke<G6Settings>("read_device_state"), READ_TIMEOUT_MS));
    } catch (error) {
      await logUiEvent(`Silent read failed: ${String(error)}`);
    }
  };

  const connectDevice = async (): Promise<void> => {
    if (isConnectingRef.current) {
      return;
    }
    isConnectingRef.current = true;
    const attemptId = ++connectAttemptRef.current;

    try {
      setPermissionError(false);
      await withTimeout(invoke("connect_device"), CONNECT_TIMEOUT_MS);
      if (attemptId !== connectAttemptRef.current) {
        return;
      }
      await logUiEvent("Connected to the G6");
      setConnected(true);
      setStatus("Connected");
      await loadSettings();
      void readDeviceStateSilently();
    } catch (error) {
      if (attemptId !== connectAttemptRef.current) {
        return;
      }
      const message = String(error);
      if (message.includes("Permission denied")) {
        setPermissionError(true);
        showDisconnected("USB Permission Denied (Linux)");
        await logUiEvent("Connection failed: USB permission denied");
      } else {
        showDisconnected();
        await logUiEvent(`Connection failed: ${message}`);
      }
    } finally {
      if (attemptId === connectAttemptRef.current) {
        isConnectingRef.current = false;
      }
    }
  };

  const resetConnection = async () => {
    try {
      await invoke("disconnect_device");
      await logUiEvent("Connection reset requested from Settings");
      showDisconnected("Searching for device...");
      showToast("Connection reset requested. Reconnecting...", "info", 2500);
      await connectDevice();
    } catch (error) {
      showToast(`Failed to reset connection: ${error}`, "error");
    }
  };

  const setupUsbPermissions = async () => {
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

  useEffect(() => {
    if (IS_LINUX) {
      invoke<boolean>("check_usb_permissions")
        .then((hasPermissions) => (hasPermissions ? undefined : setupUsbPermissions()))
        .catch((error) => console.error("Failed to check USB permissions:", error));
    }

    const unlistenUpdate = listen("device-update", () => void loadSettings());
    const unlistenAvailability = listen<{ available: boolean }>("device-availability", (event) => {
      if (!event.payload.available) {
        showDisconnected();
      } else if (!isConnectingRef.current && !connectedRef.current) {
        void connectDevice();
      }
    });

    void connectDevice();

    return () => {
      unlistenUpdate.then((unlisten) => unlisten());
      unlistenAvailability.then((unlisten) => unlisten());
    };
  }, []);

  return {
    connected,
    status,
    settings,
    permissionError,
    resetConnection,
    setupUsbPermissions,
  };
}
