import { ControlLabel } from "./ControlLabel";
import type { RgbColor } from "../../types/g6";

const SWATCHES: RgbColor[] = [
  { red: 255, green: 0, blue: 0 },
  { red: 255, green: 96, blue: 0 },
  { red: 255, green: 200, blue: 0 },
  { red: 0, green: 255, blue: 64 },
  { red: 0, green: 200, blue: 255 },
  { red: 0, green: 64, blue: 255 },
  { red: 160, green: 0, blue: 255 },
  { red: 255, green: 255, blue: 255 },
];

const toHex = ({ red, green, blue }: RgbColor) =>
  `#${[red, green, blue].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;

const fromHex = (hex: string): RgbColor => ({
  red: parseInt(hex.slice(1, 3), 16),
  green: parseInt(hex.slice(3, 5), 16),
  blue: parseInt(hex.slice(5, 7), 16),
});

const sameColor = (a: RgbColor, b: RgbColor) => a.red === b.red && a.green === b.green && a.blue === b.blue;

interface ColorControlProps {
  label: string;
  tooltip?: string;
  /// One line under the label saying what the setting does.
  description?: string;
  color: RgbColor;
  onChange: (color: RgbColor) => void;
  disabled?: boolean;
}

/// A settings row with colour swatches and a custom colour picker.
export function ColorControl({ label, tooltip, description, color, onChange, disabled = false }: ColorControlProps) {
  const isCustom = !SWATCHES.some((swatch) => sameColor(swatch, color));

  return (
    <div class={`control-row ${disabled ? "disabled" : ""}`}>
      <ControlLabel label={label} tooltip={tooltip} description={description} />
      <span class="color-swatches">
        {SWATCHES.map((swatch) => (
          <button
            key={toHex(swatch)}
            type="button"
            class={`color-swatch ${sameColor(swatch, color) ? "selected" : ""}`}
            style={{ background: toHex(swatch) }}
            aria-label={`Colour ${toHex(swatch)}`}
            onClick={() => onChange(swatch)}
          />
        ))}
        <label class={`color-swatch color-swatch-custom ${isCustom ? "selected" : ""}`} title="Custom colour">
          <input
            type="color"
            value={toHex(color)}
            onChange={(event) => onChange(fromHex((event.currentTarget as HTMLInputElement).value))}
          />
        </label>
      </span>
    </div>
  );
}
