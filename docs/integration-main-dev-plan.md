# Main + Dev Integration Plan

## Goal

Unify branch work in a single integration stream with:

- `main` as the runtime and device-sync baseline
- selected `dev` UI and workflow improvements ported in
- backend wiring cleanup so top-level files stay orchestration-focused

## Baseline Decision

- Integration branch: `integration/main-dev-unification`
- Functional source of truth: `main`
- Target merge branch after validation: `main`

## Commit Mapping (dev -> integration)

### High-value commits to port

- `f01a192` `feat: polish app shell and add update-check workflow`
  - Primary UI shell improvements
  - Update-check workflow pattern
  - Backend command modularization reference (`src-tauri/src/app/*` in dev layout)
- `6545441` `feat: add dedicated status tab and modernize app chrome`
  - Status tab layout
  - Top-level tabbed navigation shell
  - Visual refinements for titlebar/tab hierarchy
- `86dcd93` `fix: wire Linux udev hooks into dev packages`
  - Packaging parity to retain Linux post-install/remove behavior

### Utility commits (optional / contextual)

- `c42c7ba` `refactor: align dev workflow with npm and custom window controls`
  - Script orchestration reference only
- `aa8e234` `chore: ignore local reference-systems directory`
  - Local workspace hygiene only
- `f5e55bc` `chore: bump version to 1.0.3 for test installs`
  - Versioning step to redo at release-candidate stage

## Runtime Capabilities to Preserve From main

The following behaviors must remain intact after UI and refactor work:

- listener-thread device event parsing
- `device-update` emission from backend
- frontend event listener refresh path
- synchronize/read behavior on connect
- physical device button state changes reflected in UI
- UI setting changes reflected on device

## Execution Phases

1. Inventory and mapping (this document)
2. Port UI shell + tab architecture
3. Preserve and verify live sync pipeline in integrated UI
4. Refactor backend into modular wiring-oriented structure
5. Verification matrix (typecheck/build + hardware parity)
6. Release-candidate preparation (version bump + packages + upgrade smoke)

## Exit Criteria

- Integration branch compiles and packages successfully
- Manual hardware checks pass for both directions of state sync
- Packaging works as an upgrade over installed stable version
- Branch is ready for PR/merge into `main`
