use crate::config::matches_path;
use crate::models::{Category, ParsedModule, Severity, Violation};
use crate::rules::{LintContext, Rule};

pub struct BannedModuleMockRule;

impl Rule for BannedModuleMockRule {
    fn id(&self) -> &'static str {
        "VITEST-DEP-001"
    }
    fn name(&self) -> &'static str {
        "BannedModuleMockRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Dependencies
    }
    fn check(&self, module: &ParsedModule, ctx: &LintContext<'_>) -> Vec<Violation> {
        let banned = &ctx.config.deps.banned_mock_paths;
        if banned.is_empty() {
            return vec![];
        }
        module
            .vi_mocks
            .iter()
            .filter(|m| matches_path(banned, &m.source))
            .map(|m| Violation {
                rule_id: self.id().to_string(),
                rule_name: self.name().to_string(),
                severity: self.severity(),
                category: self.category(),
                message: format!(
                    "vi.mock('{}') leaks across test files via singleton module cache",
                    m.source
                ),
                file_path: module.file_path.clone(),
                line: m.line,
                col: None,
                suggestion: Some(
                    "Refactor target service to accept the dependency via constructor (DI). Construct the service in tests with a fake."
                        .to_string(),
                ),
                test_name: None,
            })
            .collect()
    }
}

pub struct ProductionSingletonImportRule;

impl Rule for ProductionSingletonImportRule {
    fn id(&self) -> &'static str {
        "VITEST-DEP-002"
    }
    fn name(&self) -> &'static str {
        "ProductionSingletonImportRule"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
    fn category(&self) -> Category {
        Category::Dependencies
    }
    fn check(&self, module: &ParsedModule, ctx: &LintContext<'_>) -> Vec<Violation> {
        let banned = &ctx.config.deps.banned_singletons;
        if banned.is_empty() {
            return vec![];
        }
        // Skip integration tests — production singletons are the contract there.
        if let Some(g) = ctx.config.deps.integration_test_glob.as_ref() {
            if g.is_match(&module.file_path) {
                return vec![];
            }
        }

        let mut out = Vec::new();
        for imp in &module.imports_parsed {
            for ban in banned {
                if !ban.from.is_match(imp.source.as_str())
                    && !ban.from.is_match(strip_relative(imp.source.as_str()))
                {
                    continue;
                }
                // Check named imports.
                for name in &imp.named {
                    if ban.names.iter().any(|n| n == name) {
                        out.push(Violation {
                            rule_id: self.id().to_string(),
                            rule_name: self.name().to_string(),
                            severity: self.severity(),
                            category: self.category(),
                            message: format!(
                                "Importing production singleton `{}` from `{}` triggers its constructor side effects in unit tests",
                                name, imp.source
                            ),
                            file_path: module.file_path.clone(),
                            line: imp.line,
                            col: None,
                            suggestion: Some(
                                "Construct a fresh instance with fakes (DI). Singletons belong in *.integration.test.ts only."
                                    .to_string(),
                            ),
                            test_name: None,
                        });
                    }
                }
                // Check default import.
                if let Some(default_name) = &imp.default {
                    if ban.names.iter().any(|n| n == default_name) {
                        out.push(Violation {
                            rule_id: self.id().to_string(),
                            rule_name: self.name().to_string(),
                            severity: self.severity(),
                            category: self.category(),
                            message: format!(
                                "Importing production singleton `{}` from `{}` triggers its constructor side effects in unit tests",
                                default_name, imp.source
                            ),
                            file_path: module.file_path.clone(),
                            line: imp.line,
                            col: None,
                            suggestion: Some(
                                "Construct a fresh instance with fakes (DI). Singletons belong in *.integration.test.ts only."
                                    .to_string(),
                            ),
                            test_name: None,
                        });
                    }
                }
            }
        }
        out
    }
}

pub struct ResetEscapeHatchRule;

const ESCAPE_HATCH_CALLS: &[&str] = &[
    "vi.resetModules",
    "vi.restoreAllMocks",
    "vi.unmock",
    "vi.doUnmock",
];

impl Rule for ResetEscapeHatchRule {
    fn id(&self) -> &'static str {
        "VITEST-DEP-003"
    }
    fn name(&self) -> &'static str {
        "ResetEscapeHatchRule"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
    fn category(&self) -> Category {
        Category::Dependencies
    }
    fn check(&self, module: &ParsedModule, _ctx: &LintContext<'_>) -> Vec<Violation> {
        let mut out = Vec::new();
        for hook in &module.hook_calls {
            for call in &hook.vi_calls {
                if ESCAPE_HATCH_CALLS.iter().any(|c| c == call) {
                    out.push(Violation {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        severity: self.severity(),
                        category: self.category(),
                        message: format!(
                            "{}() in {:?} masks underlying coupling between test files",
                            call, hook.kind
                        ),
                        file_path: module.file_path.clone(),
                        line: hook.line,
                        col: None,
                        suggestion: Some(
                            "Fix the coupling: stop using module-level vi.mock for shared infrastructure, and stop importing production singletons in unit tests."
                                .to_string(),
                        ),
                        test_name: None,
                    });
                }
            }
        }
        out
    }
}

fn strip_relative(s: &str) -> &str {
    let mut s = s.trim_start_matches("./");
    while let Some(rest) = s.strip_prefix("../") {
        s = rest;
    }
    s
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
        // Replace temp path with the simple file name so glob tests can target it.
        module.file_path = PathBuf::from(name);
        module
    }

    fn cfg(text: &str) -> Config {
        Config::parse_toml(text).unwrap()
    }

    #[test]
    fn dep_001_flags_banned_module_mock() {
        let module = parse(
            r#"
import { vi } from 'vitest';
vi.mock('../infrastructure/database', () => ({ db: {} }));
"#,
            "progress.test.ts",
        );
        let cfg = cfg(r#"
[deps]
banned_mock_paths = ["**/infrastructure/database"]
"#);
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = BannedModuleMockRule.check(&module, &ctx);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-DEP-001");
    }

    #[test]
    fn dep_001_passes_when_path_not_banned() {
        let module = parse(
            r#"
import { vi } from 'vitest';
vi.mock('./local-helper');
"#,
            "ok.test.ts",
        );
        let cfg = cfg(r#"
[deps]
banned_mock_paths = ["**/infrastructure/database"]
"#);
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = BannedModuleMockRule.check(&module, &ctx);
        assert!(v.is_empty());
    }

    #[test]
    fn dep_001_inactive_with_empty_config() {
        let module = parse(
            r#"
import { vi } from 'vitest';
vi.mock('../infrastructure/database');
"#,
            "noconfig.test.ts",
        );
        let cfg = Config::default();
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = BannedModuleMockRule.check(&module, &ctx);
        assert!(v.is_empty());
    }

    #[test]
    fn dep_002_flags_singleton_import_in_unit_test() {
        let module = parse(
            r#"
import { progressPersistence } from './progress-persistence';
"#,
            "pipeline.test.ts",
        );
        let cfg = cfg(r#"
[[deps.banned_singletons]]
from = "**/progress-persistence"
names = ["progressPersistence"]
"#);
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = ProductionSingletonImportRule.check(&module, &ctx);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-DEP-002");
    }

    #[test]
    fn dep_002_skips_integration_test_files() {
        let module = parse(
            r#"
import { progressPersistence } from './progress-persistence';
"#,
            "pipeline.integration.test.ts",
        );
        let cfg = cfg(r#"
[[deps.banned_singletons]]
from = "**/progress-persistence"
names = ["progressPersistence"]
"#);
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = ProductionSingletonImportRule.check(&module, &ctx);
        assert!(v.is_empty());
    }

    #[test]
    fn dep_002_ignores_non_banned_names() {
        let module = parse(
            r#"
import { ProgressPersistenceService } from './progress-persistence';
"#,
            "pipeline.test.ts",
        );
        let cfg = cfg(r#"
[[deps.banned_singletons]]
from = "**/progress-persistence"
names = ["progressPersistence"]
"#);
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = ProductionSingletonImportRule.check(&module, &ctx);
        assert!(v.is_empty());
    }

    #[test]
    fn dep_002_flags_default_import_singleton() {
        let module = parse(
            r#"
import db from './infrastructure/database';
"#,
            "pipeline.test.ts",
        );
        let cfg = cfg(r#"
[[deps.banned_singletons]]
from = "**/infrastructure/database"
names = ["db"]
"#);
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = ProductionSingletonImportRule.check(&module, &ctx);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule_id, "VITEST-DEP-002");
    }

    #[test]
    fn dep_003_flags_reset_in_before_each() {
        let module = parse(
            r#"
import { beforeEach, vi } from 'vitest';
beforeEach(() => {
    vi.resetModules();
    vi.restoreAllMocks();
});
"#,
            "hooks.test.ts",
        );
        let cfg = Config::default();
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = ResetEscapeHatchRule.check(&module, &ctx);
        assert_eq!(v.len(), 2);
        assert!(v.iter().all(|x| x.rule_id == "VITEST-DEP-003"));
    }

    #[test]
    fn dep_003_ignores_clear_all_mocks() {
        let module = parse(
            r#"
import { beforeEach, vi } from 'vitest';
beforeEach(() => { vi.clearAllMocks(); });
"#,
            "ok.test.ts",
        );
        let cfg = Config::default();
        let ctx = LintContext {
            config: &cfg,
            all_modules: &[],
        };
        let v = ResetEscapeHatchRule.check(&module, &ctx);
        assert!(v.is_empty());
    }
}
