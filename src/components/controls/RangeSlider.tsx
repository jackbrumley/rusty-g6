import { useEffect, useRef, useState } from "preact/hooks";

interface RangeSliderProps {
  value: number;
  min: number;
  max: number;
  step: number;
  disabled?: boolean;
  formatValue: (value: number) => string;
  /// Called once per drag (on release) or key press (on key up), never per notch.
  onCommit: (value: number) => void;
}

/// A slider with its value label. It moves and shows its value live while
/// dragging, but sends the value only when the drag ends, so the device gets
/// one write per change instead of one per notch.
export function RangeSlider({ value, min, max, step, disabled = false, formatValue, onCommit }: RangeSliderProps) {
  const [localValue, setLocalValue] = useState(value);
  const pendingRef = useRef<number | null>(null);
  const draggingRef = useRef(false);
  const valueRef = useRef(value);
  valueRef.current = value;

  // Follow the device's value, except while the user is dragging.
  useEffect(() => {
    if (!draggingRef.current && pendingRef.current === null) {
      setLocalValue(value);
    }
  }, [value]);

  const commit = () => {
    draggingRef.current = false;
    const pending = pendingRef.current;
    pendingRef.current = null;
    if (pending !== null && pending !== valueRef.current) {
      onCommit(pending);
    }
  };

  const startDrag = () => {
    draggingRef.current = true;
    // The pointer can be released outside the slider.
    window.addEventListener("pointerup", commit, { once: true });
    window.addEventListener("pointercancel", commit, { once: true });
  };

  // Clean up a drag that is still open when the slider goes away.
  useEffect(
    () => () => {
      window.removeEventListener("pointerup", commit);
      window.removeEventListener("pointercancel", commit);
    },
    []
  );

  return (
    <>
      <input
        type="range"
        class="slider"
        min={min}
        max={max}
        step={step}
        value={localValue}
        disabled={disabled}
        onPointerDown={startDrag}
        onInput={(event) => {
          const next = parseFloat((event.currentTarget as HTMLInputElement).value);
          setLocalValue(next);
          pendingRef.current = next;
        }}
        onKeyUp={() => {
          if (!draggingRef.current) {
            commit();
          }
        }}
      />
      <span class="slider-value">{formatValue(localValue)}</span>
    </>
  );
}
