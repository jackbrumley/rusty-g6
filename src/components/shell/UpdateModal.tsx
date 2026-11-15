import type { Updates } from "../../hooks/useUpdates";

/// The update dialog: shown when a check finds a newer version.
export function UpdateModal({ updates }: { updates: Updates }) {
  const { updateResult, updateError, lastCheckedLabel, installingUpdate } = updates;
  const close = () => updates.setShowUpdateModal(false);

  return (
    <div class="modal-overlay" onClick={close}>
      <section class="update-modal" onClick={(event) => event.stopPropagation()}>
        <h2>{updates.updateAvailable ? "Update Available" : "Rusty G6 is Up to Date"}</h2>
        <p>
          {updateResult?.updateAvailable
            ? `A newer Rusty G6 version is available. Current: v${updateResult.currentVersion} -> Latest: v${updateResult.latestVersion}.`
            : "You are already running the latest available version."}
        </p>
        {updateError && <p class="info-note">Last check error: {updateError}</p>}
        <p class="update-last-checked">Last checked: {lastCheckedLabel}</p>
        <div class="update-modal-actions">
          <button class="btn-compact" onClick={close}>
            Later
          </button>
          {updates.updateAvailable ? (
            <>
              <button class="btn-compact" onClick={updates.openLatestReleasePage}>
                View Release Notes
              </button>
              <button class="btn-compact" onClick={updates.installUpdate} disabled={installingUpdate}>
                {installingUpdate ? "Updating..." : "Update Now"}
              </button>
            </>
          ) : (
            <button class="btn-compact" onClick={close}>
              Close
            </button>
          )}
        </div>
      </section>
    </div>
  );
}
