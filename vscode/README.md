# Vitest Linter — VS Code Extension

[![Install from Marketplace](https://img.shields.io/badge/VS%20Code-Install-blue)](https://marketplace.visualstudio.com/items?itemName=jonathangadeaharder.vitest-linter)

Zero-config test-smell diagnostics for Vitest projects, directly in VS Code.

## Features

- Activates automatically when editing test files (`.test.ts`, `.spec.ts`, `.test.js`, etc.)
- Runs [vitest-linter](https://github.com/Jonathangadeaharder/vitest-linter) on save (or on type) and shows diagnostics in the Problems panel
- Supports 65+ lint rules across flakiness, maintenance, structure, dependencies, validation, and Playwright categories
- Configurable rule severity overrides, include/exclude globs, and run trigger

## Prerequisites

The `vitest-linter` binary must be available on your `PATH`, or you must set `vitest-linter.executablePath` in settings.

Install the binary:

```bash
cargo install vitest-linter
```

Or download a prebuilt binary from the [GitHub Releases](https://github.com/Jonathangadeaharder/vitest-linter/releases) page.

## Installation

### From the VS Code Marketplace

1. Open the Extensions view (`Ctrl+Shift+X` / `Cmd+Shift+X`)
2. Search for **Vitest Linter**
3. Click **Install**

### From VSIX

```bash
code --install-extension vitest-linter-0.1.0.vsix
```

## Configuration

Open `settings.json` (`Ctrl+,` → "Open Settings (JSON)"):

```jsonc
{
  // Enable/disable the extension (default: true)
  "vitest-linter.enable": true,

  // Path to the vitest-linter binary (default: "vitest-linter")
  "vitest-linter.executablePath": "vitest-linter",

  // When to run: "onSave" or "onType" (default: "onSave")
  "vitest-linter.run": "onSave",

  // Glob patterns to include (empty = all test files)
  "vitest-linter.include": [],

  // Glob patterns to exclude
  "vitest-linter.exclude": ["**/node_modules/**"],

  // Override severity per rule: "off" | "info" | "warning" | "error"
  "vitest-linter.severityOverrides": {
    "VITEST-FLK-001": "off",
    "VITEST-MNT-003": "info"
  }
}
```

## Commands

| Command | Description |
|---------|-------------|
| `Vitest Linter: Lint Workspace` | Lint all test files in the workspace and show results |

## How It Works

The extension spawns `vitest-linter --format json <file>` and parses the JSON output into VS Code diagnostics. No LSP server required — just the CLI binary.

## License

MIT
