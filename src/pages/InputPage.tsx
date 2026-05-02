import { SliderControl } from "../components/controls/SliderControl";
import { Tooltip } from "../components/ui/Tooltip";
import type { G6Settings } from "../types/g6";

interface InputPageProps {
  settings: G6Settings;
  showExperimental: boolean;
  micBoost: number;
  isLinux: boolean;
  onSetupMicClick: () => void;
  onSetMicrophoneBoost: (dbValue: number) => void;
}

export function InputPage({
  settings,
  showExperimental,
  micBoost,
  isLinux,
  onSetupMicClick,
  onSetMicrophoneBoost,
}: InputPageProps) {
  return (
    <section class="input-section compact surface-card">
      <div class="section-line">
        <span class="section-label">Input:</span>
        {showExperimental ? (
          <Tooltip
            text={
              isLinux
                ? "Configures ALSA capture routing for G6 microphone input."
                : "Microphone setup is automatic on Windows."
            }
          >
            <button onClick={onSetupMicClick} class="btn-compact">
              Setup Mic
            </button>
          </Tooltip>
        ) : (
          <span class="info-note" style={{ fontSize: "0.75rem" }}>
            Experimental on Linux. Recommended to use motherboard input for reliability.
          </span>
        )}
      </div>

      {showExperimental && settings && (
        <SliderControl
          label="Mic Boost:"
          value={micBoost}
          tooltip="Adjusts microphone input gain in 10 dB steps. Higher values increase sensitivity and background pickup."
          min={0}
          max={30}
          step={10}
          onChange={onSetMicrophoneBoost}
          formatValue={(val) => (val > 0 ? `+${val}dB` : `${val}dB`)}
        />
      )}
    </section>
  );
}
