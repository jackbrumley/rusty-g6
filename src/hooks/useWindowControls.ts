import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "preact/hooks";
import type { ToastType } from "../types/g6";

type ResizeDirection = "East" | "North" | "NorthEast" | "NorthWest" | "South" | "SouthEast" | "SouthWest" | "West";

/// Where the backend parked the main window (`tray::WindowParking`).
type WindowParking = "tray" | "taskbar";

export function useWindowControls(
  showToast: (message: string, type: ToastType, durationMs?: number) => void
) {
  const [isMaximized, setIsMaximized] = useState(false);
  const trayFallbackNotifiedRef = useRef(false);

  const notifyIfParkedInTaskbar = (parking: WindowParking | null) => {
    if (parking === "taskbar" && !trayFallbackNotifiedRef.current) {
      trayFallbackNotifiedRef.current = true;
      showToast("System tray is unavailable on this desktop. Minimized to taskbar instead.", "info", 4200);
    }
  };

  useEffect(() => {
    const appWindow = getCurrentWindow();

    const syncMaximized = async () => {
      try {
        setIsMaximized(await appWindow.isMaximized());
      } catch (error) {
        console.error("Failed to read maximize state:", error);
      }
    };

    syncMaximized();
    const unlistenPromise = appWindow.onResized(() => {
      syncMaximized();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten()).catch(() => {
        // no-op
      });
    };
  }, []);

  // Parks the window: hidden to the tray when one exists, else minimized.
  const handleMinimize = async () => {
    try {
      notifyIfParkedInTaskbar(await invoke<WindowParking>("minimize_to_tray_or_taskbar"));
    } catch (error) {
      console.error("Failed to park window:", error);
      await getCurrentWindow().minimize();
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
      setIsMaximized(await appWindow.isMaximized());
    } catch (error) {
      console.error("Failed to toggle maximize:", error);
    }
  };

  // Honors the close-to-tray setting: parks the window, or quits Rusty G6.
  const handleClose = async () => {
    try {
      notifyIfParkedInTaskbar(await invoke<WindowParking | null>("close_main_window"));
    } catch (error) {
      console.error("Failed to close window:", error);
      await getCurrentWindow().close();
    }
  };

  const handleTitleBarMouseDown = async (e: MouseEvent) => {
    if (e.detail > 1) {
      e.preventDefault();
      return;
    }

    if (e.button === 0 && !(e.target as HTMLElement).closest(".title-bar-button")) {
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

  // The window has no decorations, so the corner overlays start resizing.
  const handleResizeMouseDown = (direction: ResizeDirection) => async (event: MouseEvent) => {
    if (event.buttons !== 1) {
      return;
    }

    if (isMaximized) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    try {
      await getCurrentWindow().startResizeDragging(direction);
    } catch (error) {
      console.error("Failed to start resize dragging:", error);
    }
  };

  return {
    isMaximized,
    handleResizeMouseDown,
    handleMinimize,
    handleToggleMaximize,
    handleClose,
    handleTitleBarMouseDown,
    handleTitleBarDoubleClick,
  };
}
