import { useEffect, useState } from "preact/hooks";
import { Tooltip } from "../ui/Tooltip";

interface EffectControlProps {
  name: string;
  enabled: boolean;
  level: number;
  onToggle: (enabled: boolean) => void;
  onLevelChange: (level: number) => void;
  disabled?: boolean;
  tooltip?: string;
}

export function EffectControl({
  name,
  enabled,
  level,
  onToggle,
  onLevelChange,
  disabled = false,
  tooltip,
}: EffectControlProps) {
  // The slider shows its position while dragging; the level is sent on release.
  const [localLevel, setLocalLevel] = useState(level);

  useEffect(() => {
    setLocalLevel(level);
  }, [level]);

  const sliderLevel = (e: Event) => parseInt((e.currentTarget as HTMLInputElement).value, 10);

  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <span class="control-label">
        {tooltip ? <Tooltip text={tooltip}>{name}</Tooltip> : name}
      </span>
      <label class="toggle-switch">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => onToggle((e.currentTarget as HTMLInputElement).checked)}
          disabled={disabled}
        />
        <span class="toggle-slider"></span>
      </label>
      <input
        type="range"
        min="0"
        max="100"
        value={localLevel}
        onInput={(e) => setLocalLevel(sliderLevel(e))}
        onChange={(e) => onLevelChange(sliderLevel(e))}
        disabled={!enabled || disabled}
        class="slider"
      />
      <span class="slider-value">{localLevel}</span>
    </div>
  );
}
