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
            <SettingsSection title="Playback">
              <ToggleControl
                label="Direct mode"
                description="Skips SBX and the EQ for untouched, bit-perfect sound. The G6 reconnects when you switch it."
                checked={settings.direct_mode}
                onChange={(enabled) => actions.setDeviceFlag("direct_mode", enabled)}
              />
              <ToggleControl
                label="S/PDIF-out direct"
                description="Sends untouched digital sound to the optical out, e.g. for an external DAC or receiver. Speakers only."
                checked={settings.spdif_out_direct}
                onChange={(enabled) => actions.setDeviceFlag("spdif_out_direct", enabled)}
              />
            </SettingsSection>

            <SettingsSection title="DAC">
            <SelectControl
              label="Filter"
              description="How the DAC smooths the digital signal. Subtle; Natural is a good default."
              value={settings.digital_filter}
              options={DIGITAL_FILTERS}
              onChange={actions.setDacFilter}
            />
            </SettingsSection>

            <SettingsSection title="Lighting">
              <ToggleControl
                label="Lighting"
                description="The G6's logo light."
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
