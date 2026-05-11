# Vitest-Linter Parity with eslint-plugin-vitest — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 34 new rules to achieve parity with high-impact rules from eslint-plugin-vitest, expanding from 18 to 52 rules.

**Architecture:** Each rule is a struct implementing the `Rule` trait, with `id()`, `name()`, `severity()`, `category()`, and `check()` methods. Rules analyze `ParsedModule` AST data and return `Vec<Violation>`. New parser fields will be added as needed for rules that require additional AST information.

**Tech Stack:** Rust, tree-sitter (TypeScript/TSX), serde, walkdir

---

## File Structure

### Files to Create
- `src/rules/validation.rs` — valid-* rules (Epic E11)
- `src/rules/no_rules.rs` — no-* rules (Epic E12)
- `src/rules/prefer.rs` — prefer-* rules (Epic E13)
- `src/rules/require.rs` — require-* rules (Epic E14)
- `src/rules/consistency.rs` — consistency rules (Epic E15)

### Files to Modify
- `src/models.rs` — Add new fields to TestBlock, ParsedModule, DescribeBlock
- `src/parser.rs` — Extract additional AST information for new rules
- `src/rules/mod.rs` — Register all new rules, add module declarations
- `tests/integration_tests.rs` — Integration tests for each rule

---

## Epic E11: Valid Rules (5 rules)

### Task 1: Add Category::Validation to models.rs

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Add Validation category**

```rust
/// Category grouping for lint rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Category {
    Flakiness,
    Maintenance,
    Structure,
    Dependencies,
    Validation,  // NEW
}
```

- [ ] **Step 2: Update category test**

In `src/models.rs`, update the `category_values` test:

```rust
#[test]
fn category_values() {
    assert_ne!(Category::Flakiness, Category::Maintenance);
    assert_ne!(Category::Maintenance, Category::Structure);
    assert_ne!(Category::Flakiness, Category::Structure);
    assert_ne!(Category::Validation, Category::Flakiness);
}
```

- [ ] **Step 3: Commit**

```bash
git add src/models.rs
git commit -m "feat: add Validation category to models"
```

---

### Task 2: Add parser fields for valid-* rules

**Files:**
- Modify: `src/models.rs`
- Modify: `src/parser.rs`

- [ ] **Step 1: Add fields to TestBlock**

```rust
pub struct TestBlock {
    // ... existing fields ...
    pub has_expect_call_without_assertion: bool,  // expect() without .toBe/.toEqual etc
    pub has_return_of_expect: bool,  // return expect(...)
    pub describe_name: Option<String>,  // parent describe name for title validation
}
```

- [ ] **Step 2: Add parsing logic**

In `src/parser.rs`, in the test extraction logic, detect:
- `expect()` calls that don't chain to an assertion method
- `return expect(...)` patterns
- Parent describe block name

- [ ] **Step 3: Commit**

```bash
git add src/models.rs src/parser.rs
git commit -m "feat: add parser fields for valid-* rules"
```

---

### Task 3: Create validation.rs with ValidExpectRule

**Files:**
- Create: `src/rules/validation.rs`

- [ ] **Step 1: Create validation.rs**

```rust
use crate::models::{Category, ParsedModule, Severity, Violation};
use crate::rules::Rule;

/// Enforces valid `expect()` usage — every `expect()` call must chain
/// to an assertion method (`.toBe`, `.toEqual`, etc.).
pub struct ValidExpectRule;

impl Rule for ValidExpectRule {
    fn id(&self) -> &'static str {
        "VITEST-VAL-001"
    }
    fn name(&self) -> &'static str {
        "ValidExpectRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Validation
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_expect_call_without_assertion)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "expect() call without assertion method — test always passes".to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Chain an assertion: expect(x).toBe(y), expect(x).toEqual(y), etc.".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/rules/validation.rs
git commit -m "feat: add VITEST-VAL-001 ValidExpectRule"
```

---

### Task 4: Add ValidExpectInPromiseRule

**Files:**
- Modify: `src/rules/validation.rs`

- [ ] **Step 1: Implement rule**

```rust
/// Enforces that promises containing `expect` are properly awaited or returned.
/// `expect(await promise)` masks rejections.
pub struct ValidExpectInPromiseRule;

impl Rule for ValidExpectInPromiseRule {
    fn id(&self) -> &'static str {
        "VITEST-VAL-002"
    }
    fn name(&self) -> &'static str {
        "ValidExpectInPromiseRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Validation
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        // Detection: async test with expect(await ...) pattern
        // Should use expect(...).resolves instead
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.unawaited_async_assertions > 0)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "Test has {} unawaited async assertion(s) — use expect().resolves",
                    tb.unawaited_async_assertions
                ),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use expect(promise).resolves.toBe(x) instead of expect(await promise).toBe(x)".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/rules/validation.rs
git commit -m "feat: add VITEST-VAL-002 ValidExpectInPromiseRule"
```

---

### Task 5: Add ValidDescribeCallbackRule

**Files:**
- Modify: `src/rules/validation.rs`
- Modify: `src/models.rs` (add describe callback info)

- [ ] **Step 1: Add field to DescribeBlock**

```rust
pub struct DescribeBlock {
    // ... existing fields ...
    pub is_async: bool,  // describe callback is async
}
```

- [ ] **Step 2: Implement rule**

```rust
/// Enforces valid describe callbacks — async callbacks, proper function types.
pub struct ValidDescribeCallbackRule;

impl Rule for ValidDescribeCallbackRule {
    fn id(&self) -> &'static str {
        "VITEST-VAL-003"
    }
    fn name(&self) -> &'static str {
        "ValidDescribeCallbackRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Validation
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        module
            .describe_blocks
            .iter()
            .filter(|db| db.is_async)
            .map(|db| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "describe() callback should not be async — tests inside may not run".to_string(),
                file_path: db.file_path.clone(),
                line: db.line,
                col: None,
                suggestion: Some(
                    "Remove async from describe callback. Use async on individual tests instead.".to_string(),
                ),
                test_name: Some(db.name.clone()),
            })
            .collect()
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/validation.rs src/models.rs
git commit -m "feat: add VITEST-VAL-003 ValidDescribeCallbackRule"
```

---

### Task 6: Add ValidTitleRule

**Files:**
- Modify: `src/rules/validation.rs`
- Modify: `src/models.rs` (add title string info)

- [ ] **Step 1: Add fields**

```rust
pub struct TestBlock {
    // ... existing fields ...
    pub title_is_template_literal: bool,
}

pub struct DescribeBlock {
    // ... existing fields ...
    pub title_is_template_literal: bool,
    pub title_is_empty: bool,
}
```

- [ ] **Step 2: Implement rule**

```rust
/// Enforces valid titles — no empty titles, no template literals.
pub struct ValidTitleRule;

impl Rule for ValidTitleRule {
    fn id(&self) -> &'static str {
        "VITEST-VAL-004"
    }
    fn name(&self) -> &'static str {
        "ValidTitleRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Validation
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        let mut violations = Vec::new();

        for tb in &module.test_blocks {
            if tb.title_is_template_literal {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: "Test title should not be a template literal — use a static string".to_string(),
                    file_path: tb.file_path.clone(),
                    line: tb.line,
                    col: None,
                    suggestion: Some("Use a plain string for the test title".to_string()),
                    test_name: Some(tb.name.clone()),
                });
            }
        }

        for db in &module.describe_blocks {
            if db.title_is_empty {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: "describe() block has empty title".to_string(),
                    file_path: db.file_path.clone(),
                    line: db.line,
                    col: None,
                    suggestion: Some("Add a descriptive title to the describe block".to_string()),
                    test_name: Some(db.name.clone()),
                });
            }
        }

        violations
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/validation.rs src/models.rs
git commit -m "feat: add VITEST-VAL-004 ValidTitleRule"
```

---

### Task 7: Add NoUnneededAsyncExpectFunctionRule

**Files:**
- Modify: `src/rules/validation.rs`
- Modify: `src/models.rs`

- [ ] **Step 1: Add field**

```rust
pub struct TestBlock {
    // ... existing fields ...
    pub has_async_expect_wrapper: bool,  // async () => expect(await ...)
}
```

- [ ] **Step 2: Implement rule**

```rust
/// Disallows unnecessary async function wrappers around expect.
/// `async () => expect(await promise)` should be `expect(promise).resolves`.
pub struct NoUnneededAsyncExpectFunctionRule;

impl Rule for NoUnneededAsyncExpectFunctionRule {
    fn id(&self) -> &'static str {
        "VITEST-VAL-005"
    }
    fn name(&self) -> &'static str {
        "NoUnneededAsyncExpectFunctionRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Validation
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_async_expect_wrapper)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Unnecessary async wrapper — use expect().resolves instead".to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Replace `async () => expect(await p).toBe(x)` with `() => expect(p).resolves.toBe(x)`".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/validation.rs src/models.rs
git commit -m "feat: add VITEST-VAL-005 NoUnneededAsyncExpectFunctionRule"
```

---

### Task 8: Register validation rules and test

**Files:**
- Modify: `src/rules/mod.rs`

- [ ] **Step 1: Add module and register rules**

```rust
pub mod validation;

// In all_rules():
Box::new(validation::ValidExpectRule),
Box::new(validation::ValidExpectInPromiseRule),
Box::new(validation::ValidDescribeCallbackRule),
Box::new(validation::ValidTitleRule),
Box::new(validation::NoUnneededAsyncExpectFunctionRule),
```

- [ ] **Step 2: Update rule count test**

```rust
#[test]
fn all_rules_count() {
    let rules = all_rules();
    assert_eq!(rules.len(), 23); // 18 + 5
}
```

- [ ] **Step 3: Add integration tests**

In `tests/integration_tests.rs`:

```rust
#[test]
fn val001_expect_without_assertion() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "val001.test.ts", r#"
import { test, expect } from 'vitest';
test('bad', () => {
    expect(true);
});
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-VAL-001").is_some());
}

#[test]
fn val001_expect_with_assertion_passes() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "val001_good.test.ts", r#"
import { test, expect } from 'vitest';
test('good', () => {
    expect(true).toBe(true);
});
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-VAL-001").is_none());
}

#[test]
fn val003_async_describe() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "val003.test.ts", r#"
import { describe, test, expect } from 'vitest';
describe('bad', async () => {
    test('ok', () => { expect(true).toBe(true); });
});
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-VAL-003").is_some());
}
```

- [ ] **Step 4: Run tests and verify**

```bash
cargo test
cargo fmt
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: register and test E11 validation rules (5 rules)"
```

---

## Epic E12: No Rules (12 rules)

### Task 9: Create no_rules.rs with NoStandaloneExpectRule

**Files:**
- Create: `src/rules/no_rules.rs`
- Modify: `src/models.rs` (add field for expect outside test)

- [ ] **Step 1: Add field to ParsedModule**

```rust
pub struct ParsedModule {
    // ... existing fields ...
    pub expects_outside_tests: Vec<ExpectOutsideTest>,
}

pub struct ExpectOutsideTest {
    pub line: usize,
    pub file_path: PathBuf,
}
```

- [ ] **Step 2: Create no_rules.rs**

```rust
use crate::models::{Category, ParsedModule, Severity, Violation};
use crate::rules::Rule;

/// Disallows `expect()` calls outside of `it`/`test` blocks.
pub struct NoStandaloneExpectRule;

impl Rule for NoStandaloneExpectRule {
    fn id(&self) -> &'static str {
        "VITEST-NO-001"
    }
    fn name(&self) -> &'static str {
        "NoStandaloneExpectRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        module
            .expects_outside_tests
            .iter()
            .map(|e| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "expect() outside of test block — assertion never runs".to_string(),
                file_path: e.file_path.clone(),
                line: e.line,
                col: None,
                suggestion: Some("Move expect() inside a test() or it() block".to_string()),
                test_name: None,
            })
            .collect()
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/no_rules.rs src/models.rs
git commit -m "feat: add VITEST-NO-001 NoStandaloneExpectRule"
```

---

### Task 10: Add NoIdenticalTitleRule

**Files:**
- Modify: `src/rules/no_rules.rs`

- [ ] **Step 1: Implement rule**

```rust
/// Disallows identical test/describe titles within the same file.
pub struct NoIdenticalTitleRule;

impl Rule for NoIdenticalTitleRule {
    fn id(&self) -> &'static str {
        "VITEST-NO-002"
    }
    fn name(&self) -> &'static str {
        "NoIdenticalTitleRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut seen_tests = std::collections::HashSet::new();
        let mut seen_describes = std::collections::HashSet::new();

        for tb in &module.test_blocks {
            if !seen_tests.insert(&tb.name) {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: format!("Duplicate test title: '{}'", tb.name),
                    file_path: tb.file_path.clone(),
                    line: tb.line,
                    col: None,
                    suggestion: Some("Use a unique test title".to_string()),
                    test_name: Some(tb.name.clone()),
                });
            }
        }

        for db in &module.describe_blocks {
            if !seen_describes.insert(&db.name) {
                violations.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: format!("Duplicate describe title: '{}'", db.name),
                    file_path: db.file_path.clone(),
                    line: db.line,
                    col: None,
                    suggestion: Some("Use a unique describe title".to_string()),
                    test_name: Some(db.name.clone()),
                });
            }
        }

        violations
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/rules/no_rules.rs
git commit -m "feat: add VITEST-NO-002 NoIdenticalTitleRule"
```

---

### Task 11: Add remaining no-* rules

**Files:**
- Modify: `src/rules/no_rules.rs`
- Modify: `src/models.rs`
- Modify: `src/parser.rs`

- [ ] **Step 1: Add parser fields**

```rust
pub struct TestBlock {
    // ... existing fields ...
    pub has_comment_above: bool,  // commented out test
    pub uses_fit_or_xit: bool,  // f/x prefixes
}

pub struct HookCall {
    // ... existing fields ...
    pub is_duplicate: bool,  // duplicate hook
}

pub struct ParsedModule {
    // ... existing fields ...
    pub imports_node_test: bool,
    pub has_snapshot_interpolation: bool,
    pub snapshot_lines: Vec<usize>,  // lines with snapshot assertions
    pub snapshot_sizes: Vec<(usize, usize)>,  // (line, char_count)
    pub imports_from_mocks_dir: bool,
    pub has_done_callback: bool,
    pub has_conditional_expect: bool,
}
```

- [ ] **Step 2: Implement rules**

```rust
/// Disallows commented-out tests.
pub struct NoCommentedOutTestsRule;
// ... (VITEST-NO-003)

/// Disallows fit/xit prefixes — use .only/.skip instead.
pub struct NoTestPrefixesRule;
// ... (VITEST-NO-005)

/// Disallows duplicate hooks.
pub struct NoDuplicateHooksRule;
// ... (VITEST-NO-006)

/// Disallows importing from node:test.
pub struct NoImportNodeTestRule;
// ... (VITEST-NO-007)

/// Disallows string interpolation in snapshots.
pub struct NoInterpolationInSnapshotsRule;
// ... (VITEST-NO-008)

/// Disallows large snapshots (configurable threshold).
pub struct NoLargeSnapshotsRule;
// ... (VITEST-NO-009)

/// Disallows done callback pattern.
pub struct NoDoneCallbackRule;
// ... (VITEST-NO-013)

/// Disallows expect in conditional blocks.
pub struct NoConditionalExpectRule;
// ... (VITEST-NO-014)
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/no_rules.rs src/models.rs src/parser.rs
git commit -m "feat: add remaining no-* rules (12 total)"
```

---

### Task 12: Register no-* rules and test

**Files:**
- Modify: `src/rules/mod.rs`
- Modify: `tests/integration_tests.rs`

- [ ] **Step 1: Register rules**

```rust
pub mod no_rules;

// In all_rules():
Box::new(no_rules::NoStandaloneExpectRule),
Box::new(no_rules::NoIdenticalTitleRule),
Box::new(no_rules::NoCommentedOutTestsRule),
Box::new(no_rules::NoTestPrefixesRule),
Box::new(no_rules::NoDuplicateHooksRule),
Box::new(no_rules::NoImportNodeTestRule),
Box::new(no_rules::NoInterpolationInSnapshotsRule),
Box::new(no_rules::NoLargeSnapshotsRule),
Box::new(no_rules::NoDoneCallbackRule),
Box::new(no_rules::NoConditionalExpectRule),
```

- [ ] **Step 2: Add integration tests**

```rust
#[test]
fn no001_standalone_expect() {
    // Test that expect outside test block is caught
}

#[test]
fn no002_identical_titles() {
    // Test that duplicate titles are caught
}

#[test]
fn no005_fit_prefix() {
    // Test that fit/xit are caught
}

// ... etc for each rule
```

- [ ] **Step 3: Run tests and verify**

```bash
cargo test
cargo fmt
cargo clippy
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: register and test E12 no-* rules (12 rules)"
```

---

## Epic E13: Prefer Rules (11 rules)

### Task 13: Create prefer.rs

**Files:**
- Create: `src/rules/prefer.rs`
- Modify: `src/models.rs`
- Modify: `src/parser.rs`

- [ ] **Step 1: Add parser fields for prefer rules**

```rust
pub struct TestBlock {
    // ... existing fields ...
    pub uses_to_equal_for_primitive: bool,
    pub uses_includes_in_expect: bool,
    pub uses_length_check: bool,
    pub assigns_to_global_mock: bool,
    pub hook_order: Vec<HookKind>,  // actual order of hooks
    pub has_manual_loop: bool,  // for loop in test
    pub uses_mock_return_value_promise: bool,
}
```

- [ ] **Step 2: Implement rules**

```rust
/// Prefer toBe() over toEqual() for primitive comparisons.
pub struct PreferToBeRule;  // VITEST-PREF-001

/// Prefer toContain() over includes() check.
pub struct PreferToContainRule;  // VITEST-PREF-002

/// Prefer toHaveLength() over .length check.
pub struct PreferToHaveLengthRule;  // VITEST-PREF-003

/// Prefer vi.spyOn over global mock assignment.
pub struct PreferSpyOnRule;  // VITEST-PREF-005

/// Prefer toHaveBeenCalledOnce().
pub struct PreferCalledOnceRule;  // VITEST-PREF-007

/// Prefer hooks before test cases.
pub struct PreferHooksOnTopRule;  // VITEST-PREF-009

/// Prefer consistent hook ordering.
pub struct PreferHooksInOrderRule;  // VITEST-PREF-010

/// Prefer test.todo over empty tests.
pub struct PreferTodoRule;  // VITEST-PREF-012

/// Prefer mockResolvedValue over mockReturnValue(Promise.resolve()).
pub struct PreferMockPromiseShorthandRule;  // VITEST-PREF-013

/// Prefer expect().resolves over expect(await ...).
pub struct PreferExpectResolvesRule;  // VITEST-PREF-014
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/prefer.rs src/models.rs src/parser.rs
git commit -m "feat: add prefer-* rules (11 rules)"
```

---

### Task 14: Register prefer rules and test

**Files:**
- Modify: `src/rules/mod.rs`
- Modify: `tests/integration_tests.rs`

- [ ] **Step 1: Register rules**

```rust
pub mod prefer;

// In all_rules(): add all 11 prefer rules
```

- [ ] **Step 2: Add integration tests**

```rust
#[test]
fn pref001_to_equal_for_primitive() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "pref001.test.ts", r#"
import { test, expect } from 'vitest';
test('bad', () => { expect(1).toEqual(1); });
test('good', () => { expect(1).toBe(1); });
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-PREF-001").is_some());
}

// ... etc
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: register and test E13 prefer-* rules (11 rules)"
```

---

## Epic E14: Require Rules (3 rules)

### Task 15: Create require.rs

**Files:**
- Create: `src/rules/require.rs`
- Modify: `src/models.rs`

- [ ] **Step 1: Add parser fields**

```rust
pub struct ParsedModule {
    // ... existing fields ...
    pub has_top_level_code: bool,  // code outside describe blocks
    pub has_throw_without_message: bool,
}
```

- [ ] **Step 2: Implement rules**

```rust
/// Require setup/teardown to be within hooks, not top-level.
pub struct RequireHookRule;  // VITEST-REQ-001

/// Require all tests to be in top-level describe blocks.
pub struct RequireTopLevelDescribeRule;  // VITEST-REQ-002

/// Require toThrow() to include error message.
pub struct RequireToThrowMessageRule;  // VITEST-REQ-003
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/require.rs src/models.rs
git commit -m "feat: add require-* rules (3 rules)"
```

---

### Task 16: Register require rules and test

**Files:**
- Modify: `src/rules/mod.rs`
- Modify: `tests/integration_tests.rs`

- [ ] **Step 1: Register rules**

```rust
pub mod require;

// In all_rules(): add all 3 require rules
```

- [ ] **Step 2: Add integration tests**

```rust
#[test]
fn req001_top_level_code() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "req001.test.ts", r#"
import { vi, describe, test, expect } from 'vitest';
vi.mock('fs');  // BAD: top-level mock outside describe
describe('suite', () => {
    test('ok', () => { expect(true).toBe(true); });
});
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-REQ-001").is_some());
}

#[test]
fn req002_orphan_test() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "req002.test.ts", r#"
import { test, expect } from 'vitest';
test('orphan', () => { expect(true).toBe(true); });
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-REQ-002").is_some());
}
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: register and test E14 require-* rules (3 rules)"
```

---

## Epic E15: Consistency Rules (3 rules)

### Task 17: Create consistency.rs

**Files:**
- Create: `src/rules/consistency.rs`
- Modify: `src/models.rs`
- Modify: `src/parser.rs`

- [ ] **Step 1: Add parser fields**

```rust
pub struct ParsedModule {
    // ... existing fields ...
    pub uses_test_keyword: bool,
    pub uses_it_keyword: bool,
    pub uses_vi_import: bool,
    pub uses_vitest_import: bool,
    pub has_mocks_not_at_top: bool,  // vi.mock() not at top of file
}
```

- [ ] **Step 2: Implement rules**

```rust
/// Enforce using test OR it, not both in same file.
pub struct ConsistentTestItRule;  // VITEST-CON-001

/// Enforce consistent imports (vi OR vitest).
pub struct ConsistentVitestViRule;  // VITEST-CON-003

/// Enforce vi.mock() calls at top of file.
pub struct HoistedApisOnTopRule;  // VITEST-CON-004
```

- [ ] **Step 3: Commit**

```bash
git add src/rules/consistency.rs src/models.rs src/parser.rs
git commit -m "feat: add consistency rules (3 rules)"
```

---

### Task 18: Register consistency rules and test

**Files:**
- Modify: `src/rules/mod.rs`
- Modify: `tests/integration_tests.rs`

- [ ] **Step 1: Register rules**

```rust
pub mod consistency;

// In all_rules(): add all 3 consistency rules
```

- [ ] **Step 2: Add integration tests**

```rust
#[test]
fn con001_mixed_test_it() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "con001.test.ts", r#"
import { describe, test, it, expect } from 'vitest';
describe('suite', () => {
    test('uses test', () => { expect(true).toBe(true); });
    it('uses it', () => { expect(true).toBe(true); });
});
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-CON-001").is_some());
}

#[test]
fn con001_consistent_test_only_passes() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(&dir, "con001_good.test.ts", r#"
import { describe, test, expect } from 'vitest';
describe('suite', () => {
    test('one', () => { expect(true).toBe(true); });
    test('two', () => { expect(true).toBe(true); });
});
"#);
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    assert!(find_violation(&violations, "VITEST-CON-001").is_none());
}
```

- [ ] **Step 3: Run all tests and verify**

```bash
cargo test
cargo fmt
cargo clippy
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: register and test E15 consistency rules (3 rules)"
```

---

## Final Task: Update README and verify

### Task 19: Update README.md

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Update rule count**

Change "18 rules" to "52 rules" in README header.

- [ ] **Step 2: Add new rule tables**

Add tables for each new category:
- Validation (VITEST-VAL-*)
- No Rules (VITEST-NO-*)
- Prefer Rules (VITEST-PREF-*)
- Require Rules (VITEST-REQ-*)
- Consistency (VITEST-CON-*)

- [ ] **Step 3: Update examples**

Add examples showing new rules in action.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: update README for 52 rules"
```

---

### Task 20: Final verification

- [ ] **Step 1: Run full test suite**

```bash
cargo test
```

- [ ] **Step 2: Run coverage**

```bash
cargo +nightly llvm-cov --fail-under-lines 90
```

- [ ] **Step 3: Run clippy**

```bash
cargo +nightly clippy -- -W clippy::all -W clippy::pedantic
```

- [ ] **Step 4: Format**

```bash
cargo fmt
```

- [ ] **Step 5: Push**

```bash
git push
```

---

## Summary

| Epic | Rules | Total After |
|------|-------|-------------|
| E11: Valid | 5 | 23 |
| E12: No | 12 | 35 |
| E13: Prefer | 11 | 46 |
| E14: Require | 3 | 49 |
| E15: Consistency | 3 | **52** |
