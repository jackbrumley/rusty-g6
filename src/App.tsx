import "./App.css";
import { AppShell } from "./components/shell/AppShell";
import { ToastHost } from "./components/toast/ToastHost";
import { useDeviceRuntime } from "./hooks/useDeviceRuntime";
import { useUiInteractionLogger } from "./hooks/useUiInteractionLogger";
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
    checkingUpdates,
    updateResult,
    showUpdateModal,
    updateError,
    lastCheckedLabel,
    retryInSeconds,
    logSeparatorMessage,
    micBoost,
    permissionError,
    autostartEnabled,
    showExperimental,
    navigate,
    resetConnection,
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
    copySessionLog,
    openSessionLog,
    checkForUpdates,
    setShowUpdateModal,
    openLatestReleasePage,
    handleSetupMicClick,
    setMicrophoneBoost,
  } = useDeviceRuntime({ showToast });

  useUiInteractionLogger();

  const {
    isMaximized,
    handleMinimize,
    handleToggleMaximize,
    handleClose,
    handleTitleBarMouseDown,
    handleTitleBarDoubleClick,
  } = useWindowControls();

  return (
    <AppShell
      activeTab={activeTab}
      isMaximized={isMaximized}
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
            settings={settings}
            isLinux={isLinux}
            retryInSeconds={retryInSeconds}
            onReadDeviceState={readDeviceState}
            onSetupPermissions={handleSetupUsbPermissions}
            showUpdatePill={Boolean(updateResult?.updateAvailable)}
            onOpenUpdateModal={() => setShowUpdateModal(true)}
          />
        )}

        {activeTab === "output" && (
          <OutputPage
            connected={connected}
            settings={settings}
            onToggleOutput={toggleOutput}
            onSetScoutMode={setScoutMode}
            onSetSbxMode={setSbxMode}
            onSetEffect={setEffect}
            onGoToStatus={() => navigate("status")}
          />
        )}

        {activeTab === "input" && (
          <InputPage
            connected={connected}
            settings={settings}
            showExperimental={showExperimental}
            micBoost={micBoost}
            isLinux={isLinux}
            onSetupMicClick={handleSetupMicClick}
            onSetMicrophoneBoost={setMicrophoneBoost}
            onGoToStatus={() => navigate("status")}
          />
        )}

        {activeTab === "debug" && (
          <DebugPage
            settings={settings}
            appVersion={appVersion}
            autostartEnabled={autostartEnabled}
            showExperimental={showExperimental}
            checkingUpdates={checkingUpdates}
            updateAvailable={Boolean(updateResult?.updateAvailable)}
            latestVersion={updateResult?.latestVersion ?? null}
            lastCheckedLabel={lastCheckedLabel}
            logSeparatorMessage={logSeparatorMessage}
            onNavigateUiLab={() => navigate("ui-lab")}
            onReadDeviceState={readDeviceState}
            onResetConnection={resetConnection}
            onToggleAutostart={toggleAutostart}
            onToggleExperimental={toggleExperimental}
            onCheckForUpdates={() => checkForUpdates(true)}
            onOpenUpdateModal={() => setShowUpdateModal(true)}
            onSetLogSeparatorMessage={setLogSeparatorMessage}
            onClearTerminal={clearTerminal}
            onCopySessionLog={copySessionLog}
            onOpenSessionLog={openSessionLog}
          />
        )}

        {activeTab === "ui-lab" && (
          <UiLabPage
            onBack={() => navigate("debug")}
            onToast={(type, message, durationMs) => showToast(message, type, durationMs)}
          />
        )}

      </main>

      {showUpdateModal && (
        <div class="modal-overlay" onClick={() => setShowUpdateModal(false)}>
          <section class="update-modal" onClick={(event) => event.stopPropagation()}>
            <h2>{updateResult?.updateAvailable ? "Update Available" : "Rusty G6 is Up to Date"}</h2>
            <p>
              {updateResult?.updateAvailable
                ? `A newer Rusty G6 version is available. Current: v${updateResult.currentVersion} -> Latest: v${updateResult.latestVersion}.`
                : "You are already running the latest available version."}
            </p>
            {updateError && <p class="info-note">Last check error: {updateError}</p>}
            <p class="update-last-checked">Last checked: {lastCheckedLabel}</p>
            <p class="info-note">Updates are currently installed manually from the GitHub release page.</p>
            <div class="update-modal-actions">
              <button class="btn-compact btn-secondary" onClick={() => setShowUpdateModal(false)}>
                Later
              </button>
              {updateResult?.updateAvailable ? (
                <button class="btn-compact" onClick={openLatestReleasePage}>
                  Open Release Page
                </button>
              ) : (
                <button class="btn-compact" onClick={() => setShowUpdateModal(false)}>
                  Close
                </button>
              )}
            </div>
          </section>
        </div>
      )}

      <ToastHost toast={toast} onDismiss={dismissToast} onPause={pauseToast} onResume={resumeToast} />
    </AppShell>
  );
}

export default App;
