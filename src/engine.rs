use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::config::Config;
use crate::models::{ParsedModule, Violation};
use crate::parser::TsParser;
use crate::rules::{all_rules, LintContext};
use crate::suppression::SuppressionMap;

pub struct LintEngine {
    parser: TsParser,
}

impl LintEngine {
    #[allow(clippy::missing_errors_doc)]
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            parser: TsParser::new()?,
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn lint_paths(&self, paths: &[PathBuf]) -> anyhow::Result<Vec<Violation>> {
        let files = Self::discover_files(paths);

        // Parse files in parallel using rayon
        let parsed: Vec<_> = files
            .par_iter()
            .filter_map(|file| {
                let source = std::fs::read_to_string(file).ok()?;
                let module = self.parser.parse_file(file).ok()?;
                Some((module, source))
            })
            .collect();

        let mut modules = Vec::with_capacity(parsed.len());
        let mut sources = Vec::with_capacity(parsed.len());
        for (module, source) in parsed {
            modules.push(module);
            sources.push(source);
        }

        // Group modules by their resolved config root so each module is
        // evaluated against the nearest .vitest-linter.toml.
        let mut groups: HashMap<PathBuf, (Config, Vec<usize>)> = HashMap::new();
        for (idx, module) in modules.iter().enumerate() {
            let config_root = Self::resolve_config_root(&module.file_path);
            let entry = groups.entry(config_root).or_insert_with_key(|root| {
                let config = Config::load_from(root).unwrap_or_default();
                (config, Vec::new())
            });
            entry.1.push(idx);
        }

        let rules = all_rules();
        let mut violations = Vec::new();

        for (config, indices) in groups.values() {
            let group_modules: Vec<ParsedModule> =
                indices.iter().map(|i| modules[*i].clone()).collect();
            let ctx = LintContext {
                config,
                all_modules: &group_modules,
            };
            for rule in &rules {
                // Skip disabled rules
                if config.rules.is_disabled(rule.id()) {
                    continue;
                }
                for (local_idx, module) in group_modules.iter().enumerate() {
                    let global_idx = indices[local_idx];
                    let mut v = rule.check(module, &ctx);
                    // Apply severity overrides
                    if let Some(override_sev) = config.rules.severity_override(rule.id()) {
                        for violation in &mut v {
                            violation.severity = match override_sev.to_ascii_lowercase().as_str() {
                                "error" => crate::models::Severity::Error,
                                "warning" => crate::models::Severity::Warning,
                                "info" => crate::models::Severity::Info,
                                _ => violation.severity,
                            };
                        }
                    }
                    // Filter suppressed violations
                    let suppression = SuppressionMap::parse(&sources[global_idx]);
                    v.retain(|violation| {
                        !suppression.is_suppressed(violation.line, &violation.rule_id)
                    });
                    violations.append(&mut v);
                }
            }
        }

        violations.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.line.cmp(&b.line))
        });

        Ok(violations)
    }

    /// Walk up from the module's path to find the directory containing
    /// `.vitest-linter.toml`, falling back to the module's parent directory.
    fn resolve_config_root(path: &Path) -> PathBuf {
        let dir = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(Path::new(".")).to_path_buf()
        };
        let mut cur = Some(dir.as_path());
        while let Some(d) = cur {
            if d.join(".vitest-linter.toml").is_file() {
                return d.to_path_buf();
            }
            cur = d.parent();
        }
        dir
    }

    fn discover_files(paths: &[PathBuf]) -> Vec<PathBuf> {
        let candidates: Vec<PathBuf> = paths
            .par_iter()
            .flat_map_iter(|path| {
                if path.is_file() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if is_test_file(&name) {
                        vec![path.clone()]
                    } else {
                        vec![]
                    }
                } else if path.is_dir() {
                    WalkDir::new(path)
                        .into_iter()
                        .filter_map(std::result::Result::ok)
                        .filter(|entry| {
                            let name = entry.file_name().to_string_lossy().to_string();
                            is_test_file(&name)
                        })
                        .map(|entry| entry.into_path())
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect();

        let mut files = candidates;
        files.sort();
        files.dedup();
        files
    }
}

fn is_test_file(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.ends_with(".test.ts")
        || name.ends_with(".spec.ts")
        || name.ends_with(".test.tsx")
        || name.ends_with(".spec.tsx")
        || name.ends_with(".test.js")
        || name.ends_with(".spec.js")
        || name.ends_with(".test.jsx")
        || name.ends_with(".spec.jsx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_accepted_extensions() {
        assert!(is_test_file("foo.test.ts"));
        assert!(is_test_file("foo.spec.ts"));
        assert!(is_test_file("foo.test.tsx"));
        assert!(is_test_file("foo.spec.tsx"));
        assert!(is_test_file("foo.test.js"));
        assert!(is_test_file("foo.spec.js"));
        assert!(is_test_file("foo.test.jsx"));
        assert!(is_test_file("foo.spec.jsx"));
        assert!(is_test_file("path/to/bar.test.ts"));
        assert!(is_test_file("path/to/bar.spec.js"));
        assert!(is_test_file("path/to/baz.test.tsx"));
        // Case-insensitive matching
        assert!(is_test_file("Foo.TEST.ts"));
        assert!(is_test_file("path/To/BaZ.Spec.JS"));
        assert!(is_test_file("UPPER.TEST.TSX"));
    }

    #[test]
    fn test_file_rejected_extensions() {
        assert!(!is_test_file("foo.ts"));
        assert!(!is_test_file("foo.js"));
        assert!(!is_test_file("foo.tsx"));
        assert!(!is_test_file("foo.jsx"));
        assert!(!is_test_file("utils.ts"));
        assert!(!is_test_file("README.md"));
        assert!(!is_test_file("foo.test.py"));
        assert!(!is_test_file("test.ts"));
    }
}
