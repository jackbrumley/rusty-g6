import type { AppRoute } from "../../app/routes";
import type { ComponentChildren } from "preact";
import { IconCopy, IconMinus, IconSquare, IconX } from "@tabler/icons-preact";

interface AppShellProps {
  activeTab: AppRoute;
  isMaximized: boolean;
  onNavigate: (route: AppRoute) => void;
  onMinimize: () => Promise<void>;
  onToggleMaximize: () => Promise<void>;
  onClose: () => Promise<void>;
  onTitleBarMouseDown: (e: MouseEvent) => Promise<void>;
  onTitleBarDoubleClick: (e: MouseEvent) => Promise<void>;
  children: ComponentChildren;
}

export function AppShell({
  activeTab,
  isMaximized,
  onNavigate,
  onMinimize,
  onToggleMaximize,
  onClose,
  onTitleBarMouseDown,
  onTitleBarDoubleClick,
  children,
}: AppShellProps) {
  return (
    <div class="app-shell">
      <div
        class="title-bar"
        onMouseDown={onTitleBarMouseDown}
        onDblClick={onTitleBarDoubleClick}
      >
        <div class="title-bar-title">Rusty G6</div>
        <div class="title-bar-controls">
          <button class="title-bar-button minimize" onClick={onMinimize} aria-label="Minimize window">
            <IconMinus size={14} stroke={2.3} aria-hidden="true" />
          </button>
          <button class="title-bar-button" onClick={onToggleMaximize} aria-label="Maximize or restore window">
            {isMaximized ? (
              <IconCopy size={13} stroke={2.3} aria-hidden="true" />
            ) : (
              <IconSquare size={13} stroke={2.3} aria-hidden="true" />
            )}
          </button>
          <button class="title-bar-button close" onClick={onClose} aria-label="Close application">
            <IconX size={14} stroke={2.3} aria-hidden="true" />
          </button>
        </div>
      </div>

      <div class="app-body-shell" data-no-drag="true">
        <div class="tab-nav-shell">
          <nav class="tab-nav">
            <button class={`tab-button ${activeTab === "status" ? "active" : ""}`} onClick={() => onNavigate("status")}>
              Status
            </button>
            <button class={`tab-button ${activeTab === "output" ? "active" : ""}`} onClick={() => onNavigate("output")}>
              Output
            </button>
            <button class={`tab-button ${activeTab === "input" ? "active" : ""}`} onClick={() => onNavigate("input")}>
              Input
            </button>
            <button class={`tab-button ${activeTab === "debug" ? "active" : ""}`} onClick={() => onNavigate("debug")}>
              Settings
            </button>
          </nav>
        </div>

        <div class="workspace-shell">{children}</div>
      </div>
    </div>
  );
}
