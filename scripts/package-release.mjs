#!/usr/bin/env node

// Copies this platform's build outputs into release-artifacts/ under their
// standard release names (from release-assets.mjs). The release workflow
// uploads the folder's whole contents.
//
//   node scripts/package-release.mjs

import { copyFileSync, mkdirSync, existsSync, readdirSync, rmSync } from "node:fs";
import { resolve, dirname, basename } from "node:path";
import { fileURLToPath } from "node:url";
import { ASSET_NAMES, VERSION, assertPlainVersion } from "./release-assets.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SRC_TAURI = resolve(ROOT, "src-tauri");
const OUT_DIR = resolve(ROOT, "release-artifacts");

// Tauri embeds the version in the artifact filename (e.g. rusty-g6_1.0.7_amd64.deb).
// Refuse to package an artifact whose embedded version differs from the
// configured release version, so a stale build is never mislabeled.
function findForVersion(dir, pattern) {
  if (!existsSync(dir)) return null;
  for (const entry of readdirSync(dir)) {
    if (!pattern.test(entry)) continue;
    if (!entry.includes(`_${VERSION}_`) && !entry.includes(`-${VERSION}-`)) {
      console.log(`  SKIP ${entry} (embedded version does not match v${VERSION})`);
      continue;
    }
    return resolve(dir, entry);
  }
  return null;
}

// No checksum files are produced: GitHub computes a SHA-256 digest for every
// uploaded release asset, and the installers verify against that digest.
function copyArtifact(source, targetName) {
  console.log(`  ${basename(source)}  →  ${targetName}`);
  copyFileSync(source, resolve(OUT_DIR, targetName));
}

// Linux releases come only from the containerized build (npm run release:linux),
// never from a host `tauri:build`, so the glibc floor is fixed by the container
// base image rather than by whichever distro the release machine runs.
function packageLinux() {
  const bundleDir = resolve(SRC_TAURI, "target", "linux-release", "bundle");
  console.log("\nPackaging Linux artifacts...\n");

  const found = [
    [findForVersion(resolve(bundleDir, "deb"), /\.deb$/), ASSET_NAMES.linuxDeb],
    [findForVersion(resolve(bundleDir, "rpm"), /\.rpm$/), ASSET_NAMES.linuxRpm],
    [findForVersion(resolve(bundleDir, "appimage"), /\.AppImage$/), ASSET_NAMES.linuxAppImage],
  ].filter(([source]) => source);

  for (const [source, name] of found) copyArtifact(source, name);
  if (found.length === 0) console.log("  No Linux build artifacts found. Run 'npm run release:linux' first.");
  return found.length;
}

function packageWindows() {
  const bundleDir = resolve(SRC_TAURI, "target", "release", "bundle");
  console.log("\nPackaging Windows artifacts...\n");

  const found = [
    [findForVersion(resolve(bundleDir, "msi"), /\.msi$/), ASSET_NAMES.windowsMsi],
    [findForVersion(resolve(bundleDir, "nsis"), /\.exe$/), ASSET_NAMES.windowsSetup],
  ].filter(([source]) => source);

  for (const [source, name] of found) copyArtifact(source, name);
  if (found.length === 0) console.log("  No Windows build artifacts found. Run 'npm run tauri:build' first.");
  return found.length;
}

function main() {
  assertPlainVersion();
  if (process.platform === "darwin") {
    console.error("Rusty G6 does not ship a macOS build.");
    process.exit(1);
  }

  // Start from an empty folder so its whole contents can be uploaded as-is.
  rmSync(OUT_DIR, { recursive: true, force: true });
  mkdirSync(OUT_DIR, { recursive: true });

  console.log(`Rusty G6 v${VERSION} release packaging`);
  console.log(`Output: ${OUT_DIR}`);

  const total = process.platform === "win32" ? packageWindows() : packageLinux();

  console.log(`\nDone. ${total} artifact(s) packaged in ${OUT_DIR}`);
  if (total === 0) process.exit(1);
}

main();
