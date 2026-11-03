# TODO

## Bugs

### Device "connected" after reboot but controls are dead until replug

Reported 2026-11-03. Happens often after a reboot; unplugging and replugging
the G6 fixes it.

- Status shows **Connected**, but every toggle is off: the device state was
  never actually loaded into the UI.
- **Read Device State** reports "read successfully", yet the toggles still do
  not turn on.
- Toggling output (Speakers/Headphones): the device relay audibly clicks
  **once**, so the first write reaches the hardware, but the UI does not
  update to Headphones. Further clicks do nothing at all, as if the
  device/session is locked up.

**Findings (session log and kernel log of the 2026-11-03 boot):**
- The app opened interface 4 (`/dev/hidraw3`) about 20 s after boot. The node
  stayed the same until the manual replug at 18:31, so the handle was not
  stale and was the right interface.
- Every state query got **no packet at all** (`Received 0 packet(s)` for all
  14 queries, at 18:10 and again at 18:29), and the listener saw no events
  for 20 minutes. Writes succeeded. Interface 4 has only an interrupt IN
  endpoint (`ep_85`), so writes go over the control pipe (SET_REPORT) and all
  answers come back on interrupt IN: the control path worked, the IN path was
  silent.
- Right after the replug, every query was answered within 1 ms on a freshly
  opened handle.
- Every cold boot in the journal (Oct 16, 18, 28, Nov 2, Nov 3) logs 12
  `__uac_clock_find_source(): selector reported illegal value` errors for the
  G6. No later re-enumeration (replug or resume) does. The G6's firmware is
  not fully up when it is enumerated at power-on. Several of those boots are
  followed by an 8-10 s disconnect/reconnect, which is a manual replug.
- "Further clicks do nothing" was an app bug on top of that: the toggle target
  came from the cached output, which never left its default (Headphones), so
  every click re-sent "switch to Speakers". The first click switched the
  relay. Later clicks asked for the output it already had.
- "Read successfully" was wrong too: the read reported success whatever it
  parsed. Its "Refreshing device connection" step only rebuilt the HID
  enumerator, not the open handle. The UI also gave up on the start-up read
  after 1 s while the backend spent 7 s waiting on 14 unanswered queries.
  Device commands ran on the main thread, which froze the window meanwhile.

**Fixed (dev, after v1.0.8):**
- A state read fails (`DeviceUnresponsive`) unless it parsed at least one
  response. It stops after 2 silent queries in a row instead of waiting out
  all 14. Read Device State now reports the failure.
- Self-healing on an unresponsive read, logged with `SELF-HEAL`: close and
  reopen the HID handle, then read again. If the G6 is still silent (Linux),
  reset it on the USB bus (libusb `USBDEVFS_RESET`, the re-enumeration a
  replug causes), reconnect and read again. The USB reset runs at most once
  until a read succeeds, so a dead device is never reset in a loop. A
  remaining failure says to replug.
- Writes are checked for an acknowledgement. An unacknowledged output switch
  re-reads the device (healing it if needed) to learn whether it took effect
  (`UNCONFIRMED WRITE` in the log).
- Toggling output reads the device first unless its output was confirmed by
  a read or a change event since the handle was opened.
- The event listener starts once per app run, so reconnects no longer stack
  up extra reader threads on the same handle.
- Device commands run off the main thread, and the UI waits up to 15 s for a
  read. The backend emits `device-update` when a read finishes.

**Still to confirm on real hardware after a cold boot** (with autostart):
- Does the first read log `SELF-HEAL`, and which step brings the G6 back:
  reopening the handle, or only the USB reset? Search the session log for
  `SELF-HEAL`.
- Does the USB reset interrupt audio noticeably, and does the G6 keep its
  output and effect settings across it?
- If even the USB reset does not help, try a power-cycle-like fix next, e.g.
  toggling `/sys/bus/usb/devices/<port>/authorized` (needs root via the udev
  rule), or delay the first open until the audio stack has configured the
  device.
- Windows has no USB reset step (the HID driver owns the device). Check
  whether the bug happens there at all.

## Distribution

- [ ] One-line install: `install.sh` (apt/dnf/AppImage; the AppImage path must
      also install the udev rule) and `install.ps1` (per-user setup.exe), served
      from `docs/` at `https://brumley.info/rusty-g6/`.
- [ ] One-click in-app update: port Voquill's `install_update` (re-run the
      installer, exit, relaunch; relaunch old version with `--update-failed`).
- [ ] Install page at `docs/index.html` and enable GitHub Pages from `main:/docs`.
- [ ] Release workflow in GitHub Actions that builds and verifies all five
      assets, so the installer's expected file names can't drift.
