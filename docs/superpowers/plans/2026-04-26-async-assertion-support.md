# Async Assertion Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Detect unawaited async assertions in Vitest tests to prevent silent failures.

**Architecture:** Update `TsParser` to track unawaited async assertions in `TestBlock` and add a new maintenance rule `MissingAwaitAssertionRule`.

**Tech Stack:** Rust, Tree-sitter, Vitest.

---

### Task 1: Update Models

**Files:**
- Modify: `src/models.rs`

- [ ] **Step 1: Add field to TestBlock**

Modify `TestBlock` struct in `src/models.rs`:

```rust
pub struct TestBlock {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub has_assertions: bool,
    pub assertion_count: usize,
    pub has_conditional_logic: bool,
    pub has_try_catch: bool,
    pub uses_settimeout: bool,
    pub uses_datemock: bool,
    pub has_multiple_expects: bool,
    pub is_skipped: bool,
    pub is_nested: bool,
    pub has_return_statement: bool,
    pub unawaited_async_assertions: usize, // New field
}
```

- [ ] **Step 2: Commit**
```bash
git add src/models.rs
git commit -m "models: add unawaited_async_assertions to TestBlock"
```

### Task 2: Update Parser

**Files:**
- Modify: `src/parser.rs`

- [ ] **Step 1: Add field to Analysis struct**

```rust
struct Analysis {
    assertion_count: usize,
    has_conditional: bool,
    has_try_catch: bool,
    uses_settimeout: bool,
    uses_datemock: bool,
    has_return: bool,
    unawaited_async_assertions: usize, // New field
}
```

- [ ] **Step 2: Implement is_awaited helper**

Add this method to `impl TsParser`:

```rust
    fn is_awaited(node: Node) -> bool {
        let mut curr = node;
        while let Some(parent) = curr.parent() {
            if parent.kind() == "await_expression" {
                return true;
            }
            if parent.kind() == "expression_statement"
                || parent.kind() == "lexical_declaration"
                || parent.kind() == "variable_declaration"
                || parent.kind() == "statement_block"
            {
                break;
            }
            curr = parent;
        }
        false
    }
```

- [ ] **Step 3: Update walk_body to detect unawaited async assertions**

Modify `walk_body` in `src/parser.rs`:

```rust
            "call_expression" => {
                let func = node.child_by_field_name("function").unwrap();
                let text = func.utf8_text(source.as_bytes()).unwrap_or("");
                if text.starts_with("expect") {
                    st.assertion_count += 1;
                    if (text.contains(".resolves") || text.contains(".rejects")) && !Self::is_awaited(node) {
                        st.unawaited_async_assertions += 1;
                    }
                }
                // ... rest of the logic
```

- [ ] **Step 4: Update extract_test to populate the new field**

```rust
        Some(TestBlock {
            // ...
            has_return_statement: st.has_return,
            unawaited_async_assertions: st.unawaited_async_assertions,
        })
```

- [ ] **Step 5: Commit**
```bash
git add src/parser.rs
git commit -m "parser: detect unawaited async assertions"
```

### Task 3: Implement Rule

**Files:**
- Modify: `src/rules/maintenance.rs`

- [ ] **Step 1: Implement MissingAwaitAssertionRule**

Add this to `src/rules/maintenance.rs`:

```rust
pub struct MissingAwaitAssertionRule;

impl Rule for MissingAwaitAssertionRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-006"
    }
    fn name(&self) -> &'static str {
        "MissingAwaitAssertionRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
                    "Test has {} unawaited async assertions \u{2014} these will fail silently",
                    tb.unawaited_async_assertions
                ),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Add await before expect() for .resolves or .rejects assertions".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}
```

- [ ] **Step 2: Commit**
```bash
git add src/rules/maintenance.rs
git commit -m "rules: add MissingAwaitAssertionRule"
```

### Task 4: Register Rule

**Files:**
- Modify: `src/rules/mod.rs`

- [ ] **Step 1: Add rule to all_rules()**

```rust
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // ...
        Box::new(maintenance::ReturnInTestRule),
        Box::new(maintenance::MissingAwaitAssertionRule), // Add this
    ]
}
```

- [ ] **Step 2: Update tests in src/rules/mod.rs**

Update `all_rules_count` and `all_rule_ids_present` to include the new rule.

- [ ] **Step 3: Commit**
```bash
git add src/rules/mod.rs
git commit -m "rules: register MissingAwaitAssertionRule"
```

### Task 5: Verification

**Files:**
- Modify: `tests/integration_tests.rs`

- [ ] **Step 1: Add integration test case**

```rust
#[test]
fn mnt006_missing_await_assertion() {
    let dir = TempDir::new().unwrap();
    let path = write_fixture(
        &dir,
        "async_fail.test.ts",
        r#"
import { test, expect } from 'vitest';

test('missing await', async () => {
    expect(Promise.resolve(1)).resolves.toBe(1);
});
"#,
    );
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path]).unwrap();
    let v = find_violation(&violations, "VITEST-MNT-006");
    assert!(v.is_some(), "Expected VITEST-MNT-006 violation");
}
```

- [ ] **Step 2: Run cargo test**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 3: Commit**
```bash
git add tests/integration_tests.rs
git commit -m "test: add integration test for unawaited async assertions"
```
