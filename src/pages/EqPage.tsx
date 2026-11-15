import { EqualizerControl } from "../components/controls/EqualizerControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import type { G6Settings } from "../types/g6";

interface EqPageProps {
  settings: G6Settings | null;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function EqPage({ settings, actions, onGoToStatus }: EqPageProps) {
  return (
    <>
      <TabPageHeader
        title="Equalizer"
        subtitle="The G6's 10-band EQ, stored on the device for the current output."
      />
      <DeviceRequired connected={settings !== null} onGoToStatus={onGoToStatus}>
        {settings && (
          <EqualizerControl
            equalizer={settings.equalizer}
            onSetEnabled={actions.setEqEnabled}
            onSetPreamp={actions.setEqPreamp}
            onSetBand={actions.setEqBand}
            onSetBands={actions.setEqBands}
          />
        )}
      </DeviceRequired>
    </>
  );
}
