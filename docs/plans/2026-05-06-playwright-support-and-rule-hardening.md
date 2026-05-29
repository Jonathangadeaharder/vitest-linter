# Playwright Support + Rule Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Playwright E2E test linting support and harden existing rules to catch real-world test smells from the Vidiom audit.

**Architecture:** Extend the existing tree-sitter parser to detect Playwright test runtime, add `TestRuntime` enum to `ParsedModule`, gate existing Vitest-only rules, implement 13 new `VITEST-PW-*` rules, and harden 7 existing rules.

**Tech Stack:** Rust, tree-sitter-typescript, globset, serde, tempfile (tests)

---

## PR 1: TestRuntime Detection + Runtime Gating + A6 Fix

### Task 1: Add `TestRuntime` enum and `PlaywrightModule` to models

**Files:**
- Modify: `src/models.rs`
- Test: `src/models.rs` (inline tests)

- [ ] **Step 1: Add `TestRuntime` enum and `PlaywrightModule` struct to `src/models.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TestRuntime {
    Vitest,
    Playwright,
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct PlaywrightCall {
    pub call_name: String,
    pub line: usize,
    pub raw_arg: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LocatorChain {
    pub root: String,
    pub raw_arg: Option<String>,
    pub method: String,
    pub line: usize,
}

#[derive(Debug, Clone, Default)]
pub struct PlaywrightModule {
    pub calls: Vec<PlaywrightCall>,
    pub locator_chains: Vec<LocatorChain>,
    pub evaluate_inner_text: Vec<usize>,
    pub uses_axe: bool,
    pub global_stubs: Vec<GlobalStub>,
}

#[derive(Debug, Clone)]
pub struct GlobalStub {
    pub target: String,
    pub line: usize,
}
```

Add `runtime: TestRuntime`, `playwright: Option<PlaywrightModule>`, and `global_stubs: Vec<GlobalStub>` fields to `ParsedModule`.

- [ ] **Step 2: Add `Playwright` variant to `Category` enum**

```rust
pub enum Category {
    Flakiness,
    Maintenance,
    Structure,
    Dependencies,
    Validation,
    Playwright,
}
```

- [ ] **Step 3: Update all existing tests that pattern-match on `Category`**

- [ ] **Step 4: Run `cargo test --lib` — all tests must pass**

### Task 2: Extend parser to detect Playwright runtime and global stubs

**Files:**
- Modify: `src/parser.rs`
- Test: `src/parser.rs` (inline tests)

- [ ] **Step 1: Add Playwright detection logic in `collect()` method**

When an `import_statement` is found with source `@playwright/test`, set `ctx.is_playwright = true`. Also detect `global.X = vi.fn()` patterns and `vi.stubGlobal(...)` calls.

- [ ] **Step 2: Add `is_playwright` and `playwright_calls` tracking to `Context` struct**

```rust
struct Context {
    // ... existing fields
    is_playwright: bool,
    playwright_calls: Vec<PlaywrightCall>,
    locator_chains: Vec<LocatorChain>,
    evaluate_inner_text: Vec<usize>,
    uses_axe: bool,
    global_stubs: Vec<GlobalStub>,
}
```

- [ ] **Step 3: In `handle_call`, detect `vi.stubGlobal(name, ...)` and record `GlobalStub`**

- [ ] **Step 4: Detect `global.X = vi.fn()` / `globalThis.X = vi.fn()` assignments at module scope**

Walk `lexical_declaration` and `expression_statement` nodes for assignment patterns.

- [ ] **Step 5: Populate `PlaywrightModule` and set `runtime` in `parse_file` return**

```rust
let runtime = if ctx.is_playwright {
    TestRuntime::Playwright
} else {
    TestRuntime::Vitest
};
```

- [ ] **Step 6: Write parser tests for Playwright detection and global stubs**

- [ ] **Step 7: Run `cargo test --lib` — all tests must pass**

### Task 3: Add `applies_to_runtime` method to `Rule` trait and gate existing rules

**Files:**
- Modify: `src/rules/mod.rs`
- Modify: All rule files that need gating

- [ ] **Step 1: Add default `applies_to_runtime` method to `Rule` trait**

```rust
pub trait Rule {
    // ... existing methods
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool {
        true
    }
}
```

- [ ] **Step 2: Add `runtime` field to `LintContext`**

```rust
pub struct LintContext<'a> {
    pub config: &'a Config,
    pub all_modules: &'a [ParsedModule],
}
```

(Runtime info comes from each `ParsedModule` — no need to add to context.)

- [ ] **Step 3: In `engine.rs`, skip rules that don't apply to the module's runtime**

Before calling `rule.check(module, ...)`, check `rule.applies_to_runtime(module.runtime)`.

- [ ] **Step 4: Implement `applies_to_runtime` for Vitest-only rules**

Most rules return `runtime != Playwright` (i.e., they only fire for Vitest/Unknown). Override for rules that should also fire on Playwright:

- `NoAssertionRule` → `true` (fires on both)
- `FocusedTestRule` → `true` (fires on both, per A6)
- `EmptyTestRule` → `true` (fires on both)
- `NoIdenticalTitleRule` → `true` (fires on both)

- [ ] **Step 5: Run `cargo test` — all 155 tests must pass**

### Task 4: Fix A6 — FocusedTestRule fires on Playwright `test.only`

**Files:**
- Modify: `src/parser.rs` (already done — `parse_callee` handles both)
- Test: `tests/integration_tests.rs`

- [ ] **Step 1: Write integration test for Playwright `test.only`**

```rust
#[test]
fn mnt007_playwright_test_only() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "pw-only.spec.ts", r#"
import { test, expect } from '@playwright/test';

test.only('focused pw test', async ({ page }) => {
    await expect(page).toHaveTitle(/app/);
});
"#);
    let engine = LintEngine::new().unwrap();
    let (violations, _) = engine.lint_paths(&[path]).unwrap();
    let v = find_violation(&violations, "VITEST-MNT-007");
    assert!(v.is_some(), "Expected VITEST-MNT-007 for Playwright test.only");
}
```

- [ ] **Step 2: Verify parser correctly sets `is_only` for `test.only` from `@playwright/test`**

The parser already handles `.only` detection via `parse_callee` — verify it works when the binding comes from `@playwright/test`.

- [ ] **Step 3: Run `cargo test` — must pass**

---

## PR 2: Easy AST-Match Playwright Rules + A5 Timeout Hardening

### Task 5: Implement PW-001 `PwWaitForTimeoutRule`

**Files:**
- Create: `src/rules/playwright.rs`
- Modify: `src/rules/mod.rs`
- Test: `tests/integration_tests.rs`

- [ ] **Step 1: Write failing integration test**

- [ ] **Step 2: Implement rule** — flag `page.waitForTimeout(...)` / `waitForTimeout(...)` calls in Playwright files

- [ ] **Step 3: Run test — must pass**

### Task 6: Implement PW-003 `PwXPathSelectorRule`

- [ ] Flag locators using XPath: `xpath=`, `//`, `.xpath()`

### Task 7: Implement PW-004 `PwLocatorNthRule`

- [ ] Flag `.nth(N)` positional locators

### Task 8: Implement PW-005 `PwPageDollarRule`

- [ ] Flag `page.$(...)` and `page.$$(...)` raw queries

### Task 9: Implement PW-010 `PwArbitrarySleepRule`

- [ ] Flag `await new Promise(r => setTimeout(r, N))` in Playwright files

### Task 10: Fix A5 — TimeoutRule catches Promise-wrapped setTimeout

- [ ] Verify parser detects `setTimeout` inside `Promise` constructor
- [ ] Add integration test fixture

---

## PR 3: Selector Classifier + PW-002, PW-006, PW-011 + A2 DEP-001 Hardening

### Task 11: Implement `classify_selector` helper

- [ ] Pure function with 30+ test fixtures per class

### Task 12: Implement PW-002 `PwCssIdSelectorRule`

- [ ] Flag CSS ID selectors: `#id`, `input#name`

### Task 13: Implement PW-006 `PwEvaluateInnerTextRule`

- [ ] Flag `page.evaluate(() => document.body.innerText)`

### Task 14: Implement PW-011 `PwHardCssClassChainRule`

- [ ] Flag `.foo > .bar` descendant/child chains

### Task 15: Harden A2 — BannedModuleMockRule stable-dep classifier

- [ ] Add path segment + file suffix matching for stable deps
- [ ] Add integration-context boost
- [ ] Add config knobs

---

## PR 4: File-Graph Rule + PW-009

### Task 16: Implement PW-009 `PwDuplicateSpecFileRule`

- [ ] Flag duplicate spec files (e.g., `foo.spec.ts` + `foo.spec 2.ts`)

---

## PR 5: Heuristic Rules + Part A Hardening (A1, A3, A4, A7)

### Task 17: Implement PW-007 `PwTextAssertionOverRoleRule`

### Task 18: Implement PW-008 `PwTestIdOverSemanticRoleRule`

### Task 19: Implement PW-012 `PwMissingWebFirstAssertionRule`

### Task 20: Implement PW-100 `PwMissingAxeScanRule`

### Task 21: Harden A1 — MissingMockCleanupRule for global stubs

### Task 22: Harden A3 — WeakAssertionRule extended matchers

### Task 23: Harden A4 — ImplementationCoupledRule testid negative-presence

### Task 24: Harden A7 — PreferToHaveLengthRule extended patterns

---

## Final: Update docs and README

### Task 25: Update `docs/rules/` entries for all new rules

### Task 26: Update `README.md` rule table

### Task 27: Update `all_rules()` count assertion and `all_rule_ids_present` test
