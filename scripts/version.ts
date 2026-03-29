import { join } from "jsr:@std/path";

const colors = {
  reset: "\x1b[0m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  red: "\x1b[31m",
};

export async function syncVersion() {
  const rootDir = Deno.cwd();
  
  try {
    // 1. Read version from deno.json (source of truth)
    const denoJsonPath = join(rootDir, "deno.json");
    const denoJsonContent = await Deno.readTextFile(denoJsonPath);
    const denoJson = JSON.parse(denoJsonContent);
    const version = denoJson.version;
    
    if (!version) {
      throw new Error("No version found in deno.json");
    }
    
    console.log(`${colors.cyan}ℹ️ Source of truth version: ${version}${colors.reset}`);
    
    // 2. Update tauri.conf.json
    const tauriConfPath = join(rootDir, "src-tauri", "tauri.conf.json");
    try {
      const tauriConfContent = await Deno.readTextFile(tauriConfPath);
      const tauriConf = JSON.parse(tauriConfContent);
      
      if (tauriConf.version !== version) {
        console.log(`${colors.yellow}🔄 Updating tauri.conf.json from ${tauriConf.version} to ${version}${colors.reset}`);
        tauriConf.version = version;
        await Deno.writeTextFile(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");
        console.log(`${colors.green}✅ tauri.conf.json updated${colors.reset}`);
      } else {
        console.log(`${colors.green}✅ tauri.conf.json is up to date${colors.reset}`);
      }
    } catch (e) {
      console.log(`${colors.red}❌ Failed to update tauri.conf.json: ${e}${colors.reset}`);
    }
    
    // 3. Update Cargo.toml
    const cargoTomlPath = join(rootDir, "src-tauri", "Cargo.toml");
    try {
      const cargoTomlContent = await Deno.readTextFile(cargoTomlPath);
      
      // Regex to find the version field in the [package] section
      const versionRegex = /^version\s*=\s*"([^"]+)"/m;
      const match = cargoTomlContent.match(versionRegex);
      
      if (match && match[1] !== version) {
        console.log(`${colors.yellow}🔄 Updating Cargo.toml from ${match[1]} to ${version}${colors.reset}`);
        const newCargoTomlContent = cargoTomlContent.replace(versionRegex, `version = "${version}"`);
        await Deno.writeTextFile(cargoTomlPath, newCargoTomlContent);
        console.log(`${colors.green}✅ Cargo.toml updated${colors.reset}`);
      } else if (match) {
        console.log(`${colors.green}✅ Cargo.toml is up to date${colors.reset}`);
      } else {
        console.log(`${colors.yellow}⚠️ Could not find version field in Cargo.toml${colors.reset}`);
      }
    } catch (e) {
      console.log(`${colors.red}❌ Failed to update Cargo.toml: ${e}${colors.reset}`);
    }
    
  } catch (error) {
    console.error(`${colors.red}❌ Failed to sync versions: ${error}${colors.reset}`);
  }
}

if (import.meta.main) {
  await syncVersion();
}
