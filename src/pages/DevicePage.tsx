import { ColorControl } from "../components/controls/ColorControl";
import { SelectControl } from "../components/controls/SelectControl";
import { ToggleControl } from "../components/controls/ToggleControl";
import { DeviceRequired } from "../components/ui/DeviceRequired";
import { SettingsPage } from "../components/ui/SettingsPage";
import { SettingsSection } from "../components/ui/SettingsSection";
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
      <DeviceRequired connected={settings !== null} onGoToStatus={onGoToStatus}>
        {settings && (
          <SettingsPage>
            <SettingsSection title="DAC">
            <SelectControl
              label="Filter"
              tooltip="The DAC's digital reconstruction filter. Differences are subtle; NOS skips oversampling entirely."
              value={settings.digital_filter}
              options={DIGITAL_FILTERS}
              onChange={actions.setDacFilter}
            />
            </SettingsSection>

            <SettingsSection title="Lighting">
              <ToggleControl
                label="Lighting"
                tooltip="The G6's logo light."
                checked={settings.lighting.enabled}
                onChange={actions.setLightingEnabled}
              />
              <ColorControl
                label="Colour"
                color={settings.lighting.color}
                onChange={actions.setLightingColor}
                disabled={!settings.lighting.enabled}
              />
            </SettingsSection>
          </SettingsPage>
        )}
      </DeviceRequired>
    </>
  );
}
