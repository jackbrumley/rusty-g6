import "@fontsource-variable/inter";
import "./styles/tokens.css";
import "./styles/shell.css";
import "./styles/sections.css";
import "./styles/controls.css";
import "./styles/responsive.css";
import { AppShell } from "./components/shell/AppShell";
import { ToastHost } from "./components/toast/ToastHost";
import { UpdateModal } from "./components/shell/UpdateModal";
import { useAppPreferences } from "./hooks/useAppPreferences";
import { useDeviceConnection } from "./hooks/useDeviceConnection";
import { useDeviceSettings } from "./hooks/useDeviceSettings";
import { useRecordingSource } from "./hooks/useRecordingSource";
import { useRoute } from "./hooks/useRoute";
import { useUiInteractionLogger } from "./hooks/useUiInteractionLogger";
import { useToastManager } from "./hooks/useToastManager";
import { useUpdates } from "./hooks/useUpdates";
import { useWindowControls } from "./hooks/useWindowControls";
import { DebugPage } from "./pages/DebugPage";
import { DevicePage } from "./pages/DevicePage";
import { EqPage } from "./pages/EqPage";
import { MicPage } from "./pages/MicPage";
import { SoundPage } from "./pages/SoundPage";
import { StatusPage } from "./pages/StatusPage";
import { UiLabPage } from "./pages/UiLabPage";

function App() {
  const { toast, showToast, dismissToast, pauseToast, resumeToast } = useToastManager();
  const { activeTab, navigate } = useRoute();
  const connection = useDeviceConnection(showToast);
  const deviceSettings = useDeviceSettings({ showToast });
  const { recordingSource, setRecordingSource } = useRecordingSource({ showToast });
  const preferences = useAppPreferences(showToast);
  const updates = useUpdates(showToast);
  const windowControls = useWindowControls(showToast);
  useUiInteractionLogger();

  const { connected, settings } = connection;
  const goToStatus = () => navigate("status");

  return (
    <AppShell
      activeTab={activeTab}
      isMaximized={windowControls.isMaximized}
      onNavigate={navigate}
      onMinimize={windowControls.handleMinimize}
      onToggleMaximize={windowControls.handleToggleMaximize}
      onClose={windowControls.handleClose}
      onTitleBarMouseDown={windowControls.handleTitleBarMouseDown}
      onTitleBarDoubleClick={windowControls.handleTitleBarDoubleClick}
      onResizeMouseDown={windowControls.handleResizeMouseDown}
    >
      <main class="container">
        {activeTab === "status" && (
          <StatusPage
            connected={connected}
            status={connection.status}
            appVersion={preferences.appVersion}
            permissionError={connection.permissionError}
            settings={settings}
            onReadDeviceState={connection.readDeviceState}
            onSetupPermissions={connection.setupUsbPermissions}
            showUpdatePill={updates.updateAvailable}
            onOpenUpdateModal={() => updates.setShowUpdateModal(true)}
          />
        )}

        {activeTab === "sound" && (
          <SoundPage settings={settings} actions={deviceSettings} onGoToStatus={goToStatus} />
        )}

        {activeTab === "eq" && (
          <EqPage settings={settings} actions={deviceSettings} onGoToStatus={goToStatus} />
        )}

        {activeTab === "mic" && (
          <MicPage
            settings={settings}
            recordingSource={recordingSource}
            onSetRecordingSource={setRecordingSource}
            actions={deviceSettings}
            onGoToStatus={goToStatus}
          />
        )}

        {activeTab === "device" && (
          <DevicePage settings={settings} actions={deviceSettings} onGoToStatus={goToStatus} />
        )}

        {activeTab === "debug" && (
          <DebugPage
            preferences={preferences}
            updates={updates}
            connection={connection}
            onNavigateUiLab={() => navigate("ui-lab")}
            showToast={showToast}
          />
        )}

        {activeTab === "ui-lab" && (
          <UiLabPage
            onBack={() => navigate("debug")}
            onToast={(type, message, durationMs) => showToast(message, type, durationMs)}
          />
        )}
      </main>

      {updates.showUpdateModal && <UpdateModal updates={updates} />}

      <ToastHost toast={toast} onDismiss={dismissToast} onPause={pauseToast} onResume={resumeToast} />
    </AppShell>
  );
}

export default App;
