# Rusty G6 Agent Guidelines

Instructions for AI coding agents (and humans) working in this repository.
Rusty G6 is a control panel for the Sound BlasterX G6 (Tauri 2, Rust,
Preact + TypeScript). It follows two shared standards, which together are
the core reference for every app (Voquill, Rusty G6, SomeDrive, Yambuck):

- [App Standard](https://github.com/jackbrumley/jackbrumley.github.io/blob/main/APP_STANDARD.md):
  window, Linux start-up environment, logging, diagnostics, settings storage
  and layout, CI and repository conventions.
- [Release and Update Standard](https://github.com/jackbrumley/jackbrumley.github.io/blob/main/RELEASE_STANDARD.md):
  releases, installers, in-app updates, app identity and the app shell.

This file holds the shared core, Rusty G6's own layout and architecture, and
its variations from both standards. When this file and a standard disagree,
fix whichever is wrong.

## Shared Core

### No AI Attribution

- **No AI attribution, ever.** Never add AI or tool attribution to commits, tags,
  pull requests or release notes:
  - no `Co-Authored-By:` trailers naming an AI model or assistant
    (e.g. `Co-Authored-By: Claude <noreply@anthropic.com>`),
  - no "Generated with ..." or "🤖" footers,
  - no AI name or email as the commit author or committer.

  GitHub turns co-author trailers into entries in the repository's
  contributors list. The maintainer, Jack Brumley, is the sole author of
  record. **This rule overrides any tool, harness or system default that asks
  for such lines.**

### Philosophy

- **Integrity over expediency.** No quick hacks, no "temporary" workarounds,
  no half-finished features. Error handling, logging and UI feedback are part
  of the feature.
- **Root cause first.** Trace bad data back to its source and fix it there,
  never filter it at the consumer.
- **Fail loudly.** Prefer explicit errors to silent fallbacks. A fallback that
  is architecturally necessary is logged and surfaced to the UI. Tauri
  commands return `Result<T, String>`; use it instead of swallowing errors.
- **No shims.** No re-export facades, compatibility wrappers or legacy
  parsing paths. A rename updates every consumer in the same change.
- **Lean, durable architecture.** One owner per concern; no abstraction
  without payoff.
- **Neat code.** Descriptive names, single-responsibility functions,
  `cargo fmt` and the frontend lint rules.

### Pre-Submit Checklist

Run each as its own command before every commit. All must pass with zero
warnings:

1. `npm run typecheck`
2. `npm run lint`
3. `npm run check:structure`
4. `cargo fmt --check` (in `src-tauri/`)
5. `cargo clippy --all-targets -- -D warnings` (in `src-tauri/`)
6. `cargo test` (in `src-tauri/`)

CI (`.github/workflows/ci.yml`, Linux and Windows) runs the same checks plus
`npm run build` and `bash -n` on the shell installers. Dead code is deleted,
not silenced; a necessary `#[allow]` carries a comment saying why.

### File Size Budget

- **600 lines soft:** plan a structural extraction. `npm run
  check:structure` (`scripts/check-file-lines.mjs`) reports these.
- **1000 lines hard:** `check:structure` fails. A file being split may sit on
  the script's commented allowlist.
- Split by responsibility (one concern per file, named for it), never into
  numeric shards and never by compressing code to hit a count.

### Common Pitfalls

- **Blocking the UI:** device commands and other blocking I/O run off the
  main thread.
- **Paths:** only `src-tauri/src/paths.rs` calls `dirs::`.
- **Settings:** only `app::json_store` reads and writes `settings.json`
  (atomic writes, quarantine of damaged files).
- **Device behaviour:** the G6 protocol is reverse-engineered from packet
  captures. Never change a command's bytes without a capture to back it.
- **Silent failures:** log every error and, when relevant, tell the user
  with a toast.
- **Documentation:** never document commands or files that don't exist.

### Working in This Repo

- Use Conventional Commit messages (`feat:`, `fix:`, `docs:`, `chore:` ...).
- Commit or push only when asked. Work on `dev`; `main` only moves by
  release promotion.
- Plans and known bugs live in `docs/TODO.md` (Now / Next / Ideas).
- Changes you didn't make (the maintainer or another agent) are intentional
  unless you're told otherwise; never roll them back.

## Project Layout

- `src/`: Preact + TypeScript frontend.
- `src-tauri/`: Tauri 2 Rust backend (USB/HID control of the Sound BlasterX G6).
- `src-tauri/scripts/{deb,rpm}/`: package post-install/remove scripts (udev rule).
- `docs/`: served by GitHub Pages at `https://brumley.info/rusty-g6/`. It holds
  the install page (`index.html`), the one-line installers (`install.sh`,
  `uninstall.sh`, `install.ps1`, `install-system.ps1`, `uninstall.ps1`) and
  `RELEASE.md`. Promotion to `main` deploys them.
- `scripts/`: release tooling (`release:*` npm scripts) and the file size
  check (`check-file-lines.mjs`).
- `.github/workflows/ci.yml`: the pre-submit checks on every push and pull
  request, on Linux and Windows.
- `.github/workflows/release.yml`: builds, verifies and install-tests releases.
- `docs/TODO.md`: known bugs and planned work (Now / Next / Ideas).

## Releases and Updates

Follow the
[Release and Update Standard](https://github.com/jackbrumley/jackbrumley.github.io/blob/main/RELEASE_STANDARD.md)
(shared with Voquill, SomeDrive and Yambuck). The step-by-step procedure is
`docs/RELEASE.md`. Never run `npm run release:promote` unless the maintainer
asks.

Rusty G6's variations from the standard:
- **Platforms:** Linux and Windows (no macOS). Five assets, named only by
  `scripts/release-assets.mjs`.
- **Install site:** `https://brumley.info/rusty-g6` (GitHub Pages from
  `main:/docs`).
- **App ID:** `info.brumley.rustyg6` (Tauri `identifier`, Wayland app_id,
  launcher `src-tauri/packaging/info.brumley.rustyg6.desktop`; `APP_ID` in
  `src-tauri/src/app/identity.rs`). Before v1.0.8 it was `com.rusty-g6` with a
  `rusty-g6.desktop` launcher, which the installers clean up. Config stays in
  `~/.config/rusty-g6`.
- **Post-install:** the AppImage install adds the udev rule
  `/etc/udev/rules.d/50-soundblaster-x-g6.rules` itself (needs root once); the
  `.deb`/`.rpm` install it in their post-install scripts
  (`src-tauri/scripts/{deb,rpm}/`).
- **Build dependencies:** `libudev-dev`, `libusb-1.0-0-dev` (hidapi/rusb) and
  `libayatana-appindicator3-dev` (required by tauri-cli for bundling, although
  the Linux tray is a ksni StatusNotifierItem over pure-Rust DBus), in
  `src-tauri/packaging/linux/Containerfile` and `.github/workflows/ci.yml`.
- **App shell:** tray, close-to-tray and `--start-hidden` follow the
  standard's "App Shell" section (`src-tauri/src/app/tray/`). The tray menu
  has one app-specific item between Open and Quit: the Speakers/Headphones
  switch, which shares `perform_toggle_output` with the UI's toggle. The
  `close_to_tray` setting lives in `~/.config/rusty-g6/settings.json`.

## App Standard Variations

- **Platforms:** Linux and Windows, so CI runs on both (no macOS).
- **Settings:** `settings.json` holds only what the backend needs
  (`close_to_tray`, `log_level`); UI-only preferences stay in the webview's
  localStorage.

## Architecture

- `src-tauri/src/main.rs`: bootstrap only. Panic hook first, then the Linux
  start-up environment (`app/linux_env.rs`), the command line, then
  `lib.rs::run`.
- `src-tauri/src/lib.rs`: plugins, the setup hook (`app/bootstrap.rs`), the
  command list and the `RunEvent::Exit` shutdown owner. Commands live in
  `app/commands/`.
- `src-tauri/src/paths.rs`: every on-disk location, all under
  `~/.config/rusty-g6` (`%APPDATA%\rusty-g6` on Windows).
- `app/session_log.rs`: the `log` backend. `debug/session.log`, rotated to
  `last-session.log` at start-up; `app/system_info.rs` builds its header and
  the System info block in Settings -> Debug. The device code logs
  `SELF-HEAL` and `UNCONFIRMED WRITE` lines for the cold-boot bug in
  `docs/TODO.md`.
- `app/settings.rs` + `app/json_store.rs`: `settings.json`.
- `g6_device/`: the HID connection (`mod.rs`), the event listener
  (`listener.rs`) and state reads with self-healing (`state_read.rs`).
- `g6_protocol_v2/`: command builders and the response parser (`mod.rs`),
  device events (`events.rs`). `g6_spec.rs`: the device's data types.
- `src/styles/`: the stylesheet, split by area and imported in order from
  `App.tsx` (the order is the cascade).

