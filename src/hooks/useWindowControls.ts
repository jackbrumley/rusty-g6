import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "preact/hooks";

export function useWindowControls() {
  const [isMaximized, setIsMaximized] = useState(false);

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
      setIsMaximized(await appWindow.isMaximized());
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

  return {
    isMaximized,
    handleMinimize,
    handleToggleMaximize,
    handleClose,
    handleTitleBarMouseDown,
    handleTitleBarDoubleClick,
  };
}
