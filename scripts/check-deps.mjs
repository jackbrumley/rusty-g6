#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";

const colors = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  green: "\x1b[32m",
  cyan: "\x1b[36m",
  red: "\x1b[31m",
  yellow: "\x1b[33m",
};

function runOk(command, args) {
  try {
    const result = spawnSync(command, args, { stdio: "ignore" });
    return result.status === 0;
  } catch {
    return false;
  }
}

function commandExists(command) {
  const locator = process.platform === "win32" ? "where.exe" : "which";
  return runOk(locator, [command]);
}

function checkPkgConfig(name) {
  return commandExists("pkg-config") && runOk("pkg-config", ["--exists", name]);
}

function checkDeb(pkg) {
  return runOk("dpkg", ["-s", pkg]);
}

function checkRpm(pkg) {
  return runOk("rpm", ["-q", pkg]);
}

function checkPacman(pkg) {
  return runOk("pacman", ["-Qq", pkg]);
}

function linuxDepsDebian() {
  return [
    ["cmake", () => commandExists("cmake"), "sudo apt install -y cmake"],
    ["pkg-config", () => commandExists("pkg-config"), "sudo apt install -y pkg-config"],
    ["build-essential", () => checkDeb("build-essential"), "sudo apt install -y build-essential"],
    ["libusb", () => checkDeb("libusb-1.0-0-dev"), "sudo apt install -y libusb-1.0-0-dev"],
    ["libudev", () => checkDeb("libudev-dev"), "sudo apt install -y libudev-dev"],
    ["webkit2gtk", () => checkDeb("libwebkit2gtk-4.1-dev"), "sudo apt install -y libwebkit2gtk-4.1-dev"],
    ["gtk3", () => checkDeb("libgtk-3-dev"), "sudo apt install -y libgtk-3-dev"],
    [
      "ayatana appindicator",
      () => checkDeb("libayatana-appindicator3-dev"),
      "sudo apt install -y libayatana-appindicator3-dev",
    ],
    ["rust", () => commandExists("cargo"), "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"],
    ["nodejs", () => commandExists("node"), "sudo apt install -y nodejs npm"],
  ];
}

function linuxDepsFedora() {
  return [
    ["cmake", () => commandExists("cmake"), "sudo dnf install -y cmake"],
    ["pkg-config", () => commandExists("pkg-config"), "sudo dnf install -y pkgconf-pkg-config"],
    ["gcc-c++", () => checkRpm("gcc-c++"), "sudo dnf install -y gcc-c++ make"],
    ["libusb", () => checkPkgConfig("libusb-1.0"), "sudo dnf install -y libusb1-devel"],
    ["libudev", () => checkPkgConfig("libudev"), "sudo dnf install -y systemd-devel"],
    ["webkit2gtk", () => checkPkgConfig("webkit2gtk-4.1"), "sudo dnf install -y webkit2gtk4.1-devel"],
    ["gtk3", () => checkPkgConfig("gtk+-3.0"), "sudo dnf install -y gtk3-devel"],
    [
      "ayatana appindicator",
      () => checkPkgConfig("ayatana-appindicator3-0.1"),
      "sudo dnf install -y libayatana-appindicator-gtk3-devel",
    ],
    ["rust", () => commandExists("cargo"), "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"],
    ["nodejs", () => commandExists("node"), "sudo dnf install -y nodejs npm"],
  ];
}

function linuxDepsArch() {
  return [
    ["cmake", () => commandExists("cmake"), "sudo pacman -S --noconfirm cmake"],
    ["pkg-config", () => commandExists("pkg-config"), "sudo pacman -S --noconfirm pkgconf"],
    ["base-devel", () => checkPacman("base-devel"), "sudo pacman -S --noconfirm base-devel"],
    ["libusb", () => checkPacman("libusb"), "sudo pacman -S --noconfirm libusb"],
    ["libudev", () => checkPacman("systemd-libs"), "sudo pacman -S --noconfirm systemd-libs"],
    ["webkit2gtk", () => checkPacman("webkit2gtk-4.1"), "sudo pacman -S --noconfirm webkit2gtk-4.1"],
    ["gtk3", () => checkPacman("gtk3"), "sudo pacman -S --noconfirm gtk3"],
    [
      "ayatana appindicator",
      () => checkPacman("libayatana-appindicator"),
      "sudo pacman -S --noconfirm libayatana-appindicator",
    ],
    ["rust", () => commandExists("cargo"), "sudo pacman -S --noconfirm rustup && rustup default stable"],
    ["nodejs", () => commandExists("node"), "sudo pacman -S --noconfirm nodejs npm"],
  ];
}

function detectLinuxFamily() {
  const file = "/etc/os-release";
  if (!fs.existsSync(file)) return "debian";
  const content = fs.readFileSync(file, "utf8").toLowerCase();
  if (content.includes("fedora") || content.includes("rhel")) return "fedora";
  if (content.includes("arch")) return "arch";
  return "debian";
}

function getDeps() {
  if (process.platform === "win32") {
    return [
      ["rust", () => commandExists("cargo"), "winget install -e --id Rustlang.Rustup"],
      ["nodejs", () => commandExists("node"), "winget install -e --id OpenJS.NodeJS"],
    ];
  }

  if (process.platform !== "linux") {
    return [];
  }

  const family = detectLinuxFamily();
  if (family === "fedora") return linuxDepsFedora();
  if (family === "arch") return linuxDepsArch();
  return linuxDepsDebian();
}

function main() {
  console.log(`\n${colors.bright}[0]${colors.reset} ${colors.cyan}Checking system dependencies...${colors.reset}`);
  const deps = getDeps();
  if (deps.length === 0) {
    console.log(`${colors.yellow}No dependency checks for platform: ${process.platform}${colors.reset}`);
    return;
  }

  const missing = [];
  for (const [name, check, install] of deps) {
    if (check()) {
      console.log(`${colors.green}✅ ${name} is installed${colors.reset}`);
    } else {
      console.error(`${colors.red}❌ Missing: ${name}${colors.reset}`);
      missing.push(install);
    }
  }

  if (missing.length > 0) {
    console.log(`\n${colors.bright}${colors.red}Missing dependencies found.${colors.reset}`);
    console.log(`${colors.yellow}Install with:${colors.reset}`);
    for (const cmd of [...new Set(missing)]) {
      console.log(`${colors.cyan}${cmd}${colors.reset}`);
    }
    process.exit(1);
  }

  console.log(`\n${colors.green}✅ All system dependencies are installed!${colors.reset}`);
}

main();
