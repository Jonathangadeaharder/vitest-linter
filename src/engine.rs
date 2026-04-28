use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::config::Config;
use crate::models::{ParsedModule, Violation};
use crate::parser::TsParser;
use crate::rules::{all_rules, LintContext};

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
        let mut modules = Vec::new();

        for file in &files {
            match self.parser.parse_file(file) {
                Ok(m) => modules.push(m),
                Err(e) => eprintln!("Warning: Failed to parse {}: {e}", file.display()),
            }
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
                for module in &group_modules {
                    let mut v = rule.check(module, &ctx);
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
        let mut files = Vec::new();

        for path in paths {
            if path.is_file() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if is_test_file(&name) {
                    files.push(path.clone());
                }
            } else if path.is_dir() {
                for entry in WalkDir::new(path)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if is_test_file(&name) {
                        files.push(entry.into_path());
                    }
                }
            }
        }

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
