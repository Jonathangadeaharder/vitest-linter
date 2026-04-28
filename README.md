# vitest-linter

A fast, zero-config test-smell linter for TypeScript/JavaScript Vitest test suites, written in Rust.

## Features

- **11 rules** across 3 categories: Flakiness, Maintenance, Structure
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

> **11 rules** implemented across 3 categories.  Numeric suffixes are kept in
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

## Suppression

Suppress a single violation on the next line:

```typescript
// vitest-linter-disable-next-line VITEST-FLK-001
setTimeout(() => {}, 0);
```

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
