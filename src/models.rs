use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Category {
    Flakiness,
    Maintenance,
    Structure,
    Dependencies,
}

#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub category: Category,
    pub message: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub col: Option<usize>,
    pub suggestion: Option<String>,
    pub test_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Error < Severity::Warning);
        assert!(Severity::Warning < Severity::Info);
        assert!(Severity::Error < Severity::Info);
    }

    #[test]
    fn severity_equality() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Error, Severity::Warning);
    }

    #[test]
    fn category_values() {
        assert_ne!(Category::Flakiness, Category::Maintenance);
        assert_ne!(Category::Maintenance, Category::Structure);
        assert_ne!(Category::Flakiness, Category::Structure);
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
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
    pub is_only: bool,
    pub is_nested: bool,
    pub has_return_statement: bool,
    pub unawaited_async_assertions: usize,
}

#[derive(Debug, Clone)]
pub struct DescribeBlock {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub is_only: bool,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct ParsedModule {
    pub file_path: PathBuf,
    pub imports: Vec<String>,
    pub imports_parsed: Vec<ImportEntry>,
    pub vi_mocks: Vec<ViMockCall>,
    pub hook_calls: Vec<HookCall>,
    pub test_blocks: Vec<TestBlock>,
    pub describe_blocks: Vec<DescribeBlock>,
    pub has_fake_timers: bool,
}

#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub source: String,
    pub named: Vec<String>,
    pub default: Option<String>,
    pub namespace: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockScope {
    Module,
    Hook,
    Test,
}

#[derive(Debug, Clone)]
pub struct ViMockCall {
    pub source: String,
    pub line: usize,
    pub scope: MockScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookKind {
    BeforeEach,
    AfterEach,
    BeforeAll,
    AfterAll,
}

#[derive(Debug, Clone)]
pub struct HookCall {
    pub kind: HookKind,
    pub line: usize,
    pub vi_calls: Vec<String>,
}
