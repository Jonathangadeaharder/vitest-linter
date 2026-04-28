use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobMatcher};
use serde::Deserialize;

const CONFIG_FILE_NAME: &str = ".vitest-linter.toml";
const DEFAULT_INTEGRATION_GLOB: &str = "**/*.integration.test.{ts,tsx,js,jsx}";

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    deps: RawDepsConfig,
    #[serde(default)]
    rules: RawRulesConfig,
}

#[derive(Debug, Deserialize, Default)]
struct RawRulesConfig {
    /// Per-rule severity overrides: {"VITEST-FLK-001": "off", "VITEST-MNT-001": "warning"}
    #[serde(default)]
    select: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawDepsConfig {
    #[serde(default)]
    banned_mock_paths: Vec<String>,
    #[serde(default)]
    banned_singletons: Vec<RawBannedSingleton>,
    #[serde(default)]
    integration_test_glob: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawBannedSingleton {
    from: String,
    #[serde(default)]
    names: Vec<String>,
}

/// Parsed configuration from `.vitest-linter.toml` and `package.json`.
#[derive(Debug)]
pub struct Config {
    pub deps: DepsConfig,
    pub rules: RulesConfig,
}

/// Per-rule severity overrides and enable/disable settings.
#[derive(Debug, Default)]
pub struct RulesConfig {
    /// Per-rule severity overrides. Key = rule ID, value = "off" | "info" | "warning" | "error"
    pub select: HashMap<String, String>,
}

impl RulesConfig {
    /// Returns `true` if the given rule is turned off.
    #[must_use]
    pub fn is_disabled(&self, rule_id: &str) -> bool {
        self.select
            .get(rule_id)
            .is_some_and(|v| v.eq_ignore_ascii_case("off"))
    }

    /// Returns an overridden severity string for the rule, if any.
    #[must_use]
    pub fn severity_override(&self, rule_id: &str) -> Option<&str> {
        self.select.get(rule_id).map(|s| s.as_str())
    }
}

#[derive(Debug, Default)]
pub struct DepsConfig {
    pub banned_mock_paths: Vec<GlobMatcher>,
    pub banned_singletons: Vec<BannedSingleton>,
    pub integration_test_glob: Option<GlobMatcher>,
}

#[derive(Debug)]
pub struct BannedSingleton {
    pub from: GlobMatcher,
    pub names: Vec<String>,
}

impl Config {
    /// Load `.vitest-linter.toml` by walking up from `start` until found, or
    /// return an empty config when no file exists. Also checks `package.json`
    /// for a `vitest-linter` key and merges it.
    #[allow(clippy::missing_errors_doc)]
    pub fn load_from(start: &Path) -> Result<Self> {
        let mut config = if let Some(found) = find_config(start) {
            Self::from_path(&found)?
        } else {
            Self::default()
        };

        // Check for package.json override
        if let Some(pkg_dir) = find_package_json_dir(start) {
            let pkg_path = pkg_dir.join("package.json");
            if let Ok(pkg_text) = std::fs::read_to_string(&pkg_path) {
                if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&pkg_text) {
                    if let Some(vl_config) = pkg.get("vitest-linter") {
                        if let Some(select) = vl_config.get("select").and_then(|s| s.as_object()) {
                            for (key, val) in select {
                                if let Some(severity) = val.as_str() {
                                    config
                                        .rules
                                        .select
                                        .insert(key.clone(), severity.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(config)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn from_path(path: &Path) -> Result<Self> {
        let raw_text = std::fs::read_to_string(path)
            .with_context(|| format!("reading config {}", path.display()))?;
        Self::parse_toml(&raw_text)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn parse_toml(text: &str) -> Result<Self> {
        let raw: RawConfig =
            toml::from_str(text).with_context(|| "parsing vitest-linter config")?;
        Self::from_raw(raw)
    }

    fn from_raw(raw: RawConfig) -> Result<Self> {
        let mut banned_mock_paths = Vec::new();
        for pat in &raw.deps.banned_mock_paths {
            banned_mock_paths.push(compile_glob(pat)?);
        }
        let mut banned_singletons = Vec::new();
        for entry in raw.deps.banned_singletons {
            banned_singletons.push(BannedSingleton {
                from: compile_glob(&entry.from)?,
                names: entry.names,
            });
        }
        let integration_test_glob = match raw.deps.integration_test_glob.as_deref() {
            Some(p) => Some(compile_glob(p)?),
            None => Some(compile_glob(DEFAULT_INTEGRATION_GLOB)?),
        };
        Ok(Self {
            deps: DepsConfig {
                banned_mock_paths,
                banned_singletons,
                integration_test_glob,
            },
            rules: RulesConfig {
                select: raw.rules.select,
            },
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        let integration_test_glob = compile_glob(DEFAULT_INTEGRATION_GLOB).ok();
        Self {
            deps: DepsConfig {
                banned_mock_paths: Vec::new(),
                banned_singletons: Vec::new(),
                integration_test_glob,
            },
            rules: RulesConfig::default(),
        }
    }
}

fn compile_glob(pat: &str) -> Result<GlobMatcher> {
    Ok(Glob::new(pat)
        .with_context(|| format!("compiling glob `{pat}`"))?
        .compile_matcher())
}

fn find_config(start: &Path) -> Option<PathBuf> {
    let mut cur = if start.is_dir() {
        Some(start.to_path_buf())
    } else {
        start.parent().map(Path::to_path_buf)
    };
    while let Some(dir) = cur {
        let candidate = dir.join(CONFIG_FILE_NAME);
        if candidate.is_file() {
            return Some(candidate);
        }
        cur = dir.parent().map(Path::to_path_buf);
    }
    None
}

fn find_package_json_dir(start: &Path) -> Option<PathBuf> {
    let mut cur = if start.is_dir() {
        Some(start.to_path_buf())
    } else {
        start.parent().map(Path::to_path_buf)
    };
    while let Some(dir) = cur {
        let candidate = dir.join("package.json");
        if candidate.is_file() {
            return Some(dir);
        }
        cur = dir.parent().map(Path::to_path_buf);
    }
    None
}

/// Match a `vi.mock(<source>)` source string against the banlist. The check
/// matches both the literal pattern and a normalized form without leading `./`
/// or `../` segments to keep config simple for callers.
#[must_use]
pub fn matches_path(matchers: &[GlobMatcher], source: &str) -> bool {
    if matchers.is_empty() {
        return false;
    }
    let normalized = source.trim_start_matches("./");
    let stripped = trim_relative_prefix(normalized);
    matchers
        .iter()
        .any(|g| g.is_match(source) || g.is_match(normalized) || g.is_match(stripped))
}

fn trim_relative_prefix(s: &str) -> &str {
    let mut s = s;
    loop {
        if let Some(rest) = s.strip_prefix("../") {
            s = rest;
        } else {
            return s;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_default_integration_glob() {
        let cfg = Config::default();
        assert!(cfg.deps.integration_test_glob.is_some());
        let g = cfg.deps.integration_test_glob.unwrap();
        assert!(g.is_match("foo/bar.integration.test.ts"));
        assert!(!g.is_match("foo/bar.test.ts"));
    }

    #[test]
    fn parse_full_config() {
        let text = r#"
[deps]
banned_mock_paths = [
  "**/infrastructure/database",
  "**/infrastructure/event-bus",
]
integration_test_glob = "**/*.integration.test.ts"

[[deps.banned_singletons]]
from = "**/services/pipeline-orchestrator"
names = ["orchestrator"]

[[deps.banned_singletons]]
from = "**/services/progress-persistence"
names = ["progressPersistence"]
"#;
        let cfg = Config::parse_toml(text).unwrap();
        assert_eq!(cfg.deps.banned_mock_paths.len(), 2);
        assert_eq!(cfg.deps.banned_singletons.len(), 2);
        assert!(matches_path(
            &cfg.deps.banned_mock_paths,
            "../infrastructure/database",
        ));
        assert!(matches_path(
            &cfg.deps.banned_mock_paths,
            "src/lib/infrastructure/event-bus",
        ));
        assert!(!matches_path(
            &cfg.deps.banned_mock_paths,
            "../infrastructure/audio",
        ));
    }

    #[test]
    fn singleton_glob_match() {
        let text = r#"
[[deps.banned_singletons]]
from = "**/services/pipeline-orchestrator"
names = ["orchestrator"]
"#;
        let cfg = Config::parse_toml(text).unwrap();
        let s = &cfg.deps.banned_singletons[0];
        // Glob requires `services/` segment present somewhere in the path.
        assert!(s
            .from
            .is_match("apps/platform/src/lib/server/services/pipeline-orchestrator"));
        assert!(!s.from.is_match("./pipeline-orchestrator"));
    }

    #[test]
    fn find_config_walks_up() {
        let dir = tempfile::TempDir::new().unwrap();
        let nested = dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        let cfg_path = dir.path().join(CONFIG_FILE_NAME);
        std::fs::write(&cfg_path, "").unwrap();
        let found = find_config(&nested).unwrap();
        assert_eq!(found, cfg_path);
    }
}
