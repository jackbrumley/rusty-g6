import { SelectControl } from "../components/controls/SelectControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { SettingsSection } from "../components/ui/SettingsSection";
import { TabPageHeader } from "../components/ui/TabPageHeader";
import type { DeviceSettingsActions } from "../hooks/useDeviceSettings";
import { DIGITAL_FILTERS, type G6Settings } from "../types/g6";

interface DevicePageProps {
  settings: G6Settings | null;
  actions: DeviceSettingsActions;
  onGoToStatus: () => void;
}

export function DevicePage({ settings, actions, onGoToStatus }: DevicePageProps) {
  return (
    <>
      <TabPageHeader title="Device" subtitle="Hardware settings of the G6 itself." />
      <DeviceRequired connected={settings !== null} onGoToStatus={onGoToStatus}>
        {settings && (
          <SettingsSection title="DAC">
            <SelectControl
              label="Filter"
              tooltip="The DAC's digital reconstruction filter. Differences are subtle; NOS skips oversampling entirely."
              value={settings.digital_filter}
              options={DIGITAL_FILTERS}
              onChange={actions.setDacFilter}
            />
          </SettingsSection>
        )}
      </DeviceRequired>
    </>
  );
}
