use crate::models::{Category, ParsedModule, Severity, Violation};
use crate::rules::Rule;

pub struct TimeoutRule;

impl Rule for TimeoutRule {
    fn id(&self) -> &'static str {
        "VITEST-FLK-001"
    }
    fn name(&self) -> &'static str {
        "TimeoutRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Flakiness
    }
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.uses_settimeout)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Test uses setTimeout which can cause timing-dependent flakiness"
                    .to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use vi.useFakeTimers() or vi.advanceTimersByTime() for deterministic time control".to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

pub struct DateMockRule;

impl Rule for DateMockRule {
    fn id(&self) -> &'static str {
        "VITEST-FLK-002"
    }
    fn name(&self) -> &'static str {
        "DateMockRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Flakiness
    }
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
        if module.has_fake_timers {
            return vec![];
        }
        module
            .test_blocks
            .iter()
            .filter(|tb| tb.uses_datemock)
            .map(|tb| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: "Test uses Date without mocking, results may vary across runs".to_string(),
                file_path: tb.file_path.clone(),
                line: tb.line,
                col: None,
                suggestion: Some(
                    "Use vi.useFakeTimers() to freeze time and ensure consistent test results"
                        .to_string(),
                ),
                test_name: Some(tb.name.clone()),
            })
            .collect()
    }
}

pub struct NetworkImportRule;

const NETWORK_LIBS: &[&str] = &[
    "axios",
    "node-fetch",
    "got",
    "undici",
    "http",
    "https",
    "fetch",
];

impl Rule for NetworkImportRule {
    fn id(&self) -> &'static str {
        "VITEST-FLK-003"
    }
    fn name(&self) -> &'static str {
        "NetworkImportRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Flakiness
    }
    fn check(&self, module: &ParsedModule, _all_modules: &[ParsedModule]) -> Vec<Violation> {
        let mut found = false;
        for imp in &module.imports {
            for lib in NETWORK_LIBS {
                if imp.contains(lib) {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }
        if !found {
            return vec![];
        }
        vec![Violation {
            rule_id: self.id().to_string(),
            rule_name: self.name().to_string(),
            severity: self.severity(),
            category: self.category(),
            message:
                "Test file imports network libraries \u{2014} tests may fail due to network issues"
                    .to_string(),
            file_path: module.file_path.clone(),
            line: 1,
            col: None,
            suggestion: Some("Mock network calls using vi.mock() or msw".to_string()),
            test_name: None,
        }]
    }
}
