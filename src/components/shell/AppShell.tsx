import type { AppRoute } from "../../app/routes";
import type { ComponentChildren } from "preact";

interface AppShellProps {
  activeTab: AppRoute;
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
          <button class="title-bar-button minimize" onClick={onMinimize} title="Minimize">
            <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M5 12h14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
            </svg>
          </button>
          <button class="title-bar-button" onClick={onToggleMaximize} title="Maximize or Restore">
            <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M6 6h12v12H6z" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinejoin="miter" />
            </svg>
          </button>
          <button class="title-bar-button close" onClick={onClose} title="Close">
            <svg class="title-bar-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M7 7l10 10" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" />
              <path d="M17 7L7 17" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>

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
            Debug
          </button>
        </nav>
      </div>

      <div class="workspace-shell" data-no-drag="true">{children}</div>
    </div>
  );
}
