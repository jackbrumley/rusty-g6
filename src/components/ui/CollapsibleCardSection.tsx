import { IconChevronDown } from "@tabler/icons-preact";
import type { ComponentChildren } from "preact";

interface CollapsibleCardSectionProps {
  title: string;
  children: ComponentChildren;
  isOpen: boolean;
  onToggle: () => void;
}

export function CollapsibleCardSection({ title, children, isOpen, onToggle }: CollapsibleCardSectionProps) {

  return (
    <section class="debug-section compact collapsible-card-section">
      <button
        type="button"
        class="collapsible-card-trigger"
        onClick={onToggle}
        aria-expanded={isOpen}
      >
        <span class="collapsible-card-title">{title}</span>
        <IconChevronDown
          size={16}
          stroke={2.2}
          class={`collapsible-card-chevron ${isOpen ? "open" : ""}`}
          aria-hidden="true"
        />
      </button>

      <div class={`collapsible-card-content ${isOpen ? "open" : ""}`}>
        <div class="collapsible-card-inner">
          <div class="collapsible-card-panel">{children}</div>
        </div>
      </div>
    </section>
  );
}
