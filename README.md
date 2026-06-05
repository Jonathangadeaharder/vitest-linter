# vitest-linter

A fast, zero-config test-smell linter for TypeScript/JavaScript Vitest test suites, written in Rust.

## Features

- **8 must-have rules** active by default for v1.0 (66 total behind `--unstable-rules`)
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

# SARIF output (for GitHub Code Scanning)
vitest-linter --format sarif

# Write output to file
vitest-linter --format json --output report.json

# Disable terminal colours
vitest-linter --no-color

# Incremental mode (only lint files changed since base ref)
vitest-linter --incremental
vitest-linter --incremental --base origin/main

# Enable all rules (including unstable)
vitest-linter --unstable-rules
```

## Rules

> **8 must-have rules** active by default for v1.0. These catch the most
> real-world pain (high signal, low false-positive). Pass `--unstable-rules`
> to access the full set of 66 rules across 6 categories (organized into 10
> rule groups by ID prefix).

### v1.0 Rules (active by default)

These 8 rules are the community-hit set — the ones that catch real bugs in
real codebases with minimal noise.

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-MNT-007 | FocusedTestRule | Error | `it.only` / `test.only` / `describe.only` left in source |
| VITEST-MNT-005 | EmptyTestRule | Info | `it.skip` / `test.todo` left in source |
| VITEST-NO-003 | NoCommentedOutTestsRule | Maintenance | Commented-out `it(` / `test(` / `describe(` lines |
| VITEST-MNT-006 | MissingAwaitAssertionRule | Warning | `.resolves` or `.rejects` assertion not preceded by `await` |
| VITEST-FLK-001 | TimeoutRule | Warning | `setTimeout`/`setInterval` used inside a test without fake timers |
| VITEST-MNT-004 | TryCatchRule | Warning | `try/catch` inside a test — prefer `expect().toThrow()` |
| VITEST-MNT-003 | ConditionalLogicRule | Warning | `if` or `switch` statement inside a test body |
| VITEST-PREF-009 | PreferHooksOnTopRule | Structure | Hooks should be placed above all test cases in a `describe` block |

### Unstable Rules (`--unstable-rules`)

The full rule set is available behind the `--unstable-rules` flag. These rules
are still being validated against real-world repos and may produce false positives.

### Flakiness (VITEST-FLK-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-FLK-001 | TimeoutRule | Warning | `setTimeout`/`setInterval` used inside a test without fake timers |
| VITEST-FLK-002 | DateMockRule | Warning | `Date` / `Date.now()` used without `vi.useFakeTimers()` |
| VITEST-FLK-003 | NetworkImportRule | Warning | Test file imports a network library (axios, node-fetch, got, …) without mocking |
| VITEST-FLK-004 | FakeTimersCleanupRule | Warning | `vi.useFakeTimers()` without `afterEach` cleanup |
| VITEST-FLK-005 | NonDeterministicRule | Warning | `Math.random()` / `crypto.randomUUID()` used without seeding |

### Maintenance (VITEST-MNT-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-MNT-001 | NoAssertionRule | Error | `it`/`test` body contains no `expect()` calls |
| VITEST-MNT-002 | MultipleExpectRule | Warning | More than 5 `expect()` calls in a single test |
| VITEST-MNT-003 | ConditionalLogicRule | Warning | `if` or `switch` statement inside a test body |
| VITEST-MNT-004 | TryCatchRule | Warning | `try/catch` inside a test — prefer `expect().toThrow()` |
| VITEST-MNT-005 | EmptyTestRule | Info | `it.skip` / `test.todo` left in source |
| VITEST-MNT-006 | MissingAwaitAssertionRule | Warning | `.resolves` or `.rejects` assertion not preceded by `await` |
| VITEST-MNT-007 | FocusedTestRule | Error | `it.only` / `test.only` / `describe.only` left in source |
| VITEST-MNT-008 | MissingMockCleanupRule | Warning | `vi.mock()` without `afterEach` cleanup (`vi.restoreAllMocks()` / `vi.clearAllMocks()`) |
| VITEST-MNT-009 | WeakAssertionRule | Warning | All assertions in a test are weak (`toBeDefined()`, `toBeTruthy()`, `not.toThrow()`) — verify actual behavior |
| VITEST-MNT-010 | ImplementationCoupledRule | Warning | Test file is tightly coupled to a single module's implementation — test count ≈ export count with >80% name match |
| VITEST-MNT-011 | TestIdNegativePresenceRule | Warning | Uses `getByTestId` but has no negative-presence assertion — missing coverage for element absence |

### Structure (VITEST-STR-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-STR-001 | NestedDescribeRule | Warning | Test lives inside more than 3 levels of `describe` nesting |
| VITEST-STR-002 | ReturnInTestRule | Warning | `return` statement found inside a test body |

### Dependencies (VITEST-DEP-*)

Catch test-isolation bugs that arise from module-level mocking of singleton
infrastructure. DEP-001 also fires on stable dependencies (pure functions,
data models) detected via heuristics. DEP-002 requires a configured banlist.

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-DEP-001 | BannedModuleMockRule | Error | `vi.mock(<path>)` at module scope where path matches a configured banlist (e.g. shared `db`/`eventBus`/`container`) or is detected as a stable dependency (pure functions, data models, utils). Mocking stable deps indicates a DI problem. Refactor the target service to accept dependencies via constructor (DI). |
| VITEST-DEP-002 | ProductionSingletonImportRule | Error | Unit test imports a configured production singleton. Importing the singleton triggers its constructor side effects (event-handler registration, DB connections) on the production wiring. Construct a fresh instance with fakes; production singletons belong in `*.integration.test.ts` only. |
| VITEST-DEP-003 | ResetEscapeHatchRule | Warning | `vi.resetModules()` / `vi.restoreAllMocks()` / `vi.unmock()` / `vi.doUnmock()` inside `beforeEach`/`beforeAll`/`afterEach`/`afterAll`. These mask underlying coupling between test files instead of fixing it. |
| VITEST-DEP-004 | MockExportValidationRule | Warning | `vi.mock()` factory returns a key that is not exported by the source module |

### Validation (VITEST-VAL-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-VAL-001 | ValidExpectRule | Error | `expect()` call missing `.toBe()` or similar assertion — promise silently swallowed |
| VITEST-VAL-002 | ValidExpectInPromiseRule | Error | `return expect(...)` instead of `await expect(...)` — assertion may fail silently |
| VITEST-VAL-003 | ValidDescribeCallbackRule | Error | `describe()` with an `async` callback — must be synchronous |
| VITEST-VAL-004 | ValidTitleRule | Warning | Test/describe title is empty or a template literal — prefer static string titles |
| VITEST-VAL-005 | NoUnneededAsyncExpectFunctionRule | Warning | `expect(async () => {...})` wrapping async function unnecessarily — use `.resolves` / `.rejects` instead |

### No-* (VITEST-NO-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-NO-001 | NoStandaloneExpectRule | Error | `expect()` call outside any `it`/`test` body |
| VITEST-NO-002 | NoIdenticalTitleRule | Error | Two `it`/`describe` blocks in the same scope share the same title |
| VITEST-NO-003 | NoCommentedOutTestsRule | Maintenance | Commented-out `it(` / `test(` / `describe(` lines |
| VITEST-NO-005 | NoTestPrefixesRule | Warning | `fit()` / `xit()` used — use `test.skip()` / `test.only()` instead |
| VITEST-NO-006 | NoDuplicateHooksRule | Warning | Multiple `beforeEach` / `afterEach` / etc. in the same `describe` |
| VITEST-NO-007 | NoImportNodeTestRule | Error | `import ... from 'node:test'` in a Vitest project |
| VITEST-NO-008 | NoInterpolationInSnapshotsRule | Warning | Template literal with `${}` inside `toMatchSnapshot()` |
| VITEST-NO-009 | NoLargeSnapshotsRule | Warning | `toMatchSnapshot()` call exceeding a configurable line threshold |
| VITEST-NO-013 | NoDoneCallbackRule | Warning | `it('...', (done) => {})` — use async/await instead |
| VITEST-NO-014 | NoConditionalExpectRule | Warning | `expect()` inside `if` / `switch` / ternary — non-deterministic assertion |

### Prefer-* (VITEST-PREF-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-PREF-001 | PreferToBeRule | Warning | Use `.toBe()` for primitive comparisons instead of `.toEqual()` |
| VITEST-PREF-002 | PreferToContainRule | Warning | Use `.toContain()` instead of asserting on `.includes()` |
| VITEST-PREF-003 | PreferToHaveLengthRule | Warning | Use `.toHaveLength()` instead of asserting on `.length` |
| VITEST-PREF-005 | PreferSpyOnRule | Dependencies | Use `vi.spyOn()` instead of assigning `vi.fn()` to an object method |
| VITEST-PREF-007 | PreferCalledOnceRule | Warning | Use `.toHaveBeenCalledOnce()` instead of `.toHaveBeenCalledTimes(1)` |
| VITEST-PREF-009 | PreferHooksOnTopRule | Structure | Hooks should be placed above all test cases in a `describe` block |
| VITEST-PREF-010 | PreferHooksInOrderRule | Structure | Hooks should follow ordering: beforeAll → beforeEach → afterEach → afterAll |
| VITEST-PREF-012 | PreferTodoRule | Maintenance | Empty test bodies should use `test.todo()` instead |
| VITEST-PREF-013 | PreferMockPromiseShorthandRule | Warning | Use `.mockResolvedValue()` / `.mockRejectedValue()` instead of `mockReturnValue(Promise.resolve/reject(...))` |
| VITEST-PREF-014 | PreferExpectResolvesRule | Warning | Use `expect(promise).resolves` instead of `expect(await promise)` |

### Require-* (VITEST-REQ-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-REQ-001 | RequireHookRule | Structure | Setup/teardown logic outside hooks should be moved into `beforeEach`/`afterEach` |
| VITEST-REQ-002 | RequireTopLevelDescribeRule | Structure | All `it`/`test` blocks should be inside a `describe` block |
| VITEST-REQ-003 | RequireToThrowMessageRule | Warning | `.toThrow()` / `.rejects.toThrow()` should include an expected error message |

### Consistency (VITEST-CON-*)

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-CON-001 | ConsistentTestItRule | Maintenance | Mix of `test()` and `it()` in the same file — pick one |
| VITEST-CON-003 | ConsistentVitestViRule | Dependencies | Mix of `vi` named import and `vitest` namespace import |
| VITEST-CON-004 | HoistedApisOnTopRule | Structure | `vi.mock()` / `vi.hoisted()` calls should precede all `test`/`it`/`describe` blocks |

### Playwright (VITEST-PW-*)

Best-practice rules for Playwright E2E test files. These rules only fire when
the test runtime is detected as Playwright (via `@playwright/test` imports).

| Rule ID | Name | Severity | Description |
|---------|------|----------|-------------|
| VITEST-PW-001 | PwWaitForTimeoutRule | Warning | `waitForTimeout` causes flaky waits — prefer auto-retry assertions |
| VITEST-PW-002 | PwCssIdSelectorRule | Warning | CSS ID selector used — IDs may not be stable across app updates |
| VITEST-PW-003 | PwXPathSelectorRule | Warning | XPath selectors are brittle — prefer role/text/accessible-name locators |
| VITEST-PW-004 | PwLocatorNthRule | Warning | `.nth()` positional locator is fragile — DOM order may change |
| VITEST-PW-005 | PwPageDollarRule | Warning | `page.$` / `page.$$` returns raw ElementHandle — prefer locator API |
| VITEST-PW-006 | PwEvaluateInnerTextRule | Warning | `page.evaluate` with `innerText` — prefer ARIA text assertions or `getByText` |
| VITEST-PW-007 | PwTextAssertionOverRoleRule | Info | `getByText` used — prefer `getByRole` for interactive elements |
| VITEST-PW-008 | PwTestIdOverSemanticRoleRule | Info | Only `getByTestId` locators — prefer semantic role/text locators |
| VITEST-PW-009 | PwDuplicateSpecFileRule | Warning | Spec file name overlaps with another Playwright file — consolidate |
| VITEST-PW-010 | PwArbitrarySleepRule | Warning | Arbitrary `setTimeout` in Promise — prefer Playwright auto-waiting assertions |
| VITEST-PW-011 | PwHardCssClassChainRule | Warning | Hard CSS class selector chain — fragile to DOM structure changes |
| VITEST-PW-012 | PwMissingWebFirstAssertionRule | Warning | Playwright calls without accessor locators — use web-first assertions |
| VITEST-PW-100 | PwMissingAxeScanRule | Info | No axe accessibility scan — consider `@axe-core/playwright` |

## Configuration (`.vitest-linter.toml`)

Place a `.vitest-linter.toml` next to your `package.json`. The linter walks up
from the input path to find it.

```toml
[rules.select]
# Override severity per rule. Values: "off", "info", "warning", "error"
VITEST-FLK-001 = "off"
VITEST-MNT-003 = "info"

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

Alternatively, you can configure via `package.json`:

```json
{
  "vitest-linter": {
    "select": {
      "VITEST-FLK-001": "off",
      "VITEST-MNT-003": "info"
    }
  }
}
```

### Suppression Comments

Suppress specific rules inline:

```typescript
// vitest-linter-disable-next-line VITEST-FLK-001
test('uses timeout', () => {
    setTimeout(() => {}, 1000);
});

// vitest-linter-disable VITEST-MNT-003
test('has conditionals', () => {
    if (true) { expect(1).toBe(1); }
});
// vitest-linter-enable VITEST-MNT-003
```

If no config file exists, DEP-002 is inactive (no banlist → no
violations); DEP-001 still fires on stable dependencies (pure functions, data
models) detected via heuristics; DEP-003 still runs with its built-in defaults.

## Supported File Extensions

`.test.ts`, `.spec.ts`, `.test.tsx`, `.spec.tsx`, `.test.js`, `.spec.js`, `.test.jsx`, `.spec.jsx`

## Output Formats

| Format | Flag | Description |
|--------|------|-------------|
| terminal | `--format terminal` (default) | Colored human-readable output |
| json | `--format json` | Machine-readable JSON array |
| sarif | `--format sarif` | SARIF 2.1.0 for GitHub Code Scanning |

## Parity with pytest-linter

| pytest-linter Rule | Vitest-linter Rule | Notes |
|--------------------|--------------------|-------|
| PLR0911 (too-many-return-statements) | VITEST-STR-002 | Return in test body |
| PLR0912 (too-many-branches) | VITEST-MNT-003 | Conditional logic in test |
| PLR0915 (too-many-statements) | VITEST-MNT-002 | Too many assertions |
| PLC2201 (misplaced-comparison-constant) | — | Not applicable to JS/TS |
| PLR0401 (cyclic-import) | VITEST-DEP-001 | Module-level mocking |
| PLW0120 (useless-else-on-loop) | — | Not applicable |
| PLR0133 (comparison-of-constant) | — | Not applicable |
| PLW0602 (global-variable-undefined) | VITEST-DEP-002 | Production singleton import |
| PLW0603 (global-statement) | VITEST-DEP-003 | Reset escape hatch |
| PLR2004 (magic-value-comparison) | VITEST-FLK-005 | Non-deterministic values |
| PLW1514 (unspecified-encoding) | — | Not applicable |
| PLR0913 (too-many-arguments) | — | Not applicable |
| PLW0129 (assert-on-string-literal) | VITEST-MNT-001 | No assertions |
| PLR1714 (repeated-equality-comparison) | VITEST-MNT-002 | Multiple expects |

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
