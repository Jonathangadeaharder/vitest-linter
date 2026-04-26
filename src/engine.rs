use std::path::PathBuf;

use walkdir::WalkDir;

use crate::models::Violation;
use crate::parser::TsParser;
use crate::rules::all_rules;

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

        let rules = all_rules();
        let mut violations = Vec::new();

        for rule in &rules {
            for module in &modules {
                let mut v = rule.check(module, &modules);
                violations.append(&mut v);
            }
        }

        violations.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| a.line.cmp(&b.line))
        });

        Ok(violations)
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
    name.ends_with(".test.ts")
        || name.ends_with(".spec.ts")
        || name.ends_with(".test.js")
        || name.ends_with(".spec.js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_accepted_extensions() {
        assert!(is_test_file("foo.test.ts"));
        assert!(is_test_file("foo.spec.ts"));
        assert!(is_test_file("foo.test.js"));
        assert!(is_test_file("foo.spec.js"));
        assert!(is_test_file("path/to/bar.test.ts"));
        assert!(is_test_file("path/to/bar.spec.js"));
    }

    #[test]
    fn test_file_rejected_extensions() {
        assert!(!is_test_file("foo.ts"));
        assert!(!is_test_file("foo.js"));
        assert!(!is_test_file("foo.tsx"));
        assert!(!is_test_file("foo.jsx"));
        assert!(!is_test_file("foo.test.tsx"));
        assert!(!is_test_file("foo.spec.jsx"));
        assert!(!is_test_file("utils.ts"));
        assert!(!is_test_file("README.md"));
        assert!(!is_test_file("foo.test.py"));
        assert!(!is_test_file("test.ts"));
    }
}
