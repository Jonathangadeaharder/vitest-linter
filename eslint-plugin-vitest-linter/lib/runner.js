const { execFileSync } = require("child_process");
const path = require("path");

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
  if (cache.has(normalized)) {
    return cache.get(normalized);
  }

  let violations = [];
  try {
    const bin = getBinary();
    const result = execFileSync(bin, ["--format", "json", normalized], {
      encoding: "utf8",
      maxBuffer: 10 * 1024 * 1024,
      timeout: 30000,
    });
    violations = JSON.parse(result);
  } catch {
    violations = [];
  }

  cache.set(normalized, violations);
  return violations;
}

function clearCache() {
  cache.clear();
  binaryPath = null;
}

module.exports = { getViolations, clearCache };
