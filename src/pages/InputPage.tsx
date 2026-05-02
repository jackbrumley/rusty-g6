import { SliderControl } from "../components/controls/SliderControl";
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
    <section class="input-section compact">
      <div class="section-line">
        <span class="section-label">Input:</span>
        {showExperimental ? (
          <button
            onClick={onSetupMicClick}
            class="btn-compact"
            title={isLinux ? "Configure ALSA mixer for microphone input" : undefined}
          >
            Setup Mic
          </button>
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
