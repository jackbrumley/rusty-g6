# Release Process

Rusty G6 follows the
[Release and Update Standard](https://github.com/jackbrumley/jackbrumley.github.io/blob/main/RELEASE_STANDARD.md),
the same process as Voquill, SomeDrive and Yambuck. This is Rusty G6's copy of
the procedure; the standard has the reasoning and the full installer and
updater contracts.

Release identity (tag, title, asset names, download links) is consumed by
`docs/install.sh`, `docs/install.ps1` and the in-app updater, so a format
mistake breaks updates and one-line installs for every user.

## Format

| Where | Format | Example |
|---|---|---|
| `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` | `X.Y.Z` | `1.0.8` |
| Version bump commit | `chore: bump version to X.Y.Z` | |
| Git tag and release title | `vX.Y.Z` | `v1.0.8` |
| First line of the release notes | `# Rusty G6 vX.Y.Z` | |
| Assets | `rusty-g6-X.Y.Z-<os>-x64[-setup].<ext>` | `rusty-g6-1.0.8-linux-x64.deb` |

A complete release carries five assets, all named by `scripts/release-assets.mjs`:
- `rusty-g6-X.Y.Z-windows-x64-setup.exe`
- `rusty-g6-X.Y.Z-windows-x64.msi`
- `rusty-g6-X.Y.Z-linux-x64.deb`
- `rusty-g6-X.Y.Z-linux-x64.rpm`
- `rusty-g6-X.Y.Z-linux-x64.AppImage`

## Branches

Development happens on `dev`; releases are tagged on `dev`. Promotion
fast-forwards `main` to the release tag, so `main` always equals the latest
release. GitHub Pages serves `brumley.info/rusty-g6/` (install page and
installer scripts) from `main:/docs`, so promotion also deploys installer
changes. Never commit to `main` directly.

## 1. Prepare and Tag

```bash
npm run release:prepare -- X.Y.Z
git push origin dev vX.Y.Z
```

Pushing the tag starts the **Release** workflow
(`.github/workflows/release.yml`):
1. **Prepare** checks the tag matches `tauri.conf.json` and creates the
   pre-release with the notes skeleton.
2. **Build** runs Linux (inside the Ubuntu 22.04 container,
   `npm run release:linux`) and Windows in parallel and uploads the assets.
3. **Verify** fails unless the release carries exactly the five assets, each
   with a SHA-256 digest.
4. **Install test** installs the pre-release with the installer scripts from
   the tag: `.deb`, AppImage and uninstall on Linux; MSI and setup.exe on
   Windows.

Follow it with:
```bash
gh run watch "$(gh run list --workflow release.yml --limit 1 --json databaseId --jq '.[0].databaseId')" --exit-status
```

## 2. Write the Release Notes

```bash
npm run --silent release:notes > release-notes.md   # commits since the last tag print to the terminal
gh release edit vX.Y.Z --notes-file release-notes.md
```

Replace every `TODO`. Structure: `# Rusty G6 vX.Y.Z`, one summary paragraph,
themed `###` sections of `- Label: sentence` bullets separated by `---`, then
the generated Downloads block (never edited). Write for users, not developers.

## 3. Promote

```bash
npm run release:promote
```

Refuses unless all five assets have digests and the notes contain no `TODO`.
It marks the release latest and fast-forwards `main` to the tag. From then on
the in-app updater and the one-line installers serve the new version.

## 4. Verify

- An older installed version offers the update and **Update Now** installs and
  relaunches it.
- `curl -sf https://brumley.info/rusty-g6/install.sh | bash` installs the new
  version.

## When Something Fails

- **Transient failure** (network, runner): `gh run rerun <run-id> --failed`.
- **Code failure:** nothing was promoted, so nobody received it. Fix on `dev`,
  then:
  ```bash
  gh release delete vX.Y.Z --cleanup-tag --yes
  git tag -d vX.Y.Z
  git tag vX.Y.Z
  git push origin dev vX.Y.Z
  ```
- **Dry run** of every build without a release:
  `gh workflow run release.yml --ref dev`.

## Building Locally (Fallback)

- Linux (needs `podman`): `npm run release:linux`, then
  `node scripts/package-release.mjs`.
- Windows: `npm run tauri:build`, then `node scripts/package-release.mjs`.
- Upload with `gh release upload vX.Y.Z release-artifacts/*`.

## Agent Checklist

- [ ] Version prepared with `npm run release:prepare`; tag is `vX.Y.Z`.
- [ ] Release workflow green: build, verify and install test.
- [ ] Notes from `npm run release:notes`; no `TODO`; Downloads block untouched.
- [ ] Assets only from the workflow or `package-release.mjs`.
- [ ] `npm run release:promote` only when the maintainer asks.
- [ ] No AI attribution in commits, tags or notes.
