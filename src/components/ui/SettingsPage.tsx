import type { ComponentChildren } from "preact";

/// The layout of every tab but Status: `SettingsSection`s flowing into two
/// balanced columns when the window is wide enough.
export function SettingsPage({ children }: { children: ComponentChildren }) {
  return <div class="settings-page">{children}</div>;
}
