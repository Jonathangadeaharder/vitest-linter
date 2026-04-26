use crate::models::{Category, ParsedModule, Severity, Violation};
use crate::rules::Rule;

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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
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
