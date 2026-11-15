import type { ComponentChildren } from "preact";

interface SettingsSectionProps {
  title: string;
  children: ComponentChildren;
}

/// A titled group of settings rows: plain rows with hairline dividers.
export function SettingsSection({ title, children }: SettingsSectionProps) {
  return (
    <section class="settings-section">
      <h3 class="settings-section-title">{title}</h3>
      <div class="settings-rows">{children}</div>
    </section>
  );
}
