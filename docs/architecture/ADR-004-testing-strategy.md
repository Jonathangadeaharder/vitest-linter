---
id: ADR-004
kind: adr
title: Testing Strategy
status: accepted
date: 2026-05-17T00:00:00.000Z
authors: [Jonathan Gadea Harder]
reviewers: [Jonathan Gadea Harder]
tags: []
supersedes: []
superseded_by: []
depends_on: []
blocks: []
implements: []
related: []
external: []
project: vitest-linter
checksum: ea802cc14c8f64a0a15ddd317a89b5afaf1505e068b42b8924cc5ff5f2cf5d8d
---

**Deciders:** @Jonathangadeaharder

## Context

vitest-linter is a linting tool — correctness of every rule is critical. False positives erode trust, false negatives leave bugs undetected. Testing strategy must cover unit, integration, mutation, and regression testing.

## Decision

### Test Levels

| Level | Tool | Location | Scope |
|-------|------|----------|-------|
| Unit | `cargo test` (inline `#[cfg(test)]` modules) | Each `src/` module | Parser methods, config parsing, suppression logic, model invariants |
| Integration | `cargo test` (separate `tests/` dir) | `tests/integration_tests.rs` (4000+ lines, 100+ tests) | Full end-to-end: write tempfile → parse → lint → assert violations |
| Corpus | `cargo test` | `tests/corpus.rs` | Regression tests against real-world `.test.ts` fixtures |
| Mutation | `cargo-mutants` | `.cargo/mutants.toml` (excludes benches, tests) | Score ≥75% gate in scheduled CI |
| Benchmark | `cargo bench` (criterion) | `benches/lint_large_corpus.rs` | Performance regression detection |

### Coverage Requirements

- **Tool:** `cargo +nightly llvm-cov` with `--lcov` output.
- **Threshold:** Line coverage ≥90% (enforced in CI via `--fail-under-lines 90`).
- **Report format:** `lcov` (consumed by SonarCloud).
- **Project key (SonarCloud):** `Jonathangadeaharder_vitest-linter`

### Mutation Testing

- **Tool:** `cargo-mutants` (via `taiki-e/install-action`).
- **Schedule:** Daily at 3 AM UTC (`cron: "0 3 * * *"`), also `workflow_dispatch`.
- **Runner:** Self-hosted (cost-effective for long-running mutation analysis).
- **Exclusions:** `benches/`, `tests/` (test harness code itself excluded).
- **Threshold:** Score ≥75%. Fails if below.

### Test Patterns

- **Integration tests:** Write temp `.test.ts` file → `LintEngine::new(true)` → `lint_paths()` → find specific `rule_id` violation.
- **Parser tests:** Write temp file → `TsParser::parse_file()` → assert field values on `ParsedModule`.
- **Suppression tests:** Include `// vitest-linter-disable-next-line` comments in fixture → assert violation is suppressed.
- **CLI tests:** `run_cli()` with various `--format` flags, assert JSON/terminal/sarif output.

## Consequences

- Every new rule requires at minimum: 1 positive integration test (triggers) + 1 negative test (doesn't trigger on clean code).
- Parser changes require inline unit tests for new detection functions.
- Mutation testing runs daily, not on every PR (too expensive at 5+ min).
- Line coverage of 90% is achievable (current is 98.95% lines, 92.42% branches).
