const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const cache = new Map();

function findBinary() {
  const binName = process.platform === "win32" ? "vitest-linter.exe" : "vitest-linter";

  const local = path.resolve(__dirname, "..", "node_modules", ".bin", binName);
  try {
    execFileSync(local, ["--help"], { timeout: 5000, stdio: "pipe" });
    return local;
  } catch {
    // fallthrough
  }

  return binName;
}

let binaryPath = null;

function getBinary() {
  if (!binaryPath) {
    binaryPath = findBinary();
  }
  return binaryPath;
}

function getViolations(filePath) {
  const normalized = path.resolve(filePath);
  let mtime = 0;
  try {
    mtime = fs.statSync(normalized).mtimeMs;
  } catch {
    // File might not be accessible
  }
  const cacheKey = `${normalized}:${mtime}`;

  if (cache.has(cacheKey)) {
    return cache.get(cacheKey);
  }

  const violations = {};
  try {
    const bin = getBinary();
    const result = execFileSync(bin, ["--format", "json", normalized], {
      encoding: "utf8",
      maxBuffer: 10 * 1024 * 1024,
      timeout: 30000,
    });
    const raw = JSON.parse(result);
    for (const v of raw) {
      if (!violations[v.rule_id]) violations[v.rule_id] = [];
      violations[v.rule_id].push(v);
    }
  } catch {
    // Return empty map on error
  }

  cache.set(cacheKey, violations);
  return violations;
}

function clearCache() {
  cache.clear();
  binaryPath = null;
}

module.exports = { getViolations, clearCache };
