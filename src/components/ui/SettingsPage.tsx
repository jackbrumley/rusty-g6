import type { ComponentChildren } from "preact";

/// The layout of every tab but Status: `SettingsSection`s in one centred
/// column.
export function SettingsPage({ children }: { children: ComponentChildren }) {
  return <div class="settings-page">{children}</div>;
}
