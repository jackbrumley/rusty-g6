# Integration Validation Matrix

## Automated Checks (Completed)

- Frontend build and typecheck:
  - Command: `npm run build`
  - Working directory: `src/`
  - Result: pass
- Backend compile check:
  - Command: `cargo check`
  - Working directory: `src/src-tauri/`
  - Result: pass (warnings only)

## Manual Hardware Checks (Required)

Run these with a connected Sound Blaster X G6 before merging to `main`.

1. Connect path
   - Start app and verify Status tab reflects connected state.
   - Verify output/effects tabs render controls only after active connection.

2. Device -> UI synchronization
   - Press physical device buttons (SBX/Scout/output) directly on hardware.
   - Confirm UI updates without requiring full app restart.

3. UI -> Device synchronization
   - Toggle output/effects in app.
   - Confirm corresponding change is reflected on the hardware behavior/state.

4. Disconnect/reconnect resilience
   - Unplug/replug device while app remains open.
   - Confirm UI status and controls recover to accurate state.

5. Linux permission flow
   - Validate permission failure path surfaces actionable status.
   - Verify `Fix Permissions` action succeeds when required.

## Release Gate

Integration branch is release-candidate ready when:

- automated checks pass,
- all manual hardware checks pass,
- package build and upgrade smoke test succeed.
