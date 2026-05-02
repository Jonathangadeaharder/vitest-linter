use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Severity level for a lint violation, ordered Error > Warning > Info.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Category grouping for lint rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Category {
    Flakiness,
    Maintenance,
    Structure,
    Dependencies,
    Validation,
}

/// A single lint violation found by a rule.
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
    pub uses_fake_timers: bool,
    pub uses_random: bool,
    pub has_expect_call_without_assertion: bool,
    pub has_return_of_expect: bool,
    pub title_is_template_literal: bool,
    pub has_async_expect_wrapper: bool,
    pub uses_fit_or_xit: bool,
    pub has_done_callback: bool,
    pub has_conditional_expect: bool,
    pub weak_assertion_count: usize,
    pub has_real_timers_call: bool,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct DescribeBlock {
    pub name: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub is_only: bool,
    pub depth: usize,
    pub title_is_template_literal: bool,
    pub title_is_empty: bool,
    pub is_async: bool,
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
    pub expects_outside_tests: Vec<ExpectOutsideTest>,
    pub imports_node_test: bool,
    pub snapshot_sizes: Vec<SnapshotSize>,
    pub exports: Vec<ExportEntry>,
}

#[derive(Debug, Clone)]
pub struct ExpectOutsideTest {
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct SnapshotSize {
    pub line: usize,
    pub size: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportKind {
    Named,
    Default,
    Namespace,
}

#[derive(Debug, Clone)]
pub struct ExportEntry {
    pub name: String,
    pub kind: ExportKind,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct ModuleGraph {
    pub modules: HashMap<PathBuf, ParsedModule>,
    pub edges: HashMap<PathBuf, Vec<PathBuf>>,
}

impl ModuleGraph {
    /// Build a module graph from test modules and source modules.
    #[must_use]
    pub fn new(test_modules: &[ParsedModule], source_modules: &[ParsedModule]) -> Self {
        let mut modules = HashMap::new();
        let mut edges = HashMap::new();

        // Add all modules to the graph
        for module in test_modules.iter().chain(source_modules.iter()) {
            modules.insert(module.file_path.clone(), module.clone());
            edges.insert(module.file_path.clone(), Vec::new());
        }

        // Build edges from imports
        for module in test_modules {
            for imp in &module.imports_parsed {
                // Try to resolve relative imports
                if imp.source.starts_with('.') || imp.source.starts_with('/') {
                    if let Some(parent) = module.file_path.parent() {
                        let base = parent.join(&imp.source);
                        let exts = [".ts", ".tsx", ".js", ".jsx"];
                        for ext in &exts {
                            let candidate = base.with_extension(ext.strip_prefix('.').unwrap());
                            if modules.contains_key(&candidate) {
                                edges
                                    .entry(module.file_path.clone())
                                    .or_default()
                                    .push(candidate);
                                break;
                            }
                        }
                    }
                }
            }
        }

        Self { modules, edges }
    }

    /// Get a module by its file path.
    #[must_use]
    pub fn get_module(&self, path: &Path) -> Option<&ParsedModule> {
        self.modules.get(path)
    }

    /// Get the dependencies of a module.
    #[must_use]
    pub fn get_dependencies(&self, path: &Path) -> Vec<&ParsedModule> {
        self.edges
            .get(path)
            .map(|deps| {
                deps.iter()
                    .filter_map(|dep| self.modules.get(dep))
                    .collect()
            })
            .unwrap_or_default()
    }
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
    pub factory_keys: Vec<String>,
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
