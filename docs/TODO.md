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

Leads to check:
- The HID handle opened at boot may be stale or the wrong interface (device
  enumerated before udev rules / audio stack were ready), so writes partly work
  but reads return nothing useful.
- "Read successfully" may only mean the request was sent, not that responses
  were parsed and applied to settings. Confirm the response path updates state
  and emits to the UI.
- The listener thread that confirms changes ("state will be updated by listener
  when device confirms") may have died or never started, which would explain
  one click going through and the UI never updating. Check for a held lock or
  pending-change flag that never clears.
- Recent change `7a1f3a8 feat: switch to event-driven device reconnect` touched
  this path (`bootstrap.rs`, `g6_device.rs`, `useDeviceRuntime.ts`): check
  whether the initial connect at startup (no hotplug event) is handled.
- Fix idea: when a read returns no valid state, or a write is not confirmed
  within a timeout, close and reopen the device automatically (a software
  "replug").

## Distribution

- [ ] One-line install: `install.sh` (apt/dnf/AppImage; the AppImage path must
      also install the udev rule) and `install.ps1` (per-user setup.exe), served
      from `docs/` at `https://brumley.info/rusty-g6/`.
- [ ] One-click in-app update: port Voquill's `install_update` (re-run the
      installer, exit, relaunch; relaunch old version with `--update-failed`).
- [ ] Install page at `docs/index.html` and enable GitHub Pages from `main:/docs`.
- [ ] Release workflow in GitHub Actions that builds and verifies all five
      assets, so the installer's expected file names can't drift.
