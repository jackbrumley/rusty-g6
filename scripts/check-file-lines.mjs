import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

// File size budget (App Standard section 8): 600 lines soft, 1000 hard.
const ROOT = process.cwd();
const MAX_LINES = 1000;
const SOFT_MAX_LINES = 600;

// Files allowed over the hard cap while they are being split. Each entry
// needs a comment saying why and where the split is tracked (docs/TODO.md).
const ALLOWLIST = new Set([]);
const TARGET_DIRS = ["src", path.join("src-tauri", "src")];
const SOURCE_EXTENSIONS = new Set([".rs", ".ts", ".tsx", ".css"]);

const gatherFiles = async (directoryPath) => {
  const entries = await readdir(directoryPath, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(directoryPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await gatherFiles(fullPath)));
      continue;
    }
    if (entry.isFile() && SOURCE_EXTENSIONS.has(path.extname(entry.name))) {
      files.push(fullPath);
    }
  }
  return files;
};

const lineCount = async (filePath) => {
  const content = await readFile(filePath, "utf8");
  if (content.length === 0) {
    return 0;
  }
  return content.split(/\r?\n/).length;
};

const run = async () => {
  const filePaths = [];
  for (const relativeDirectory of TARGET_DIRS) {
    const fullDirectory = path.join(ROOT, relativeDirectory);
    filePaths.push(...(await gatherFiles(fullDirectory)));
  }

  const oversized = [];
  const overSoftBudget = [];
  for (const filePath of filePaths) {
    const count = await lineCount(filePath);
    const relativePath = path.relative(ROOT, filePath);
    if (count > SOFT_MAX_LINES && count <= MAX_LINES) {
      overSoftBudget.push(relativePath);
    }
    if (count > MAX_LINES && !ALLOWLIST.has(relativePath)) {
      oversized.push({
        relativePath: path.relative(ROOT, filePath),
        count,
      });
    }
  }

  if (oversized.length > 0) {
    oversized.sort((a, b) => b.count - a.count);
    console.error(`Line cap violation: files must be <= ${MAX_LINES} lines.`);
    for (const violation of oversized) {
      console.error(`- ${violation.relativePath}: ${violation.count}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log(`Line cap check passed (${MAX_LINES} max lines).`);
  if (overSoftBudget.length > 0) {
    console.log(`${overSoftBudget.length} file(s) are over the ${SOFT_MAX_LINES}-line soft budget; split them when you next change them.`);
  }
};

await run();
