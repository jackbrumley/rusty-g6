import "./App.css";
import { AppShell } from "./components/shell/AppShell";
import { ToastHost } from "./components/toast/ToastHost";
import { useDeviceRuntime } from "./hooks/useDeviceRuntime";
import { useToastManager } from "./hooks/useToastManager";
import { useWindowControls } from "./hooks/useWindowControls";
import { DebugPage } from "./pages/DebugPage";
import { InputPage } from "./pages/InputPage";
import { OutputPage } from "./pages/OutputPage";
import { StatusPage } from "./pages/StatusPage";
import { UiLabPage } from "./pages/UiLabPage";

function App() {
  const { toast, showToast, dismissToast, pauseToast, resumeToast } = useToastManager();
  const {
    connected,
    status,
    settings,
    activeTab,
    appVersion,
    isLinux,
    logSeparatorMessage,
    micBoost,
    permissionError,
    autostartEnabled,
    showExperimental,
    navigate,
    connectDevice,
    disconnectDevice,
    handleSetupUsbPermissions,
    toggleOutput,
    setSbxMode,
    setScoutMode,
    setEffect,
    readDeviceState,
    toggleAutostart,
    toggleExperimental,
    setLogSeparatorMessage,
    clearTerminal,
    handleSetupMicClick,
    setMicrophoneBoost,
  } = useDeviceRuntime({ showToast });

  const {
    handleMinimize,
    handleToggleMaximize,
    handleClose,
    handleTitleBarMouseDown,
    handleTitleBarDoubleClick,
  } = useWindowControls();

  return (
    <AppShell
      activeTab={activeTab}
      onNavigate={navigate}
      onMinimize={handleMinimize}
      onToggleMaximize={handleToggleMaximize}
      onClose={handleClose}
      onTitleBarMouseDown={handleTitleBarMouseDown}
      onTitleBarDoubleClick={handleTitleBarDoubleClick}
    >
      <main class="container">
        {activeTab === "status" && (
          <StatusPage
            connected={connected}
            status={status}
            appVersion={appVersion}
            permissionError={permissionError}
            onConnect={() => connectDevice(false)}
            onDisconnect={disconnectDevice}
            onSetupPermissions={handleSetupUsbPermissions}
          />
        )}

        {activeTab === "output" && connected && settings && (
          <OutputPage
            settings={settings}
            onToggleOutput={toggleOutput}
            onSetScoutMode={setScoutMode}
            onSetSbxMode={setSbxMode}
            onSetEffect={setEffect}
          />
        )}

        {activeTab === "input" && connected && settings && (
          <InputPage
            settings={settings}
            showExperimental={showExperimental}
            micBoost={micBoost}
            isLinux={isLinux}
            onSetupMicClick={handleSetupMicClick}
            onSetMicrophoneBoost={setMicrophoneBoost}
          />
        )}

        {activeTab === "debug" && (
          <DebugPage
            appVersion={appVersion}
            settings={settings}
            autostartEnabled={autostartEnabled}
            showExperimental={showExperimental}
            logSeparatorMessage={logSeparatorMessage}
            onNavigateUiLab={() => navigate("ui-lab")}
            onReadDeviceState={readDeviceState}
            onToggleAutostart={toggleAutostart}
            onToggleExperimental={toggleExperimental}
            onSetLogSeparatorMessage={setLogSeparatorMessage}
            onClearTerminal={clearTerminal}
          />
        )}

        {activeTab === "ui-lab" && (
          <UiLabPage
            onBack={() => navigate("debug")}
            onToast={(type, message, durationMs) => showToast(message, type, durationMs)}
          />
        )}

        {(activeTab === "output" || activeTab === "input") && !connected && (
          <div class="info-panel">
            <p>Connect your SoundBlaster X G6 from the Status tab to begin.</p>
            <p class="info-note">This page only shows controls once a device session is active.</p>
            <button class="btn-compact btn-secondary" onClick={() => navigate("status")}>
              Go to Status
            </button>
          </div>
        )}
      </main>

      <ToastHost toast={toast} onDismiss={dismissToast} onPause={pauseToast} onResume={resumeToast} />
    </AppShell>
  );
}

export default App;
