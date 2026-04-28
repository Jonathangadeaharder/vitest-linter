use crate::models::{Category, HookKind, ParsedModule, Severity, Violation};
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
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

/// Flags files using `vi.mock()` without `afterEach` cleanup, allowing
/// mocks to leak between tests.
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
    fn check(&self, module: &ParsedModule, _ctx: &crate::rules::LintContext<'_>) -> Vec<Violation> {
        if module.vi_mocks.is_empty() {
            return vec![];
        }

        let has_cleanup = module.hook_calls.iter().any(|h| {
            h.kind == HookKind::AfterEach
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
                    "Add afterEach(() => { vi.restoreAllMocks() }) to clean up mocks between tests"
                        .to_string(),
                ),
                test_name: None,
            }]
        } else {
            vec![]
        }
    }
}
