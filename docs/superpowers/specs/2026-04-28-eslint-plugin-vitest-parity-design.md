# Vitest-Linter Parity with eslint-plugin-vitest

## Goal

Achieve parity with the high-impact rules from `eslint-plugin-vitest`, expanding from 18 to **57 rules** (+39 new).

## Design Principles

Every rule must pass this test: **"Does this rule find actual bad tests that would cause real problems?"**

- Style-only rules that don't affect correctness → Skip
- Rules that require user configuration to be useful → Skip  
- Rules that are too opinionated or niche → Skip
- Rules that catch real bugs, flakiness, or maintenance issues → Keep

## Rule Analysis

### Epic 1: Valid Rules (valid-*) — 5 rules

These enforce correct Vitest API usage. All catch real bugs.

| Rule ID | Name | Verdict | Reasoning |
|---------|------|---------|-----------|
| VITEST-VAL-001 | `valid-expect` | **KEEP** | `expect()` without assertion is a silent false-positive test. Critical. |
| VITEST-VAL-002 | `valid-expect-in-promise` | **KEEP** | Unreturned promise with expect = test passes regardless of assertion. Silent bug. |
| VITEST-VAL-003 | `valid-describe-callback` | **KEEP** | Async describe callback, wrong callback type = tests never run. |
| VITEST-VAL-004 | `valid-title` | **KEEP** | Empty/template-literal titles break CI output readability. |
| VITEST-VAL-005 | `no-unneeded-async-expect-function` | **KEEP** | `async () => expect(await p)` masks rejection; `expect(p).resolves` is correct. |

### Epic 2: No Rules (no-*) — 12 rules (skipped 2)

| Rule ID | Name | Verdict | Reasoning |
|---------|------|---------|-----------|
| VITEST-NO-001 | `no-standalone-expect` | **KEEP** | Expect outside it/test = test never runs, false pass. |
| VITEST-NO-002 | `no-identical-title` | **KEEP** | Duplicate titles confuse CI output, hide failures. |
| VITEST-NO-003 | `no-commented-out-tests` | **KEEP** | Dead code. If intentional, use skip/todo instead. |
| VITEST-NO-004 | `no-alias-methods` | **SKIP** | `toBeCalled` vs `toHaveBeenCalledWith` — both work, purely stylistic. |
| VITEST-NO-005 | `no-test-prefixes` | **KEEP** | `fit`/`xit` are legacy; `.only`/`.skip` are modern. Prevents accidental focus. |
| VITEST-NO-006 | `no-duplicate-hooks` | **KEEP** | Duplicate beforeEach = setup runs twice, wastes time, may cause bugs. |
| VITEST-NO-007 | `no-import-node-test` | **KEEP** | Mixing node:test with vitest = two test runners, undefined behavior. |
| VITEST-NO-008 | `no-interpolation-in-snapshots` | **KEEP** | Dynamic snapshots change every run, defeating snapshot purpose. |
| VITEST-NO-009 | `no-large-snapshots` | **KEEP** | Huge snapshots are unmaintainable and slow down reviews. |
| VITEST-NO-010 | `no-mocks-import` | **SKIP** | Some teams use __mocks__ intentionally. Too opinionated. |
| VITEST-NO-011 | `no-restricted-matchers` | **SKIP** | Requires user configuration. We already have config system for this. |
| VITEST-NO-012 | `no-restricted-vi-methods` | **SKIP** | Requires user configuration. |
| VITEST-NO-013 | `no-done-callback` | **KEEP** | `done` callback is error-prone (forgotten call = hanging test). Modern async/await is better. |
| VITEST-NO-014 | `no-conditional-expect` | **KEEP** | `if (x) expect(y)` = test passes when x is false. Silent false-positive. |

### Epic 3: Prefer Rules (prefer-*) — 11 rules (skipped 7)

| Rule ID | Name | Verdict | Reasoning |
|---------|------|---------|-----------|
| VITEST-PREF-001 | `prefer-to-be` | **KEEP** | `toBe` is faster for primitives, `toEqual` does deep comparison unnecessarily. |
| VITEST-PREF-002 | `prefer-to-contain` | **KEEP** | `expect(arr).toContain(x)` is clearer than `expect(arr.includes(x)).toBe(true)`. |
| VITEST-PREF-003 | `prefer-to-have-length` | **KEEP** | `toHaveLength(3)` is more readable than `toBe(3)` on `.length`. |
| VITEST-PREF-004 | `prefer-strict-equal` | **SKIP** | `toStrictEqual` catches extra properties, but many tests intentionally use `toEqual`. Too strict. |
| VITEST-PREF-005 | `prefer-spy-on` | **KEEP** | `window.fetch = mock` pollutes global state; `vi.spyOn` auto-restores. |
| VITEST-PREF-006 | `prefer-called-with` | **SKIP** | Sometimes `toHaveBeenCalled()` is sufficient. Not always better. |
| VITEST-PREF-007 | `prefer-called-once` | **KEEP** | `toHaveBeenCalledOnce()` is clearer than `toHaveBeenCalledTimes(1)`. |
| VITEST-PREF-008 | `prefer-called-times` | **SKIP** | `toHaveBeenCalledTimes(n)` is already explicit. No improvement. |
| VITEST-PREF-009 | `prefer-hooks-on-top` | **KEEP** | Hooks after tests = confusing setup order, easy to miss. |
| VITEST-PREF-010 | `prefer-hooks-in-order` | **KEEP** | Consistent beforeAll→beforeEach→afterEach→afterAll order aids readability. |
| VITEST-PREF-011 | `prefer-each` | **SKIP** | Manual loops are sometimes clearer. Too opinionated. |
| VITEST-PREF-012 | `prefer-todo` | **KEEP** | `test.todo` is explicit about planned tests; empty `it()` is confusing. |
| VITEST-PREF-013 | `prefer-mock-promise-shorthand` | **KEEP** | `mockResolvedValue(x)` is clearer than `mockReturnValue(Promise.resolve(x))`. |
| VITEST-PREF-014 | `prefer-expect-resolves` | **KEEP** | `expect(p).resolves.toBe(x)` is safer than `expect(await p).toBe(x)`. |
| VITEST-PREF-015 | `prefer-vi-mocked` | **SKIP** | `vi.mocked()` vs `as Mock` — both work, TS handles type safety. |
| VITEST-PREF-016 | `prefer-comparison-matcher` | **SKIP** | `toBeGreaterThan` vs `> x &&` — marginal improvement, not worth enforcing. |
| VITEST-PREF-017 | `prefer-equality-matcher` | **SKIP** | Redundant with prefer-to-be. |
| VITEST-PREF-018 | `prefer-lowercase-title` | **SKIP** | Style-only, doesn't affect test quality. |

### Epic 4: Require Rules (require-*) — 3 rules (skipped 2)

| Rule ID | Name | Verdict | Reasoning |
|---------|------|---------|-----------|
| VITEST-REQ-001 | `require-hook` | **KEEP** | Top-level `vi.mock()` or variable mutation = shared state across tests. Real bug source. |
| VITEST-REQ-002 | `require-top-level-describe` | **KEEP** | Orphan tests outside describe = poor organization, harder to debug failures. |
| VITEST-REQ-003 | `require-to-throw-message` | **KEEP** | `expect(fn).toThrow()` without message = catches any error, masks real failures. |
| VITEST-REQ-004 | `require-test-timeout` | **SKIP** | Too strict. Most tests don't need explicit timeouts. |
| VITEST-REQ-005 | `require-local-test-context` | **SKIP** | Too niche (only for concurrent snapshots). |

### Epic 5: Consistency Rules — 3 rules (skipped 1)

| Rule ID | Name | Verdict | Reasoning |
|---------|------|---------|-----------|
| VITEST-CON-001 | `consistent-test-it` | **KEEP** | Mixing `test` and `it` in same file is confusing. Pick one. |
| VITEST-CON-002 | `consistent-test-filename` | **SKIP** | Too opinionated. Teams have different conventions. |
| VITEST-CON-003 | `consistent-vitest-vi` | **KEEP** | Mixing `import { test } from 'vitest'` and `vi.fn()` = confusing imports. |
| VITEST-CON-004 | `hoisted-apis-on-top` | **KEEP** | `vi.mock()` hoisting is confusing; putting it at top makes behavior explicit. |

---

## Final Rule Count

| Category | Rules | Skipped |
|----------|-------|---------|
| valid-* | 5 | 0 |
| no-* | 12 | 2 |
| prefer-* | 11 | 7 |
| require-* | 3 | 2 |
| consistency | 3 | 1 |
| **Total new** | **34** | **12** |
| **Total after** | **52** | — |

## Implementation Plan

5 Epics, each independently shippable:

### Epic E11: Valid Rules (5 rules)
- VITEST-VAL-001 through VITEST-VAL-005
- Focus: Catch tests that pass incorrectly

### Epic E12: No Rules (12 rules)  
- VITEST-NO-001 through VITEST-NO-014
- Focus: Catch bad patterns and dead code

### Epic E13: Prefer Rules (11 rules)
- VITEST-PREF-001 through VITEST-PREF-014
- Focus: Enforce idiomatic Vitest usage

### Epic E14: Require Rules (3 rules)
- VITEST-REQ-001 through VITEST-REQ-003
- Focus: Enforce safe patterns

### Epic E15: Consistency Rules (3 rules)
- VITEST-CON-001, VITEST-CON-003, VITEST-CON-004
- Focus: Enforce consistent style

## Technical Notes

### Rule ID Naming
- `VITEST-VAL-*` for valid rules
- `VITEST-NO-*` for no rules  
- `VITEST-PREF-*` for prefer rules
- `VITEST-REQ-*` for require rules
- `VITEST-CON-*` for consistency rules

### Implementation Pattern
Each rule:
1. Struct implementing `Rule` trait
2. `id()`, `name()`, `severity()`, `check()` methods
3. Tree-sitter AST analysis
4. Integration test per rule
5. Unit tests within module

### Severity Mapping
- Error: Rules that catch definite bugs (valid-*, no-standalone-expect, no-identical-title)
- Warning: Rules that catch likely bad patterns (most no-*, prefer-*, require-*)
- Info: Style suggestions (consistency rules)

### Configuration
Rules that need configuration (like max-nested-describe) will use existing `.vitest-linter.toml` system.

## What We Skip (and Why)

| Skipped Rule | Reason |
|--------------|--------|
| no-alias-methods | Stylistic only, both forms work |
| no-mocks-import | Too opinionated, some teams use __mocks__ |
| no-restricted-matchers | Already handled by config system |
| no-restricted-vi-methods | Already handled by config system |
| prefer-strict-equal | Too strict for general use |
| prefer-called-with | Not always better than toHaveBeenCalled |
| prefer-called-times | Already explicit |
| prefer-each | Manual loops sometimes clearer |
| prefer-vi-mocked | TS handles type safety |
| prefer-comparison-matcher | Marginal improvement |
| prefer-equality-matcher | Redundant |
| prefer-lowercase-title | Style only |
| consistent-test-filename | Too opinionated |
| require-test-timeout | Too strict for most codebases |
| require-local-test-context | Too niche |
