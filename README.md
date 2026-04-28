# vitest-linter

A fast, zero-config test-smell linter for TypeScript/JavaScript Vitest test suites, written in Rust.

## Features

- **14 rules** across 4 categories: Flakiness, Maintenance, Structure, Dependencies
- Optional `.vitest-linter.toml` for project-specific banlists (DI rules)
- Recursive file discovery via [walkdir](https://docs.rs/walkdir)
- Tree-sitter-powered AST analysis (TypeScript **and** TSX/JSX)
- JSON and terminal output formats
- Exit code 1 on `Error`-severity violations (CI-friendly)

## Installation

```bash
cargo install vitest-linter
```

Or download a prebuilt binary from the [GitHub Releases](https://github.com/Jonathangadeaharder/vitest-linter/releases) page.

## Usage

```bash
# Lint current directory (recursively)
vitest-linter

# Lint specific paths
vitest-linter src/tests/ lib/

# JSON output
vitest-linter --format json

# Write output to file
vitest-linter --format json --output report.json

# Disable terminal colours
vitest-linter --no-color
```

## Rules

> **14 rules** implemented across 4 categories.  Numeric suffixes are kept in
> parity with pytest-linter where a 1:1 semantic mapping exists.

### Flakiness (VITEST-FLK-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-FLK-001 | TimeoutRule | Warning | `setTimeout`/`setInterval` used inside a test without fake timers |
| VITEST-FLK-002 | DateMockRule | Warning | `Date` / `Date.now()` used without `vi.useFakeTimers()` |
| VITEST-FLK-003 | NetworkImportRule | Warning | Test file imports a network library (axios, node-fetch, got, …) without mocking |

### Maintenance (VITEST-MNT-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-MNT-001 | NoAssertionRule | Error | `it`/`test` body contains no `expect()` calls |
| VITEST-MNT-002 | MultipleExpectRule | Warning | More than 5 `expect()` calls in a single test |
| VITEST-MNT-003 | ConditionalLogicRule | Warning | `if` or `switch` statement inside a test body |
| VITEST-MNT-004 | TryCatchRule | Warning | `try/catch` inside a test — prefer `expect().toThrow()` |
| VITEST-MNT-005 | EmptyTestRule | Info | `it.skip` / `test.todo` left in source |
| VITEST-MNT-006 | MissingAwaitAssertionRule | Error | `.resolves` or `.rejects` assertion not preceded by `await` |

### Structure (VITEST-STR-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-STR-001 | NestedDescribeRule | Warning | Test lives inside more than one level of `describe` nesting |
| VITEST-STR-002 | ReturnInTestRule | Warning | `return` statement found inside a test body |

### Dependencies (VITEST-DEP-*)

Catch test-isolation bugs that arise from module-level mocking of singleton
infrastructure. Active only when `.vitest-linter.toml` configures a banlist.

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-DEP-001 | BannedModuleMockRule | Error | `vi.mock(<path>)` at module scope where path matches a configured banlist (e.g. shared `db`/`eventBus`/`container`). Such mocks leak across test files via the module cache and silently corrupt downstream tests. Refactor the target service to accept dependencies via constructor (DI). |
| VITEST-DEP-002 | ProductionSingletonImportRule | Error | Unit test imports a configured production singleton. Importing the singleton triggers its constructor side effects (event-handler registration, DB connections) on the production wiring. Construct a fresh instance with fakes; production singletons belong in `*.integration.test.ts` only. |
| VITEST-DEP-003 | ResetEscapeHatchRule | Warning | `vi.resetModules()` / `vi.restoreAllMocks()` / `vi.unmock()` inside `beforeEach`/`beforeAll`/`afterEach`/`afterAll`. These mask underlying coupling between test files instead of fixing it. |

## Configuration (`.vitest-linter.toml`)

Place a `.vitest-linter.toml` next to your `package.json`. The linter walks up
from the input path to find it.

```toml
[deps]
# Paths whose module-level vi.mock(...) is forbidden. Globbed against the
# string passed to vi.mock(...). Leading "./" and "../" are stripped before
# matching, so "**/infrastructure/database" matches "../infrastructure/database".
banned_mock_paths = [
  "**/infrastructure/database",
  "**/infrastructure/event-bus",
  "**/infrastructure/container",
]

# Override default integration-test glob (used by DEP-002 to skip files where
# importing real singletons is the contract under test).
integration_test_glob = "**/*.integration.test.{ts,tsx,js,jsx}"

# Production singletons that must not be imported in unit tests.
[[deps.banned_singletons]]
from  = "**/services/pipeline-orchestrator"
names = ["orchestrator"]

[[deps.banned_singletons]]
from  = "**/services/progress-persistence"
names = ["progressPersistence"]
```

If no config file exists, DEP-001 and DEP-002 are inactive (no banlist → no
violations); DEP-003 still runs with its built-in defaults.

## Supported File Extensions

`.test.ts`, `.spec.ts`, `.test.tsx`, `.spec.tsx`, `.test.js`, `.spec.js`, `.test.jsx`, `.spec.jsx`

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | No `Error`-severity violations found |
| `1` | At least one `Error`-severity violation found |

## Development

```bash
# Run all tests
cargo test

# Run benchmarks
cargo bench

# Lint + format
cargo clippy --all-targets
cargo fmt
```

## License

MIT
