use crate::models::{Category, ModuleGraph, ParsedModule, Severity, Violation};
use crate::rules::Rule;

/// Flags tests that call `expect()` without a chained assertion method
/// (e.g. `expect(value)` with no `.toBe()`, `.toEqual()`, etc.).
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
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_expect_call_without_assertion)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message:
                    "expect() called without an assertion method \u{2014} the assertion will always pass"
                        .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Add an assertion method like .toBe(), .toEqual(), or .toBeTruthy() after expect()"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

/// Flags tests that use `.resolves` or `.rejects` without `await`, causing
/// the assertion to fail silently.
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
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_return_of_expect)
            .map(|tb| {
                Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message:
                    "expect() returned instead of awaited \u{2014} the assertion may fail silently"
                        .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use await before expect() for async assertions, or remove the return statement"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            }
            })
            .collect()
    }
}

/// Flags `describe` blocks with `async` callbacks. Vitest `describe` callbacks
/// must be synchronous — async callbacks are silently ignored.
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
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        module
            .describe_blocks
            .iter()
            .filter(|db| db.is_async)
            .map(|db| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "describe('{}') has an async callback \u{2014} describe callbacks must be synchronous",
                    db.name
                ),
                file_path: db.file_path.clone(),
                line: db.line,
                col: None,
                suggestion: Some(
                    "Remove the async keyword from the describe callback. Use async in individual test callbacks instead"
                        .to_string(),
                ),
                test_name: None,
            })
            .collect()
    }
}

/// Flags test and describe blocks with empty or template-literal titles,
/// which produce unclear test output.
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
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        let mut out = Vec::new();

        for tb in &module.test_blocks {
            if tb.title_is_template_literal {
                out.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: format!(
                        "Test '{}' uses a template literal title \u{2014} prefer static string titles for stable test output",
                        tb.name
                    ),
                    file_path: tb.file_path.clone(),
                    line: tb.line,
                    col: None,
                    suggestion: Some(
                        "Use a plain string literal for the test title instead of a template literal"
                            .to_string(),
                    ),
                    test_name: Some(tb.name.clone()),
                });
            }
        }

        for db in &module.describe_blocks {
            if db.title_is_empty {
                out.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: "describe() has an empty title \u{2014} provide a meaningful name"
                        .to_string(),
                    file_path: db.file_path.clone(),
                    line: db.line,
                    col: None,
                    suggestion: Some("Add a descriptive name to the describe() block".to_string()),
                    test_name: None,
                });
            }
            if db.title_is_template_literal {
                out.push(Violation {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.severity(),
                    category: self.category(),
                    message: format!(
                        "describe('{}') uses a template literal title \u{2014} prefer static string titles",
                        db.name
                    ),
                    file_path: db.file_path.clone(),
                    line: db.line,
                    col: None,
                    suggestion: Some(
                        "Use a plain string literal for the describe title instead of a template literal"
                            .to_string(),
                    ),
                    test_name: None,
                });
            }
        }

        out
    }
}

/// Flags tests that wrap `expect()` in an async function unnecessarily.
/// E.g. `expect(async () => { ... })` when no async assertion is needed.
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
    fn check(
        &self,
        module: &ParsedModule,
        _ctx: &crate::rules::LintContext<'_>,
        _graph: &ModuleGraph,
    ) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.has_async_expect_wrapper)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message:
                    "expect() wraps an async function unnecessarily \u{2014} use expect(...).resolves or expect(...).rejects instead"
                        .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use expect(await asyncFn()) or expect(asyncFn()).resolves instead of expect(async () => ...)"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::parser::TsParser;
    use std::path::PathBuf;

    fn parse(content: &str, name: &str) -> ParsedModule {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        let parser = TsParser::new().unwrap();
        let mut module = parser.parse_file(&path).unwrap();
        module.file_path = PathBuf::from(name);
        module
    }

    fn default_ctx() -> crate::rules::LintContext<'static> {
        static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
        let config = CONFIG.get_or_init(Config::default);
        crate::rules::LintContext {
            config,
            all_modules: &[],
        }
    }

    #[test]
    fn val_001_flags_expect_without_assertion() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test('bare expect', () => {
    expect(true);
});
"#,
            "bare_expect.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidExpectRule.check(&module, &ctx, &ModuleGraph::default());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-VAL-001");
    }

    #[test]
    fn val_001_no_violation_with_assertion() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test('proper expect', () => {
    expect(1).toBe(1);
});
"#,
            "proper_expect.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidExpectRule.check(&module, &ctx, &ModuleGraph::default());
        assert!(v.is_empty());
    }

    #[test]
    fn val_002_flags_return_of_expect() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test('return expect', () => {
    return expect(Promise.resolve(1)).resolves.toBe(1);
});
"#,
            "return_expect.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidExpectInPromiseRule.check(&module, &ctx, &ModuleGraph::default());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-VAL-002");
    }

    #[test]
    fn val_002_no_violation_without_return() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test('no return', async () => {
    await expect(Promise.resolve(1)).resolves.toBe(1);
});
"#,
            "no_return.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidExpectInPromiseRule.check(&module, &ctx, &ModuleGraph::default());
        assert!(v.is_empty());
    }

    #[test]
    fn val_003_flags_async_describe() {
        let module = parse(
            r#"
import { describe, test, expect } from 'vitest';

describe('async describe', async () => {
    test('inside', () => {
        expect(1).toBe(1);
    });
});
"#,
            "async_describe.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidDescribeCallbackRule.check(&module, &ctx, &ModuleGraph::default());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-VAL-003");
    }

    #[test]
    fn val_003_no_violation_sync_describe() {
        let module = parse(
            r#"
import { describe, test, expect } from 'vitest';

describe('sync describe', () => {
    test('inside', () => {
        expect(1).toBe(1);
    });
});
"#,
            "sync_describe.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidDescribeCallbackRule.check(&module, &ctx, &ModuleGraph::default());
        assert!(v.is_empty());
    }

    #[test]
    fn val_004_flags_template_literal_test_title() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test(`template title`, () => {
    expect(1).toBe(1);
});
"#,
            "template_title.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidTitleRule.check(&module, &ctx, &ModuleGraph::default());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-VAL-004");
    }

    #[test]
    fn val_004_flags_empty_describe_title() {
        let module = parse(
            r#"
import { describe, test, expect } from 'vitest';

describe('', () => {
    test('inside', () => {
        expect(1).toBe(1);
    });
});
"#,
            "empty_describe.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidTitleRule.check(&module, &ctx, &ModuleGraph::default());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-VAL-004");
    }

    #[test]
    fn val_004_no_violation_with_string_titles() {
        let module = parse(
            r#"
import { describe, test, expect } from 'vitest';

describe('proper title', () => {
    test('proper test', () => {
        expect(1).toBe(1);
    });
});
"#,
            "proper_titles.test.ts",
        );
        let ctx = default_ctx();
        let v = ValidTitleRule.check(&module, &ctx, &ModuleGraph::default());
        assert!(v.is_empty());
    }

    #[test]
    fn val_005_flags_async_expect_wrapper() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test('async wrapper', () => {
    expect(async () => {
        await Promise.resolve(1);
    }).not.toThrow();
});
"#,
            "async_wrapper.test.ts",
        );
        let ctx = default_ctx();
        let v = NoUnneededAsyncExpectFunctionRule.check(&module, &ctx, &ModuleGraph::default());
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-VAL-005");
    }

    #[test]
    fn val_005_no_violation_without_async_wrapper() {
        let module = parse(
            r#"
import { test, expect } from 'vitest';

test('sync expect', () => {
    expect(1).toBe(1);
});
"#,
            "sync_expect.test.ts",
        );
        let ctx = default_ctx();
        let v = NoUnneededAsyncExpectFunctionRule.check(&module, &ctx, &ModuleGraph::default());
        assert!(v.is_empty());
    }
}
