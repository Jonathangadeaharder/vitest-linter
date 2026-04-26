# Stub Rules Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 8 of 9 stub rules in pytest-linter that currently return empty `Vec<Violation>`.

**Architecture:** Enrich parser models with new fields computed during tree-sitter parsing, then implement rules against the enriched data. Follows the existing pattern: parser extracts → rule checks → violation produced.

**Tech Stack:** Rust, tree-sitter (0.24), tree-sitter-python (0.23), tempfile (3.x) for tests

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `pytest-linter/src/models.rs` | Modify | Add `AssertionInfo` struct, 5 new fields to `TestFunction` |
| `pytest-linter/src/parser.rs` | Modify | Add 5 parser enhancements (assertion analysis, parametrize values, CWD, pytest.raises, fixture mutation) |
| `pytest-linter/src/rules/fixtures.rs` | Modify | Implement FIX-009, FIX-007, DBC-001 |
| `pytest-linter/src/rules/flakiness.rs` | Modify | Implement FLK-004, XDIST-001 |
| `pytest-linter/src/rules/maintenance.rs` | Modify | Implement MNT-002, MNT-003, PARAM-002 |
| `pytest-linter/tests/integration_tests.rs` | Modify | Add 16+ new integration tests (2 per rule) |
| `pytest-linter/src/rules/maintenance.rs` | Modify | Remove `ParametrizeNoVariationRule` from `all_rules()` |
| `pytest-linter/src/rules/mod.rs` | Modify | Update `all_rules()` to remove PARAM-004 |

---

### Task 1: Add AssertionInfo struct and new TestFunction fields to models.rs

**Files:**
- Modify: `pytest-linter/src/models.rs:57-77`

- [ ] **Step 1: Add AssertionInfo struct before TestFunction**

Add between line 56 and 57 (before `TestFunction`):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionInfo {
    pub is_magic: bool,
    pub is_suboptimal: bool,
    pub has_comparison: bool,
    pub expression_text: String,
    pub line: usize,
}
```

- [ ] **Step 2: Add 5 new fields to TestFunction**

Add these fields after `docstring: Option<String>,` (after line 76), before the closing `}`:

```rust
    pub assertions: Vec<AssertionInfo>,
    pub parametrize_values: Vec<Vec<String>>,
    pub uses_cwd_dependency: bool,
    pub uses_pytest_raises: bool,
    pub mutates_fixture_deps: Vec<String>,
```

- [ ] **Step 3: Run tests to see which break**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo build 2>&1 | head -30`
Expected: Compile errors in parser.rs where `TestFunction` is constructed — fields missing.

- [ ] **Step 4: Fix TestFunction construction in parser.rs**

In `build_test_function()` (around line 163-181), add default values for new fields in the `TestFunction { ... }` struct literal:

```rust
            assertions: vec![],
            parametrize_values: vec![],
            uses_cwd_dependency: false,
            uses_pytest_raises: false,
            mutates_fixture_deps: vec![],
```

- [ ] **Step 5: Verify build and all existing tests pass**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test 2>&1 | tail -10`
Expected: All 195 tests pass.

- [ ] **Step 6: Commit**

```bash
git add pytest-linter/src/models.rs pytest-linter/src/parser.rs
git commit -m "feat: add AssertionInfo struct and new fields to TestFunction for stub rule support"
```

---

### Task 2: Implement assertion analysis in parser.rs

**Files:**
- Modify: `pytest-linter/src/parser.rs:328-344` (count_assertions area)
- Modify: `pytest-linter/src/parser.rs:118-182` (build_test_function)

- [ ] **Step 1: Write failing test for magic assert detection**

Add to `tests/integration_tests.rs`:

```rust
#[test]
fn test_magic_assert_detected_in_assertions() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_magic.py",
        r#"
def test_magic():
    assert True
    assert 1
    assert 42 == 42
"#,
    );
    let module = parse_file(&path);
    assert_eq!(module.test_functions.len(), 1);
    let test = &module.test_functions[0];
    assert_eq!(test.assertions.len(), 3);
    assert!(test.assertions[0].is_magic);
    assert!(test.assertions[1].is_magic);
    assert!(!test.assertions[2].is_magic);
    assert!(test.assertions[2].has_comparison);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_magic_assert_detected_in_assertions 2>&1 | tail -10`
Expected: FAIL (assertions field is empty vec)

- [ ] **Step 3: Implement extract_assertions in parser.rs**

Add new method to `impl PythonParser` after `count_assertions_recursive` (around line 344):

```rust
    fn extract_assertions(body: Option<&tree_sitter::Node>, source: &[u8], func_start_row: usize) -> Vec<crate::models::AssertionInfo> {
        body.map_or(vec![], |b| {
            let mut infos = Vec::new();
            Self::collect_assertion_info(*b, source, func_start_row, &mut infos);
            infos
        })
    }

    fn collect_assertion_info(
        node: tree_sitter::Node,
        source: &[u8],
        func_start_row: usize,
        infos: &mut Vec<crate::models::AssertionInfo>,
    ) {
        if node.kind() == "assert_statement" {
            let line = node.start_position().row + 1;
            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();
            let expr_node = children.into_iter().find(|c| {
                !c.kind().starts_with(',') && c.kind() != "comment"
            });
            let expression_text = expr_node
                .map(|n| Self::node_text(n, source))
                .unwrap_or_default();
            let has_comparison = expr_node.is_some_and(|n| {
                Self::has_node_kind_recursive(n, "comparison_operator")
            });
            let is_magic = expr_node.is_some_and(|n| {
                let kind = n.kind();
                if kind == "true" || kind == "false" {
                    return true;
                }
                if kind == "integer" {
                    let text = Self::node_text(n, source);
                    return text == "0" || text == "1";
                }
                !has_comparison && kind == "identifier"
            });
            let is_suboptimal = expr_node.is_some_and(|n| {
                Self::is_suboptimal_assertion(n, source)
            });
            infos.push(crate::models::AssertionInfo {
                is_magic,
                is_suboptimal,
                has_comparison,
                expression_text,
                line,
            });
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_assertion_info(child, source, func_start_row, infos);
        }
    }

    fn has_node_kind_recursive(node: tree_sitter::Node, kind: &str) -> bool {
        if node.kind() == kind {
            return true;
        }
        let mut cursor = node.walk();
        node.children(&mut cursor).any(|c| Self::has_node_kind_recursive(c, kind))
    }

    fn is_suboptimal_assertion(expr: tree_sitter::Node, source: &[u8]) -> bool {
        if expr.kind() == "comparison_operator" {
            let mut cursor = expr.walk();
            for child in expr.children(&mut cursor) {
                if child.kind() == "call" {
                    let func = child.child_by_field_name("function");
                    if let Some(f) = func {
                        let name = Self::node_text(f, source);
                        if name == "len" || name == "type" {
                            return true;
                        }
                    }
                }
                if child.kind() == "not" {
                    let mut nc = child.walk();
                    for inner in child.children(&mut nc) {
                        if inner.kind() == "none" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
```

Then update `build_test_function` to populate assertions. After line that computes `docstring` (~line 161), add:

```rust
        let assertions = Self::extract_assertions(body.as_ref(), source, line);
```

And update the `TestFunction { ... }` struct literal to replace `assertions: vec![]` with:

```rust
            assertions,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_magic_assert_detected_in_assertions 2>&1 | tail -5`
Expected: PASS

- [ ] **Step 5: Write test for suboptimal assert detection**

```rust
#[test]
fn test_suboptimal_assert_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_suboptimal.py",
        r#"
def test_suboptimal():
    assert len(items) == 3
    assert type(x) == int
    assert result is not None
    assert value == expected
"#,
    );
    let module = parse_file(&path);
    let test = &module.test_functions[0];
    assert_eq!(test.assertions.len(), 4);
    assert!(test.assertions[0].is_suboptimal, "assert len(x)==N should be suboptimal");
    assert!(test.assertions[1].is_suboptimal, "assert type(x)==Y should be suboptimal");
    assert!(test.assertions[2].is_suboptimal, "assert x is not None should be suboptimal");
    assert!(!test.assertions[3].is_suboptimal, "normal comparison should not be suboptimal");
}
```

- [ ] **Step 6: Run suboptimal test**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_suboptimal_assert_detected 2>&1 | tail -5`
Expected: PASS

- [ ] **Step 7: Run all tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test 2>&1 | tail -5`
Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
git add pytest-linter/src/parser.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement assertion analysis (magic + suboptimal detection) in parser"
```

---

### Task 3: Implement CWD, pytest.raises, and fixture mutation detection in parser.rs

**Files:**
- Modify: `pytest-linter/src/parser.rs:118-182` (build_test_function)

- [ ] **Step 1: Write failing tests for new detection features**

Add to `tests/integration_tests.rs`:

```rust
#[test]
fn test_cwd_dependency_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_cwd.py",
        r#"
import os
from pathlib import Path

def test_cwd_usage():
    cwd = os.getcwd()
    os.chdir("/tmp")
    assert cwd
"#,
    );
    let module = parse_file(&path);
    assert!(module.test_functions[0].uses_cwd_dependency);
}

#[test]
fn test_no_cwd_dependency() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_no_cwd.py",
        r#"
def test_normal():
    assert 1 == 1
"#,
    );
    let module = parse_file(&path);
    assert!(!module.test_functions[0].uses_cwd_dependency);
}

#[test]
fn test_pytest_raises_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_raises.py",
        r#"
import pytest

def test_error():
    with pytest.raises(ValueError):
        raise ValueError("bad")
"#,
    );
    let module = parse_file(&path);
    assert!(module.test_functions[0].uses_pytest_raises);
}

#[test]
fn test_fixture_mutation_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_mutation.py",
        r#"
def test_mutates(my_list):
    my_list.append(1)
    my_list.extend([2, 3])
    assert len(my_list) == 3
"#,
    );
    let module = parse_file(&path);
    assert!(module.test_functions[0].mutates_fixture_deps.contains(&"my_list".to_string()));
}

#[test]
fn test_no_fixture_mutation_on_read_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_readonly.py",
        r#"
def test_read_only(my_list):
    x = my_list[0]
    assert x
"#,
    );
    let module = parse_file(&path);
    assert!(module.test_functions[0].mutates_fixture_deps.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_cwd_dependency_detected test_pytest_raises_detected test_fixture_mutation_detected 2>&1 | tail -10`
Expected: FAIL (fields are default values)

- [ ] **Step 3: Add detection methods to PythonParser**

Add these methods to `impl PythonParser` in parser.rs (after `extract_fixture_deps` around line 404):

```rust
    fn detect_cwd_dependency(body: Option<&tree_sitter::Node>, source: &[u8]) -> bool {
        body.is_some_and(|b| Self::has_cwd_call(*b, source))
    }

    fn has_cwd_call(node: tree_sitter::Node, source: &[u8]) -> bool {
        if node.kind() == "call" {
            let func = node.child_by_field_name("function");
            if let Some(f) = func {
                let text = Self::node_text(f, source);
                if text == "os.getcwd" || text == "os.chdir" || text == "Path.cwd" {
                    return true;
                }
                if text.contains("getcwd") || text.contains("chdir") {
                    return true;
                }
            }
        }
        if node.kind() == "call" {
            let func = node.child_by_field_name("function");
            if let Some(f) = func {
                if f.kind() == "attribute" {
                    let attr = f.child_by_field_name("attribute");
                    if let Some(a) = attr {
                        let name = Self::node_text(a, source);
                        if name == "getcwd" || name == "chdir" {
                            return true;
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        node.children(&mut cursor).any(|c| Self::has_cwd_call(c, source))
    }

    fn detect_pytest_raises(body: Option<&tree_sitter::Node>, source: &[u8]) -> bool {
        body.is_some_and(|b| Self::has_pytest_raises(*b, source))
    }

    fn has_pytest_raises(node: tree_sitter::Node, source: &[u8]) -> bool {
        if node.kind() == "call" {
            let func = node.child_by_field_name("function");
            if let Some(f) = func {
                if f.kind() == "attribute" {
                    let text = Self::node_text(f, source);
                    if text == "pytest.raises" {
                        return true;
                    }
                    let attr = f.child_by_field_name("attribute");
                    if let Some(a) = attr {
                        let name = Self::node_text(a, source);
                        let obj = f.child_by_field_name("object");
                        let obj_name = obj.map(|o| Self::node_text(o, source));
                        if name == "raises" && obj_name.as_deref() == Some("pytest") {
                            return true;
                        }
                    }
                }
            }
        }
        let mut cursor = node.walk();
        node.children(&mut cursor).any(|c| Self::has_pytest_raises(c, source))
    }

    fn detect_fixture_mutations(
        body: Option<&tree_sitter::Node>,
        source: &[u8],
        fixture_deps: &[String],
    ) -> Vec<String> {
        let mut mutated = Vec::new();
        if let Some(b) = body {
            Self::find_mutations(*b, source, fixture_deps, &mut mutated);
        }
        mutated.sort();
        mutated.dedup();
        mutated
    }

    fn find_mutations(
        node: tree_sitter::Node,
        source: &[u8],
        fixture_deps: &[String],
        mutated: &mut Vec<String>,
    ) {
        if node.kind() == "call" {
            let func = node.child_by_field_name("function");
            if let Some(f) = func {
                if f.kind() == "attribute" {
                    let obj = f.child_by_field_name("object");
                    let attr = f.child_by_field_name("attribute");
                    if let (Some(obj), Some(attr)) = (obj, attr) {
                        let obj_name = Self::node_text(obj, source);
                        let method = Self::node_text(attr, source);
                        let mutating_methods = [
                            "append", "extend", "remove", "pop", "clear",
                            "update", "insert", "add", "discard",
                        ];
                        if mutating_methods.contains(&method.as_str()) {
                            if fixture_deps.contains(&obj_name) {
                                mutated.push(obj_name);
                            }
                        }
                    }
                }
            }
        }
        if node.kind() == "assignment" {
            let target = node.child_by_field_name("left");
            if let Some(t) = target {
                Self::check_assignment_target(t, source, fixture_deps, mutated);
            }
        }
        if node.kind() == "delete_statement" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                let text = Self::node_text(child, source);
                if fixture_deps.contains(&text.trim().to_string()) {
                    mutated.push(text.trim().to_string());
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::find_mutations(child, source, fixture_deps, mutated);
        }
    }

    fn check_assignment_target(
        target: tree_sitter::Node,
        source: &[u8],
        fixture_deps: &[String],
        mutated: &mut Vec<String>,
    ) {
        if target.kind() == "subscript" {
            let value = target.child_by_field_name("value");
            if let Some(v) = value {
                let name = Self::node_text(v, source);
                if fixture_deps.contains(&name) {
                    mutated.push(name);
                }
            }
        }
        if target.kind() == "attribute" {
            let obj = target.child_by_field_name("object");
            if let Some(o) = obj {
                let name = Self::node_text(o, source);
                if fixture_deps.contains(&name) {
                    mutated.push(name);
                }
            }
        }
    }
```

- [ ] **Step 4: Wire detections into build_test_function**

In `build_test_function`, after computing `docstring` and before the `TestFunction { ... }` literal, add:

```rust
        let uses_cwd_dependency = Self::detect_cwd_dependency(body.as_ref(), source);
        let uses_pytest_raises = Self::detect_pytest_raises(body.as_ref(), source);
        let mutates_fixture_deps = Self::detect_fixture_mutations(body.as_ref(), source, &fixture_deps);
```

Update the `TestFunction { ... }` struct literal to replace the default values:

```rust
            uses_cwd_dependency,
            uses_pytest_raises,
            mutates_fixture_deps,
```

- [ ] **Step 5: Run all 5 new tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_cwd_dependency_detected test_no_cwd_dependency test_pytest_raises_detected test_fixture_mutation_detected test_no_fixture_mutation 2>&1 | tail -10`
Expected: All 5 PASS

- [ ] **Step 6: Run full test suite**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test 2>&1 | tail -5`
Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add pytest-linter/src/parser.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: add CWD, pytest.raises, and fixture mutation detection to parser"
```

---

### Task 4: Implement parametrize value extraction in parser.rs

**Files:**
- Modify: `pytest-linter/src/parser.rs:220-252` (count_parametrize_args_ast area)
- Modify: `pytest-linter/src/parser.rs:118-182` (build_test_function)

- [ ] **Step 1: Write failing test for parametrize value extraction**

```rust
#[test]
fn test_parametrize_values_extracted() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_param_values.py",
        r#"
import pytest

@pytest.mark.parametrize("x", [1, 2, 2, 3])
def test_dup(x):
    assert x > 0
"#,
    );
    let module = parse_file(&path);
    let test = &module.test_functions[0];
    assert_eq!(test.parametrize_values.len(), 1);
    assert_eq!(test.parametrize_values[0], vec!["1", "2", "2", "3"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_parametrize_values_extracted 2>&1 | tail -5`
Expected: FAIL (parametrize_values is empty vec)

- [ ] **Step 3: Add extract_parametrize_values method**

Add to `impl PythonParser` after `count_parametrize_args_ast`:

```rust
    fn extract_parametrize_values(decorators: &[DecoratorInfo], source: &[u8]) -> Vec<Vec<String>> {
        let mut all_values = Vec::new();
        for dec in decorators {
            if !dec.text.contains("parametrize") {
                continue;
            }
            if let Some(node) = dec.node {
                if let Some(values) = Self::extract_values_from_decorator(node, source) {
                    all_values.push(values);
                }
            }
        }
        all_values
    }

    fn extract_values_from_decorator(decorator_node: tree_sitter::Node, source: &[u8]) -> Option<Vec<String>> {
        let mut cursor = decorator_node.walk();
        for child in decorator_node.children(&mut cursor) {
            if child.kind() == "call" {
                let mut call_cursor = child.walk();
                for call_child in child.children(&mut call_cursor) {
                    if call_child.kind() == "argument_list" {
                        let mut args_cursor = call_child.walk();
                        for arg in call_child.children(&mut args_cursor) {
                            if arg.kind() == "list" || arg.kind() == "tuple" {
                                let mut values = Vec::new();
                                let mut elem_cursor = arg.walk();
                                for elem in arg.children(&mut elem_cursor) {
                                    match elem.kind() {
                                        "," | "(" | ")" | "[" | "]" | "comment" => {}
                                        _ if !elem.is_extra() => {
                                            values.push(Self::node_text(elem, source).trim().to_string());
                                        }
                                        _ => {}
                                    }
                                }
                                return Some(values);
                            }
                        }
                    }
                }
            }
        }
        None
    }
```

- [ ] **Step 4: Wire into build_test_function**

In `build_test_function`, after computing decorators (line ~130), add:

```rust
        let parametrize_values = Self::extract_parametrize_values(&decorators, source);
```

Update the `TestFunction { ... }` to replace `parametrize_values: vec![]` with:

```rust
            parametrize_values,
```

- [ ] **Step 5: Run test**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_parametrize_values_extracted 2>&1 | tail -5`
Expected: PASS

- [ ] **Step 6: Run full test suite**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test 2>&1 | tail -5`
Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add pytest-linter/src/parser.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: extract parametrize literal values for duplicate detection"
```

---

### Task 5: Implement FIX-009 (FixtureOverlyBroadScopeRule)

**Files:**
- Modify: `pytest-linter/src/rules/fixtures.rs:283-301`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_overly_broad_scope_triggers_fix009() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "conftest.py",
        r#"
import pytest

@pytest.fixture(scope="session")
def simple_value():
    return 42

@pytest.fixture(scope="module")
def another_simple():
    return "hello"
"#,
    );
    let violations = lint_single_file(&path);
    let v1 = find_violation(&violations, "PYTEST-FIX-009");
    assert!(v1.is_some(), "Expected PYTEST-FIX-009 for session-scoped simple fixture");
}

#[test]
fn test_broad_scope_with_expensive_setup_does_not_trigger_fix009() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "conftest.py",
        r#"
import pytest

@pytest.fixture(scope="session")
def db_connection():
    conn = create_engine(DB_URL)
    yield conn
    conn.close()
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-FIX-009");
    assert!(v.is_none(), "Session-scoped fixture with yield should not trigger FIX-009");
}
```

- [ ] **Step 2: Implement the rule**

Replace the `check()` method body of `FixtureOverlyBroadScopeRule` (fixtures.rs:298-300):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for fixture in &module.fixtures {
            if fixture.scope >= crate::models::FixtureScope::Module
                && !fixture.has_yield
                && !fixture.has_db_commit
                && !fixture.has_db_rollback
                && !fixture.uses_file_io
            {
                violations.push(make_violation(
                    self.id(),
                    self.name(),
                    self.severity(),
                    self.category(),
                    format!(
                        "Fixture '{}' has scope '{}' but no expensive setup — consider using function scope for better isolation",
                        fixture.name, fixture.scope
                    ),
                    module.file_path.clone(),
                    fixture.line,
                    Some("Change fixture scope to 'function'".to_string()),
                    None,
                ));
            }
        }
        violations
    }
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_overly_broad test_broad_scope_with_expensive 2>&1 | tail -5`
Expected: Both PASS

- [ ] **Step 4: Commit**

```bash
git add pytest-linter/src/rules/fixtures.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement FIX-009 FixtureOverlyBroadScopeRule"
```

---

### Task 6: Implement FLK-004 (CwdDependencyRule)

**Files:**
- Modify: `pytest-linter/src/rules/flakiness.rs:132-150`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_cwd_dependency_triggers_flk004() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_cwd_flaky.py",
        r#"
import os

def test_cwd():
    os.getcwd()
    assert True
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-FLK-004");
    assert!(v.is_some(), "Expected PYTEST-FLK-004 violation");
    assert_eq!(v.unwrap().rule_name, "CwdDependencyRule");
}

#[test]
fn test_no_cwd_dependency_does_not_trigger_flk004() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_no_cwd.py",
        r#"
def test_safe():
    assert 1 == 1
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-FLK-004");
    assert!(v.is_none());
}
```

- [ ] **Step 2: Implement the rule**

Replace `check()` body of `CwdDependencyRule` (flakiness.rs:147-149):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for test in &module.test_functions {
            if test.uses_cwd_dependency {
                violations.push(make_violation(
                    self.id(),
                    self.name(),
                    self.severity(),
                    self.category(),
                    format!("Test '{}' depends on the current working directory", test.name),
                    module.file_path.clone(),
                    test.line,
                    Some("Use absolute paths or tmp_path fixture instead".to_string()),
                    Some(test.name.clone()),
                ));
            }
        }
        violations
    }
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_cwd_dependency_triggers test_no_cwd_dependency_does_not 2>&1 | tail -5`
Expected: Both PASS

- [ ] **Step 4: Commit**

```bash
git add pytest-linter/src/rules/flakiness.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement FLK-004 CwdDependencyRule"
```

---

### Task 7: Implement DBC-001 (NoContractHintRule)

**Files:**
- Modify: `pytest-linter/src/rules/fixtures.rs:303-321`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_no_contract_hint_triggers_dbc001() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_happy_path.py",
        r#"
def test_happy_only():
    result = add(1, 2)
    assert result == 3
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-DBC-001");
    assert!(v.is_some(), "Expected PYTEST-DBC-001 for happy-path-only test");
}

#[test]
fn test_with_pytest_raises_does_not_trigger_dbc001() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_with_error.py",
        r#"
import pytest

def test_error():
    with pytest.raises(ValueError):
        validate(-1)
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-DBC-001");
    assert!(v.is_none(), "Test with pytest.raises should not trigger DBC-001");
}

#[test]
fn test_parametrized_does_not_trigger_dbc001() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_param.py",
        r#"
import pytest

@pytest.mark.parametrize("x", [1, 2, 3])
def test_values(x):
    assert x > 0
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-DBC-001");
    assert!(v.is_none(), "Parametrized tests should not trigger DBC-001");
}
```

- [ ] **Step 2: Implement the rule**

Replace `check()` body of `NoContractHintRule` (fixtures.rs:318-320):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for test in &module.test_functions {
            if test.has_assertions
                && !test.uses_pytest_raises
                && !test.has_try_except
                && !test.is_parametrized
            {
                violations.push(make_violation(
                    self.id(),
                    self.name(),
                    self.severity(),
                    self.category(),
                    format!("Test '{}' only tests the happy path — consider adding error/edge case coverage", test.name),
                    module.file_path.clone(),
                    test.line,
                    Some("Add tests for error conditions using pytest.raises".to_string()),
                    Some(test.name.clone()),
                ));
            }
        }
        violations
    }
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_no_contract_hint test_with_pytest_raises_does_not test_parametrized_does_not 2>&1 | tail -5`
Expected: All PASS

- [ ] **Step 4: Commit**

```bash
git add pytest-linter/src/rules/fixtures.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement DBC-001 NoContractHintRule"
```

---

### Task 8: Implement FIX-007 (FixtureMutationRule)

**Files:**
- Modify: `pytest-linter/src/rules/fixtures.rs:222-240`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_fixture_mutation_triggers_fix007() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "conftest.py",
        r#"
import pytest

@pytest.fixture
def items():
    return [1, 2, 3]
"#,
    );
    let test_path = write_temp_file(
        dir.path(),
        "test_mutate.py",
        r#"
def test_mutates(items):
    items.append(4)
    assert len(items) == 4
"#,
    );
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path, test_path]).unwrap();
    let v = find_violation(&violations, "PYTEST-FIX-007");
    assert!(v.is_some(), "Expected PYTEST-FIX-007 violation");
}

#[test]
fn test_no_fixture_mutation_does_not_trigger_fix007() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "conftest.py",
        r#"
import pytest

@pytest.fixture
def items():
    return [1, 2, 3]
"#,
    );
    let test_path = write_temp_file(
        dir.path(),
        "test_safe.py",
        r#"
def test_read_only(items):
    assert len(items) == 3
"#,
    );
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[path, test_path]).unwrap();
    let v = find_violation(&violations, "PYTEST-FIX-007");
    assert!(v.is_none(), "Read-only fixture usage should not trigger FIX-007");
}
```

- [ ] **Step 2: Implement the rule**

Replace `check()` body of `FixtureMutationRule` (fixtures.rs:237-239):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for test in &module.test_functions {
            for dep_name in &test.mutates_fixture_deps {
                let is_mutable_fixture = ctx
                    .fixture_map
                    .get(dep_name)
                    .is_some_and(|fixtures| fixtures.iter().any(|f| f.returns_mutable));
                if is_mutable_fixture {
                    violations.push(make_violation(
                        self.id(),
                        self.name(),
                        self.severity(),
                        self.category(),
                        format!("Test '{}' mutates fixture '{}' which may affect other tests", test.name, dep_name),
                        module.file_path.clone(),
                        test.line,
                        Some("Create a fresh copy of the fixture value before modifying it".to_string()),
                        Some(test.name.clone()),
                    ));
                }
            }
        }
        violations
    }
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_fixture_mutation_triggers test_no_fixture_mutation_does_not 2>&1 | tail -5`
Expected: Both PASS

- [ ] **Step 4: Commit**

```bash
git add pytest-linter/src/rules/fixtures.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement FIX-007 FixtureMutationRule"
```

---

### Task 9: Implement MNT-002 (MagicAssertRule) and MNT-003 (SuboptimalAssertRule)

**Files:**
- Modify: `pytest-linter/src/rules/maintenance.rs:41-79`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_magic_assert_triggers_mnt002() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_magic_assert.py",
        r#"
def test_magic():
    assert True
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-MNT-002");
    assert!(v.is_some(), "Expected PYTEST-MNT-002 violation");
    assert!(v.unwrap().message.contains("Magic assertion"));
}

#[test]
fn test_normal_assert_does_not_trigger_mnt002() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_normal.py",
        r#"
def test_normal():
    assert add(1, 2) == 3
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-MNT-002");
    assert!(v.is_none());
}

#[test]
fn test_suboptimal_assert_triggers_mnt003() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_suboptimal.py",
        r#"
def test_subopt():
    assert len(items) == 3
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-MNT-003");
    assert!(v.is_some(), "Expected PYTEST-MNT-003 violation");
    assert!(v.unwrap().message.contains("Suboptimal"));
}

#[test]
fn test_good_assert_does_not_trigger_mnt003() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_good.py",
        r#"
def test_good():
    assert result == expected
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-MNT-003");
    assert!(v.is_none());
}
```

- [ ] **Step 2: Implement MagicAssertRule**

Replace `check()` body of `MagicAssertRule` (maintenance.rs:56-58):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for test in &module.test_functions {
            for assertion in &test.assertions {
                if assertion.is_magic {
                    violations.push(make_violation(
                        self.id(),
                        self.name(),
                        self.severity(),
                        self.category(),
                        format!(
                            "Magic assertion at line {}: '{}' — this always passes/fails",
                            assertion.line, assertion.expression_text
                        ),
                        module.file_path.clone(),
                        assertion.line,
                        Some("Replace with a meaningful comparison".to_string()),
                        Some(test.name.clone()),
                    ));
                }
            }
        }
        violations
    }
```

- [ ] **Step 3: Implement SuboptimalAssertRule**

Replace `check()` body of `SuboptimalAssertRule` (maintenance.rs:76-78):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for test in &module.test_functions {
            for assertion in &test.assertions {
                if assertion.is_suboptimal {
                    violations.push(make_violation(
                        self.id(),
                        self.name(),
                        self.severity(),
                        self.category(),
                        format!(
                            "Suboptimal assertion at line {}: '{}'",
                            assertion.line, assertion.expression_text
                        ),
                        module.file_path.clone(),
                        assertion.line,
                        Some("Use a more direct assertion pattern".to_string()),
                        Some(test.name.clone()),
                    ));
                }
            }
        }
        violations
    }
```

- [ ] **Step 4: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_magic_assert test_normal_assert_does_not test_suboptimal_assert test_good_assert_does_not 2>&1 | tail -5`
Expected: All PASS

- [ ] **Step 5: Commit**

```bash
git add pytest-linter/src/rules/maintenance.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement MNT-002 MagicAssertRule and MNT-003 SuboptimalAssertRule"
```

---

### Task 10: Implement XDIST-001 (XdistSharedStateRule)

**Files:**
- Modify: `pytest-linter/src/rules/flakiness.rs:199-217`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_xdist_shared_state_triggers_xdist001() {
    let dir = tempfile::tempdir().unwrap();
    let conftest = write_temp_file(
        dir.path(),
        "conftest.py",
        r#"
import pytest

@pytest.fixture(scope="session")
def shared_list():
    return []
"#,
    );
    let test_path = write_temp_file(
        dir.path(),
        "test_shared.py",
        r#"
def test_mutates_shared(shared_list):
    shared_list.append(1)
    assert len(shared_list) == 1
"#,
    );
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[conftest, test_path]).unwrap();
    let v = find_violation(&violations, "PYTEST-XDIST-001");
    assert!(v.is_some(), "Expected PYTEST-XDIST-001 violation");
}

#[test]
fn test_function_scope_mutable_does_not_trigger_xdist001() {
    let dir = tempfile::tempdir().unwrap();
    let conftest = write_temp_file(
        dir.path(),
        "conftest.py",
        r#"
import pytest

@pytest.fixture
def local_list():
    return []
"#,
    );
    let test_path = write_temp_file(
        dir.path(),
        "test_local.py",
        r#"
def test_mutates_local(local_list):
    local_list.append(1)
    assert len(local_list) == 1
"#,
    );
    let engine = LintEngine::new().unwrap();
    let violations = engine.lint_paths(&[conftest, test_path]).unwrap();
    let v = find_violation(&violations, "PYTEST-XDIST-001");
    assert!(v.is_none(), "Function-scoped fixture should not trigger XDIST-001");
}
```

- [ ] **Step 2: Implement the rule**

Replace `check()` body of `XdistSharedStateRule` (flakiness.rs:214-216):

```rust
    fn check(&self, module: &ParsedModule, all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        let session_mutable_fixtures: Vec<&str> = all_modules
            .iter()
            .flat_map(|m| m.fixtures.iter())
            .filter(|f| f.scope == crate::models::FixtureScope::Session && f.returns_mutable)
            .map(|f| f.name.as_str())
            .collect();

        if session_mutable_fixtures.is_empty() {
            return violations;
        }

        for m in all_modules {
            for test in &m.test_functions {
                for dep in &test.mutates_fixture_deps {
                    if session_mutable_fixtures.contains(&dep.as_str()) {
                        violations.push(make_violation(
                            self.id(),
                            self.name(),
                            self.severity(),
                            self.category(),
                            format!(
                                "Session-scoped fixture '{}' returns mutable state that is modified by test '{}' — unsafe for xdist",
                                dep, test.name
                            ),
                            m.file_path.clone(),
                            test.line,
                            Some("Use function scope or return immutable values".to_string()),
                            Some(test.name.clone()),
                        ));
                    }
                }
            }
        }
        violations
    }
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_xdist_shared_state test_function_scope_mutable_does_not 2>&1 | tail -5`
Expected: Both PASS

- [ ] **Step 4: Commit**

```bash
git add pytest-linter/src/rules/flakiness.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement XDIST-001 XdistSharedStateRule"
```

---

### Task 11: Implement PARAM-002 (ParametrizeDuplicateRule)

**Files:**
- Modify: `pytest-linter/src/rules/maintenance.rs:357-375`

- [ ] **Step 1: Write integration tests**

```rust
#[test]
fn test_parametrize_duplicate_triggers_param002() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_dup_param.py",
        r#"
import pytest

@pytest.mark.parametrize("x", [1, 2, 2, 3])
def test_dup(x):
    assert x > 0
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-PARAM-002");
    assert!(v.is_some(), "Expected PYTEST-PARAM-002 violation");
    assert!(v.unwrap().message.contains("duplicate"));
}

#[test]
fn test_parametrize_no_duplicate_does_not_trigger_param002() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_temp_file(
        dir.path(),
        "test_no_dup.py",
        r#"
import pytest

@pytest.mark.parametrize("x", [1, 2, 3])
def test_unique(x):
    assert x > 0
"#,
    );
    let violations = lint_single_file(&path);
    let v = find_violation(&violations, "PYTEST-PARAM-002");
    assert!(v.is_none());
}
```

- [ ] **Step 2: Implement the rule**

Replace `check()` body of `ParametrizeDuplicateRule` (maintenance.rs:372-374):

```rust
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule], _ctx: &RuleContext) -> Vec<Violation> {
        let mut violations = Vec::new();
        for test in &module.test_functions {
            for values in &test.parametrize_values {
                let mut seen = std::collections::HashSet::new();
                let mut duplicates = std::collections::HashSet::new();
                for val in values {
                    if !seen.insert(val) {
                        duplicates.insert(val.as_str());
                    }
                }
                if !duplicates.is_empty() {
                    let dup_str: Vec<&str> = duplicates.into_iter().collect();
                    violations.push(make_violation(
                        self.id(),
                        self.name(),
                        self.severity(),
                        self.category(),
                        format!(
                            "Parametrize in test '{}' has duplicate values: {}",
                            test.name,
                            dup_str.join(", ")
                        ),
                        module.file_path.clone(),
                        test.line,
                        Some("Remove duplicate parametrize values".to_string()),
                        Some(test.name.clone()),
                    ));
                }
            }
        }
        violations
    }
```

- [ ] **Step 3: Run tests**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test test_parametrize_duplicate test_parametrize_no_duplicate 2>&1 | tail -5`
Expected: Both PASS

- [ ] **Step 4: Commit**

```bash
git add pytest-linter/src/rules/maintenance.rs pytest-linter/tests/integration_tests.rs
git commit -m "feat: implement PARAM-002 ParametrizeDuplicateRule"
```

---

### Task 12: Remove PARAM-004 and run final verification

**Files:**
- Modify: `pytest-linter/src/rules/maintenance.rs:418-436`
- Modify: `pytest-linter/src/rules/mod.rs:22-54`

- [ ] **Step 1: Remove ParametrizeNoVariationRule struct**

Delete the `ParametrizeNoVariationRule` struct and impl block from maintenance.rs (lines 418-436).

- [ ] **Step 2: Update all_rules()**

Remove `Box::new(maintenance::ParametrizeNoVariationRule),` from `all_rules()` in mod.rs.

Update `test_all_rules_returns_29` to `test_all_rules_returns_28`.

- [ ] **Step 3: Run full test suite**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo test 2>&1 | tail -10`
Expected: All tests pass, rule count is 28.

- [ ] **Step 4: Run clippy**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo +nightly clippy --lib -- -W clippy::all -W clippy::pedantic -W clippy::nursery 2>&1 | grep "generated"`
Expected: 0 warnings

- [ ] **Step 5: Run coverage**

Run: `cd /Users/jonathangadeaharder/Documents/projects/pytest-linter/pytest-linter && cargo +nightly llvm-cov --branch --doctests 2>&1 | grep "^TOTAL"`
Expected: Branch coverage ≥85%

- [ ] **Step 6: Commit**

```bash
git add pytest-linter/src/rules/maintenance.rs pytest-linter/src/rules/mod.rs
git commit -m "refactor: remove PARAM-004 ParametrizeNoVariationRule (requires semantic analysis)"
```

- [ ] **Step 7: Push all commits**

```bash
git push origin feat/rust-rewrite
```
