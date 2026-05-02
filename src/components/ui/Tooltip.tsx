import type { ComponentChildren } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";

interface TooltipProps {
  text: string;
  children: ComponentChildren;
}

export function Tooltip({ text, children }: TooltipProps) {
  const anchorRef = useRef<HTMLSpanElement | null>(null);
  const tooltipRef = useRef<HTMLSpanElement | null>(null);
  const [open, setOpen] = useState(false);
  const [position, setPosition] = useState<{ left: number; top: number }>({ left: 0, top: 0 });

  const updatePosition = () => {
    const anchorEl = anchorRef.current;
    const tooltipEl = tooltipRef.current;
    if (!anchorEl || !tooltipEl) {
      return;
    }

    const viewportPadding = 10;
    const gap = 8;
    const anchorRect = anchorEl.getBoundingClientRect();
    const tooltipRect = tooltipEl.getBoundingClientRect();

    const minLeft = viewportPadding;
    const maxLeft = Math.max(viewportPadding, window.innerWidth - viewportPadding - tooltipRect.width);
    const desiredLeft = anchorRect.left + anchorRect.width / 2 - tooltipRect.width / 2;
    const left = Math.min(maxLeft, Math.max(minLeft, desiredLeft));

    const desiredTopAbove = anchorRect.top - tooltipRect.height - gap;
    const desiredTopBelow = anchorRect.bottom + gap;
    const fitsAbove = desiredTopAbove >= viewportPadding;
    const fitsBelow = desiredTopBelow + tooltipRect.height <= window.innerHeight - viewportPadding;

    let top = desiredTopAbove;
    if (!fitsAbove && fitsBelow) {
      top = desiredTopBelow;
    } else if (!fitsAbove && !fitsBelow) {
      const minTop = viewportPadding;
      const maxTop = Math.max(viewportPadding, window.innerHeight - viewportPadding - tooltipRect.height);
      top = Math.min(maxTop, Math.max(minTop, desiredTopBelow));
    }

    setPosition({ left, top });
  };

  useEffect(() => {
    if (!open) {
      return;
    }

    updatePosition();

    const onWindowChange = () => {
      updatePosition();
    };

    window.addEventListener("resize", onWindowChange);
    window.addEventListener("scroll", onWindowChange, true);

    return () => {
      window.removeEventListener("resize", onWindowChange);
      window.removeEventListener("scroll", onWindowChange, true);
    };
  }, [open]);

  return (
    <span
      class="tooltip-anchor"
      ref={anchorRef}
      onMouseEnter={() => setOpen(true)}
      onMouseLeave={() => setOpen(false)}
      onFocusIn={() => setOpen(true)}
      onFocusOut={() => setOpen(false)}
    >
      <span class="tooltip-target">{children}</span>
      <span
        class={`app-tooltip ${open ? "open" : ""}`}
        role="tooltip"
        ref={tooltipRef}
        style={{ left: `${position.left}px`, top: `${position.top}px` }}
      >
        {text}
      </span>
    </span>
  );
}
