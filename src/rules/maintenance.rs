use crate::models::{Category, HookKind, ModuleGraph, ParsedModule, Severity, Violation};
use crate::rules::Rule;

/// Flags tests that contain no `expect()` assertions — they pass even if
/// the code under test is broken.
pub struct NoAssertionRule;

impl Rule for NoAssertionRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-001"
    }
    fn name(&self) -> &'static str {
        "NoAssertionRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| !tb.has_assertions)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Test has no assertions \u{2014} it will pass even if the code is broken"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Add at least one expect() assertion to verify behavior".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests with more than 5 `expect()` calls, suggesting they should
/// be split into focused, single-behavior tests.
pub struct MultipleExpectRule;

impl Rule for MultipleExpectRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-002"
    }
    fn name(&self) -> &'static str {
        "MultipleExpectRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.assertion_count > 5)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "Test has {} assertions \u{2014} consider splitting into focused tests",
                    tb.assertion_count
                ),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Each test should verify one behavior. Split multiple assertions into separate tests"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests that contain `if`/`switch` statements — tests should be
/// deterministic without conditional branching.
pub struct ConditionalLogicRule;

impl Rule for ConditionalLogicRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-003"
    }
    fn name(&self) -> &'static str {
        "ConditionalLogicRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_conditional_logic)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Test contains conditional logic (if/switch) \u{2014} tests should be deterministic"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Replace conditional logic with separate test cases using test.each() or describe.each()"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests that use `try/catch` — prefer `expect().toThrow()` or
/// `expect().rejects` for error testing.
pub struct TryCatchRule;

impl Rule for TryCatchRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-004"
    }
    fn name(&self) -> &'static str {
        "TryCatchRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_try_catch)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Test uses try/catch \u{2014} use expect(() => fn()).toThrow() instead"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use expect().toThrow() or expect().rejects for error testing".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags skipped tests (`it.skip`, `test.todo`) that provide no value
/// and should either be fixed or removed.
pub struct EmptyTestRule;

impl Rule for EmptyTestRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-005"
    }
    fn name(&self) -> &'static str {
        "EmptyTestRule"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.is_skipped)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Skipped test detected \u{2014} skipped tests provide no value"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Either fix the test or remove it. Avoid leaving skipped tests in the codebase"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests inside deeply nested `describe` blocks (deeper than 3 levels),
/// which harms readability and should be flattened.
pub struct NestedDescribeRule;

impl Rule for NestedDescribeRule {
    fn id(&self) -> &'static str {
        "VITEST-STR-001"
    }
    fn name(&self) -> &'static str {
        "NestedDescribeRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Structure
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.is_nested)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Deeply nested describe blocks \u{2014} consider flattening test structure"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Keep describe nesting to 2 levels. Use descriptive test names instead of deep nesting"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests that use `return` statements — tests should use assertions
/// to verify behavior, not return values.
pub struct ReturnInTestRule;

impl Rule for ReturnInTestRule {
    fn id(&self) -> &'static str {
        "VITEST-STR-002"
    }
    fn name(&self) -> &'static str {
        "ReturnInTestRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Structure
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_return_statement)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Return statement in test \u{2014} tests should use assertions, not return values"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Replace return with explicit expect() assertions".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests with unawaited `.resolves` or `.rejects` assertions that
/// will fail silently without `await`.
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
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
                    "Test has {} unawaited async assertions — these will fail silently",
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

/// Flags focused tests (`it.only`, `test.only`, `describe.only`) that
/// skip all other tests in the file — a common CI failure.
pub struct FocusedTestRule;

impl Rule for FocusedTestRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-007"
    }
    fn name(&self) -> &'static str {
        "FocusedTestRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        let mut out = Vec::new();

        for tb in &module.test_blocks {
            if tb.is_only {
                out.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: format!(
                        "Focused test '{}' uses .only — all other tests in this file will be skipped",
                        tb.name
                    ),
                    file_path: tb.file_path.clone(),
                    line: tb.line,
                    col: None,
                    suggestion: Some(
                        "Remove .only before committing. Focused tests mask failures in CI".to_string(),
                    ),
                    test_name: Some(tb.name.clone()),
                });
            }
        }

        for db in &module.describe_blocks {
            if db.is_only {
                out.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: format!(
                        "Focused describe '{}' uses .only — all other tests in this file will be skipped",
                        db.name
                    ),
                    file_path: db.file_path.clone(),
                    line: db.line,
                    col: None,
                    suggestion: Some(
                        "Remove .only before committing. Focused tests mask failures in CI".to_string(),
                    ),
                    test_name: None,
                });
            }
        }

        out
    }
}

/// Flags files using `vi.mock()` without `afterEach` or `beforeEach` cleanup,
/// allowing mocks to leak between tests.
pub struct MissingMockCleanupRule;

const MOCK_CLEANUP_CALLS: &[&str] = &["vi.restoreAllMocks", "vi.clearAllMocks", "vi.resetAllMocks"];

impl Rule for MissingMockCleanupRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-008"
    }
    fn name(&self) -> &'static str {
        "MissingMockCleanupRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        if module.vi_mocks.is_empty() {
            return vec![];
        }

        let has_cleanup = module.hook_calls.iter().any(|h| {
            (h.kind == HookKind::AfterEach || h.kind == HookKind::BeforeEach)
                && h.vi_calls
                    .iter()
                    .any(|c| MOCK_CLEANUP_CALLS.iter().any(|mc| c == mc))
        });

        if has_cleanup {
            return vec![];
        }

        // Report once per file (on first vi.mock line)
        let first_mock = module.vi_mocks.iter().min_by_key(|m| m.line);
        if let Some(mock) = first_mock {
            vec![Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "vi.mock('{}') used without afterEach cleanup — mocks may leak between tests",
                    mock.source
                ),
                file_path: module.file_path.clone(),
                line: mock.line,
                col: None,
                suggestion: Some(
                    "Add afterEach(() => { vi.restoreAllMocks() }) or beforeEach(() => { vi.clearAllMocks() }) to clean up mocks between tests"
                        .to_string(),
                ),
                test_name: None,
            }]
        } else {
            vec![]
        }
    }
}

/// Flags tests where all assertions are weak — `toBeDefined()`,
/// `toBeTruthy()`, `not.toThrow()`, etc. — that verify existence or
/// truthiness rather than actual values.
pub struct WeakAssertionRule;

impl Rule for WeakAssertionRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-009"
    }
    fn name(&self) -> &'static str {
        "WeakAssertionRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, _graph: &ModuleGraph) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.assertion_count > 0 && tb.weak_assertion_count == tb.assertion_count)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "All {} assertion(s) in this test are weak — they verify existence or truthiness, not actual behavior",
                    tb.assertion_count
                ),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use specific assertions like toBe(), toEqual(), or toHaveLength() instead of toBeDefined()/toBeTruthy()/not.toThrow()"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags test files that are tightly coupled to a single module's implementation —
/// the test imports one production module, test count ≈ export count, and >80% of
/// test names match export names. Such tests break on any refactor and provide
/// low-value coverage.
pub struct ImplementationCoupledRule;

impl Rule for ImplementationCoupledRule {
    fn id(&self) -> &'static str {
        "VITEST-MNT-010"
    }
    fn name(&self) -> &'static str {
        "ImplementationCoupledRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Maintenance
    }
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>, graph: &ModuleGraph) -> Vec<Violation> {
        // Only flag files that import exactly one production module.
        let prod_imports: Vec<&str> = module
            .imports
            .iter()
            .filter(|imp| {
                !imp.starts_with("vitest")
                    && !imp.starts_with("@testing-library")
                    && !imp.starts_with("jest")
                    && !imp.starts_with("@jest")
                    && !imp.contains("node_modules")
                    && !imp.starts_with(".")
                    && !imp.starts_with("/")
            })
            .map(|s| s.as_str())
            .collect();

        if prod_imports.len() != 1 {
            return vec![];
        }

        // Resolve the source module from the graph and get its exports.
        let source_module_path = prod_imports[0];
        let resolved = _ctx.config.resolve_module_path(source_module_path);
        let source_module = match graph.get_module(std::path::Path::new(&resolved)) {
            Some(m) => m,
            None => return vec![],
        };

        let export_count = source_module.exports.len();
        let test_count = module.test_blocks.len();

        if export_count == 0 || test_count == 0 {
            return vec![];
        }

        // Check ratio: test count should be within 0.8–1.2 of export count.
        let ratio = test_count as f64 / export_count as f64;
        if ratio < 0.8 || ratio > 1.2 {
            return vec![];
        }

        // Check if >80% of test names match export names.
        let export_names: Vec<String> = source_module
            .exports
            .iter()
            .map(|e| e.name.to_lowercase())
            .collect();

        let matching = module
            .test_blocks
            .iter()
            .filter(|tb| {
                let test_name_lower = tb.name.to_lowercase();
                export_names.iter().any(|en| test_name_lower.contains(en))
            })
            .count();

        let match_ratio = matching as f64 / test_count as f64;
        if match_ratio < 0.8 {
            return vec![];
        }

        vec![Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            severity: self.severity(),
            category: self.category(),
            message: format!(
                "Test file is tightly coupled to '{}' — {} tests for {} exports, {}% name match",
                source_module_path,
                test_count,
                export_count,
                (match_ratio * 100.0) as u32
            ),
            file_path: module.file_path.clone(),
            line: 1,
            col: None,
            suggestion: Some(
                "Test behavior, not implementation details. Refactor tests to verify public API outcomes rather than mirroring export structure"
                    .to_string(),
            ),
            test_name: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn empty_module() -> ParsedModule {
        ParsedModule {
            file_path: PathBuf::from("test.ts"),
            test_blocks: vec![],
            describe_blocks: vec![],
            imports: vec![],
            imports_parsed: vec![],
            hook_calls: vec![],
            vi_mocks: vec![],
            exports: vec![],
            has_fake_timers: false,
            expects_outside_tests: vec![],
            imports_node_test: false,
            snapshot_sizes: vec![],
        }
    }

    #[test]
    fn implementation_coupled_no_prod_imports() {
        let mut module = empty_module();
        module.imports = vec!["vitest".into(), "@testing-library/react".into()];
        let rule = ImplementationCoupledRule;
        let violations = rule.check(&module, &crate::rules::LintContext::default(), &ModuleGraph::new(&[], &[]));
        assert!(violations.is_empty());
    }

    #[test]
    fn implementation_coupled_multiple_prod_imports() {
        let mut module = empty_module();
        module.imports = vec!["vitest".into(), "lodash".into(), "axios".into()];
        let rule = ImplementationCoupledRule;
        let violations = rule.check(&module, &crate::rules::LintContext::default(), &ModuleGraph::new(&[], &[]));
        assert!(violations.is_empty());
    }
}
