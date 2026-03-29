import { exists } from "jsr:@std/fs";

const colors = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  red: "\x1b[31m",
};

interface Dependency {
  name: string;
  cmd?: string;
  apt?: string | string[];
  pacman?: string | string[];
  pkgConfig?: string;
  install?: {
    apt: string;
    pacman: string;
    windows?: string;
  };
  desc: string;
  defaults?: string[];
}

const REGISTRY: Record<string, Dependency[]> = {
  linux: [
    {
      name: "cmake",
      cmd: "cmake",
      apt: "cmake",
      pacman: "cmake",
      install: {
        apt: "sudo apt install cmake",
        pacman: "sudo pacman -S cmake",
      },
      desc: "CMake build system",
    },
    {
      name: "pkg-config",
      cmd: "pkg-config",
      apt: "pkg-config",
      pacman: "pkgconf",
      install: {
        apt: "sudo apt install pkg-config",
        pacman: "sudo pacman -S pkgconf",
      },
      desc: "Package configuration tool",
    },
    {
      name: "build-essential",
      apt: "build-essential",
      pacman: "base-devel",
      cmd: "g++",
      install: {
        apt: "sudo apt install build-essential",
        pacman: "sudo pacman -S base-devel",
      },
      desc: "Build tools (gcc, g++, make, etc.)",
    },
    {
      name: "libusb",
      apt: "libusb-1.0-0-dev",
      pacman: "libusb",
      pkgConfig: "libusb-1.0",
      install: {
        apt: "sudo apt install libusb-1.0-0-dev",
        pacman: "sudo pacman -S libusb",
      },
      desc: "libusb development headers (Required for rusb)",
    },
    {
      name: "libudev",
      apt: "libudev-dev",
      pacman: "systemd-libs",
      pkgConfig: "libudev",
      install: {
        apt: "sudo apt install libudev-dev",
        pacman: "sudo pacman -S systemd-libs",
      },
      desc: "libudev development headers (Required for hidapi)",
    },
    {
      name: "libwebkit2gtk-4.1",
      apt: "libwebkit2gtk-4.1-dev",
      pacman: "webkit2gtk-4.1",
      pkgConfig: "webkit2gtk-4.1",
      install: {
        apt: "sudo apt install libwebkit2gtk-4.1-dev",
        pacman: "sudo pacman -S webkit2gtk-4.1",
      },
      desc: "WebKitGTK development headers (Required for Tauri)",
    },
    {
      name: "libgtk-3",
      apt: "libgtk-3-dev",
      pacman: "gtk3",
      pkgConfig: "gtk+-3.0",
      install: {
        apt: "sudo apt install libgtk-3-dev",
        pacman: "sudo pacman -S gtk3",
      },
      desc: "GTK3 development headers (Required for Tauri)",
    },
    {
      name: "rust",
      cmd: "cargo",
      desc: "Rust toolchain (cargo, rustc)",
      install: {
        apt: "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
        pacman: "sudo pacman -S rustup && rustup default stable",
        windows: "https://rustup.rs/",
      },
    }
  ],
  windows: [
    {
      name: "rust",
      cmd: "cargo",
      desc: "Rust toolchain (cargo, rustc)",
      install: {
        apt: "https://rustup.rs/",
        pacman: "https://rustup.rs/",
        windows: "winget install -e --id Rustlang.Rustup",
      },
    }
  ]
};

async function isCommandInPath(cmd: string): Promise<boolean> {
  try {
    const process = new Deno.Command(cmd, {
      args: ["--version"],
      stdout: "null",
      stderr: "null",
    });
    const { success } = await process.output();
    return success;
  } catch {
    return false;
  }
}

async function checkLinuxDeps(isDev: boolean) {
  console.log(`\n${colors.bright}[0]${colors.reset} ${colors.cyan}Checking Linux dependencies...${colors.reset}`);

  const hasApt = await isCommandInPath("apt-get");
  const hasPacman = await isCommandInPath("pacman");
  const hasPkgConfig = await isCommandInPath("pkg-config");

  const missingPackages: string[] = [];
  const missingDescriptions: string[] = [];

  for (const dep of REGISTRY.linux) {
    let found = false;

    // 1. Check if command is in PATH
    if (dep.cmd && await isCommandInPath(dep.cmd)) {
      found = true;
    }

    // 2. Check via pkg-config (best for libraries/headers)
    if (!found && hasPkgConfig && dep.pkgConfig) {
      try {
        const process = new Deno.Command("pkg-config", {
          args: ["--exists", dep.pkgConfig],
        });
        const { success } = await process.output();
        if (success) found = true;
      } catch { /* ignore */ }
    }

    // 3. Check via package manager
    if (!found) {
      if (hasApt && dep.apt) {
        const aptPackages = Array.isArray(dep.apt) ? dep.apt : [dep.apt];
        for (const pkg of aptPackages) {
          try {
            const process = new Deno.Command("dpkg", {
              args: ["-s", pkg],
            });
            const { success } = await process.output();
            if (success) {
              found = true;
              break;
            }
          } catch { /* ignore */ }
        }
      } else if (hasPacman && dep.pacman) {
        const pacmanPackages = Array.isArray(dep.pacman) ? dep.pacman : [dep.pacman];
        for (const pkg of pacmanPackages) {
          try {
            const process = new Deno.Command("pacman", {
              args: ["-Qq", pkg],
            });
            const { success } = await process.output();
            if (success) {
              found = true;
              break;
            }
          } catch { /* ignore */ }
        }
      }
    }

    if (!found) {
      console.error(`${colors.red}❌ Missing: ${dep.name}${colors.reset} (${dep.desc})`);
      missingDescriptions.push(`${dep.name} (${dep.desc})`);
      
      if (hasApt && dep.apt) {
        const pkg = Array.isArray(dep.apt) ? dep.apt[0] : dep.apt;
        missingPackages.push(pkg);
      } else if (hasPacman && dep.pacman) {
        const pkg = Array.isArray(dep.pacman) ? dep.pacman[0] : dep.pacman;
        missingPackages.push(pkg);
      } else if (dep.install) {
        // Handle tools with custom install commands (like Rust)
        const customCmd = hasApt ? dep.install.apt : (hasPacman ? dep.install.pacman : "");
        if (customCmd) {
          missingDescriptions.push(`${colors.yellow}👉 To install ${dep.name}: ${colors.bright}${customCmd}${colors.reset}`);
        }
      }
    } else {
      console.log(`${colors.green}✅ ${dep.name} is installed${colors.reset}`);
    }
  }

  if (missingPackages.length > 0 || missingDescriptions.some(d => d.includes("👉"))) {
    if (missingPackages.length > 0) {
      console.log(`\n${colors.bright}${colors.red}Found ${missingPackages.length} missing system packages!${colors.reset}`);
      
      let installCmd = "";
      if (hasApt) {
        installCmd = `sudo apt install ${missingPackages.join(" ")}`;
      } else if (hasPacman) {
        installCmd = `sudo pacman -S ${missingPackages.join(" ")}`;
      }

      if (installCmd) {
        console.log(`${colors.yellow}Please install them with:${colors.reset}`);
        console.log(`${colors.bright}${colors.cyan}${installCmd}${colors.reset}`);
      }
    }

    const specialInstalls = missingDescriptions.filter(d => d.includes("👉"));
    if (specialInstalls.length > 0) {
      console.log(`\n${colors.bright}${colors.yellow}Additional setup required:${colors.reset}`);
      for (const special of specialInstalls) {
        console.log(special);
      }
    }
    
    Deno.exit(1);
  }
}

async function checkWindowsDeps() {
  console.log(`\n${colors.bright}[0]${colors.reset} ${colors.cyan}Checking Windows build dependencies...${colors.reset}`);

  const missingTools: string[] = [];

  for (const dep of REGISTRY.windows) {
    if (await isCommandInPath(dep.cmd!)) {
      console.log(`${colors.green}✅ ${dep.name} is available in PATH${colors.reset}`);
      continue;
    } else {
      missingTools.push(dep.name);
      console.error(`${colors.red}❌ Missing tool: ${dep.name} (${dep.desc})${colors.reset}`);
    }
  }

  if (missingTools.length > 0) {
    const installIds = missingTools.map(name => {
      const dep = REGISTRY.windows.find(d => d.name === name);
      return dep?.install?.windows?.split(" ").pop() || name;
    });

    console.log(`\n${colors.yellow}Please install the missing tools with:${colors.reset}`);
    console.log(`${colors.bright}${colors.cyan}winget install ${installIds.join(" ")}${colors.reset}`);
    console.log(`${colors.yellow}Note: You MUST restart your terminal after installation.${colors.reset}`);
    Deno.exit(1);
  }
}

export async function verifyDependencies(isDev: boolean = false) {
  if (Deno.build.os === "linux") {
    await checkLinuxDeps(isDev);
  } else if (Deno.build.os === "windows") {
    await checkWindowsDeps();
  } else {
    console.log(`${colors.yellow}⚠️ Unsupported OS for dependency checking: ${Deno.build.os}${colors.reset}`);
  }
}
