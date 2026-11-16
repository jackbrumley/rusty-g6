import { useEffect, useRef, useState } from "preact/hooks";

/// How long a switch keeps showing the requested value after its write
/// succeeded, waiting for the device's report to catch up.
const REPORT_GRACE_MS = 1000;

/// A switch's request to change its setting. Resolves to `false` when the
/// change failed; any other result counts as success.
export type ToggleRequest = (enabled: boolean) => unknown;

/// What a switch shows, and the handler for its clicks.
///
/// While the user's change is on its way, the switch shows the requested
/// value instead of each report the device sends meanwhile; without this,
/// fast on-off-on clicks made it jump back and forth as the reports for the
/// earlier clicks arrived. Clicks during a write are coalesced: only the
/// latest wish is sent next, so the writes reach the device one at a time
/// and in order. Once the last write is done, the device's value shows
/// again: at once if the write failed, otherwise when the device reports
/// the requested value, or after a short grace if it never does.
export function useRequestedToggle(reported: boolean, request: ToggleRequest) {
  const [requested, setRequested] = useState<boolean | null>(null);
  const [settling, setSettling] = useState(false);
  const sendingRef = useRef(false);
  const queuedRef = useRef<boolean | null>(null);

  const send = async (enabled: boolean) => {
    sendingRef.current = true;
    setSettling(false);
    const succeeded = (await request(enabled)) !== false;
    sendingRef.current = false;

    const queued = queuedRef.current;
    queuedRef.current = null;
    if (queued !== null && queued !== enabled) {
      void send(queued);
    } else if (succeeded) {
      setSettling(true);
    } else {
      setRequested(null);
    }
  };

  const toggle = (enabled: boolean) => {
    setRequested(enabled);
    if (sendingRef.current) {
      queuedRef.current = enabled;
    } else {
      void send(enabled);
    }
  };

  // After the last write: hand back to the device's value once it agrees,
  // or after the grace period if it reports something else.
  useEffect(() => {
    if (!settling) {
      return;
    }
    if (reported === requested) {
      setSettling(false);
      setRequested(null);
      return;
    }
    const timer = setTimeout(() => {
      setSettling(false);
      setRequested(null);
    }, REPORT_GRACE_MS);
    return () => clearTimeout(timer);
  }, [settling, reported, requested]);

  return { shown: requested ?? reported, toggle };
}
