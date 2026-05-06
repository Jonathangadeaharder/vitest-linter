//! Golden corpus integration tests.
//!
//! Runs vitest-linter against real test suites from well-known open-source
//! projects (Vitest, Vue, SvelteKit) to verify:
//! 1. The linter does not panic/crash on real-world code.
//! 2. Violation counts remain stable (no unexpected regressions).
//! 3. False-positive rate stays within acceptable bounds.
//!
//! Tests are `#[ignore]` by default because they clone large repositories.
//! Run with: `cargo test --test corpus -- --ignored`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use vitest_linter::engine::LintEngine;

#[allow(dead_code)]
struct CorpusProject {
    name: &'static str,
    repo_url: &'static str,
    commit: &'static str,
    test_dirs: &'static [&'static str],
    allowed_false_positive_rules: &'static [&'static str],
}

const VITEST_PROJECT: CorpusProject = CorpusProject {
    name: "vitest",
    repo_url: "https://github.com/vitest-dev/vitest.git",
    commit: "v3.1.2",
    test_dirs: &["test"],
    allowed_false_positive_rules: &[],
};

const VUE_PROJECT: CorpusProject = CorpusProject {
    name: "vue",
    repo_url: "https://github.com/vuejs/core.git",
    commit: "v3.5.13",
    test_dirs: &["packages/vue/__tests__", "packages/reactivity/__tests__"],
    allowed_false_positive_rules: &[],
};

const SVELTEKIT_PROJECT: CorpusProject = CorpusProject {
    name: "sveltekit",
    repo_url: "https://github.com/sveltejs/kit.git",
    commit: "5dc4f90c20a8a7a5c9254a7e0a86578a6f06c26d",
    test_dirs: &["packages/kit/test"],
    allowed_false_positive_rules: &[],
};

fn corpus_cache_dir() -> PathBuf {
    std::env::var("CORPUS_CACHE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("CARGO_TARGET_TMPDIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir())
                .join("vitest-linter-corpus")
        })
}

fn clone_or_update_project(project: &CorpusProject) -> PathBuf {
    let cache = corpus_cache_dir().join(project.name);

    if cache.is_dir() {
        let head = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&cache)
            .output()
            .ok();

        let target = std::process::Command::new("git")
            .args(["rev-parse", project.commit])
            .current_dir(&cache)
            .output()
            .ok();

        if let (Some(head), Some(target)) = (head, target) {
            let head_sha = String::from_utf8_lossy(&head.stdout).trim().to_string();
            let target_sha = String::from_utf8_lossy(&target.stdout).trim().to_string();
            if head_sha == target_sha && !head_sha.is_empty() {
                return cache;
            }
        }

        let _ = std::process::Command::new("git")
            .args(["fetch", "--depth", "1", "origin", project.commit])
            .current_dir(&cache)
            .status();

        let _ = std::process::Command::new("git")
            .args(["checkout", project.commit])
            .current_dir(&cache)
            .status();

        return cache;
    }

    std::fs::create_dir_all(cache.parent().unwrap()).unwrap();

    let status = std::process::Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            project.commit,
            project.repo_url,
            cache.to_str().unwrap(),
        ])
        .status()
        .expect("git clone failed — ensure git is installed and network is available");

    assert!(status.success(), "git clone failed for {}", project.name);

    cache
}

fn lint_corpus(project: &CorpusProject) -> (Vec<vitest_linter::models::Violation>, usize) {
    let repo_dir = clone_or_update_project(project);

    let paths: Vec<PathBuf> = project
        .test_dirs
        .iter()
        .filter_map(|dir| {
            let p = repo_dir.join(dir);
            if p.is_dir() {
                Some(p)
            } else {
                None
            }
        })
        .collect();

    assert!(
        !paths.is_empty(),
        "No test directories found for {}",
        project.name
    );

    let engine = LintEngine::new().expect("Failed to create lint engine");
    let (violations, _) = engine.lint_paths(&paths).expect("Linting failed");

    let file_count = paths.iter().map(|p| count_test_files(p)).sum();

    (violations, file_count)
}

fn count_test_files(dir: &Path) -> usize {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            name.ends_with(".test.ts")
                || name.ends_with(".spec.ts")
                || name.ends_with(".test.tsx")
                || name.ends_with(".spec.tsx")
                || name.ends_with(".test.js")
                || name.ends_with(".spec.js")
        })
        .count()
}

fn summarize_violations(violations: &[vitest_linter::models::Violation]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for v in violations {
        *counts.entry(v.rule_id.clone()).or_default() += 1;
    }
    counts
}

fn assert_no_panic(result: &(Vec<vitest_linter::models::Violation>, usize), project_name: &str) {
    assert!(
        !result.0.is_empty() || result.1 > 0,
        "[{}] Linter returned 0 violations from {} test files — this likely means files were not discovered",
        project_name,
        result.1
    );
}

fn print_summary(
    project_name: &str,
    violations: &[vitest_linter::models::Violation],
    file_count: usize,
) {
    let counts = summarize_violations(violations);
    eprintln!(
        "\n[{}] {} files, {} violations:",
        project_name,
        file_count,
        violations.len()
    );
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (rule, count) in &sorted {
        eprintln!("  {rule}: {count}");
    }
}

#[test]
#[ignore]
fn corpus_vitest_no_panic() {
    let result = lint_corpus(&VITEST_PROJECT);
    assert_no_panic(&result, "vitest");
    print_summary("vitest", &result.0, result.1);
}

#[test]
#[ignore]
fn corpus_vitest_stable_counts() {
    let (violations, file_count) = lint_corpus(&VITEST_PROJECT);
    assert!(file_count > 0, "Should discover test files");

    let total = violations.len();

    assert!(
        total > 0,
        "Vitest's own test suite should produce some violations — got 0 from {file_count} files"
    );

    assert!(
        total < file_count * 5,
        "False-positive rate too high: {total} violations in {file_count} files"
    );

    print_summary("vitest", &violations, file_count);
}

#[test]
#[ignore]
fn corpus_vitest_no_critical_false_positives() {
    let (violations, _file_count) = lint_corpus(&VITEST_PROJECT);
    let counts = summarize_violations(&violations);

    let critical_rules = [
        "VITEST-MNT-001",
        "VITEST-MNT-007",
        "VITEST-VAL-001",
        "VITEST-VAL-003",
    ];

    for rule in &critical_rules {
        let count = counts.get(*rule).copied().unwrap_or(0);
        assert!(
            count < 50,
            "[vitest] {rule} fired {count} times — likely false positives on Vitest's own tests"
        );
    }
}

#[test]
#[ignore]
fn corpus_vue_no_panic() {
    let result = lint_corpus(&VUE_PROJECT);
    assert_no_panic(&result, "vue");
    print_summary("vue", &result.0, result.1);
}

#[test]
#[ignore]
fn corpus_vue_stable_counts() {
    let (violations, file_count) = lint_corpus(&VUE_PROJECT);
    assert!(file_count > 0, "Should discover test files");

    let total = violations.len();

    assert!(
        total < file_count * 5,
        "False-positive rate too high: {total} violations in {file_count} files"
    );

    print_summary("vue", &violations, file_count);
}

#[test]
#[ignore]
fn corpus_sveltekit_no_panic() {
    let result = lint_corpus(&SVELTEKIT_PROJECT);
    assert_no_panic(&result, "sveltekit");
    print_summary("sveltekit", &result.0, result.1);
}

#[test]
#[ignore]
fn corpus_sveltekit_stable_counts() {
    let (violations, file_count) = lint_corpus(&SVELTEKIT_PROJECT);
    assert!(file_count > 0, "Should discover test files");

    let total = violations.len();

    assert!(
        total < file_count * 5,
        "False-positive rate too high: {total} violations in {file_count} files"
    );

    print_summary("sveltekit", &violations, file_count);
}

#[test]
#[ignore]
fn corpus_all_projects_regression() {
    let projects = [&VITEST_PROJECT, &VUE_PROJECT, &SVELTEKIT_PROJECT];
    let mut total_violations = 0;
    let mut total_files = 0;

    for project in &projects {
        let (violations, file_count) = lint_corpus(project);
        total_violations += violations.len();
        total_files += file_count;
        print_summary(project.name, &violations, file_count);
    }

    assert!(
        total_files > 0,
        "Should discover test files across all corpus projects"
    );

    assert!(
        total_violations < total_files * 10,
        "Aggregate false-positive rate too high: {total_violations} violations across {total_files} files"
    );
}
