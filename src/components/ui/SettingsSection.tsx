import type { ComponentChildren } from "preact";

interface SettingsSectionProps {
  title: string;
  /// Span both columns of a `SettingsPage` (e.g. the EQ curve).
  wide?: boolean;
  children: ComponentChildren;
}

/// A titled group of settings rows: plain rows with hairline dividers.
export function SettingsSection({ title, wide = false, children }: SettingsSectionProps) {
  return (
    <section class={`settings-section ${wide ? "settings-section-wide" : ""}`}>
      <h3 class="settings-section-title">{title}</h3>
      <div class="settings-rows">{children}</div>
    </section>
  );
}
