# Design: Rust CI Workflow for pytest-linter/vitest-linter

**Date:** 2026-04-26  
**Status:** Approved  
**Branch:** `feat/rust-rewrite`

## Summary

Replace the stale Python-era `ci.yml` with a Rust-native CI pipeline. Delete 4 obsolete Python workflows. The new workflow covers: formatting, clippy linting, tests, branch coverage with line threshold gating, and dogfooding the linters on sample test files.

## Triggers

- Push to `main`, `master`, `develop`
- Pull requests targeting `main`, `master`, `develop`

## Jobs

### 1. fmt (Format Check)
- `cargo fmt --check --all`
- Gate: Yes (fails on unformatted code)

### 2. clippy (Lint)
- Install nightly toolchain with clippy component
- Run on both crates: `cargo +nightly clippy --lib -- -W clippy::all -W clippy::pedantic -W clippy::nursery`
- Gate: Yes (zero warnings required)

### 3. test (Unit + Integration Tests)
- `cargo test --all` (runs both crates)
- Gate: Yes (all tests must pass)

### 4. coverage (Branch Coverage)
- Install nightly toolchain with `llvm-tools-preview`
- Install `cargo-llvm-cov`
- Run on both crates: `cargo +nightly llvm-cov --fail-under-lines 90`
- Gate: Yes (lines ≥90%)
- Branch coverage is reported but not gated (llvm-cov lacks `--fail-under-branches`)

### 5. dogfood (Self-Test)
- Depends on: test job
- Build both crates in release mode
- Create temp Python test files with intentional smells (time.sleep, file I/O without tmp_path, etc.)
- Create temp TypeScript test files with intentional smells
- Run each linter on the test files
- Verify violations are found (check exit code or parse JSON output)
- Gate: Yes (linters must detect expected smells without crashing)

## Workflow Setup

- **Runner:** `ubuntu-latest`
- **Rust toolchain:** `dtolnay/rust-toolchain@stable` for fmt/test, nightly for clippy/coverage
- **Cache:** `Swatinem/rust-cache@v2` for Cargo target + registry
- **Permissions:** `contents: read` (no write needed)

## Files to Delete

- `.github/workflows/publish.yml` — references `setup.py`, Python packages
- `.github/workflows/release.yml` — references Python release process
- `.github/workflows/security.yml` — runs `bandit` (Python scanner)
- `.github/workflows/repomix.yml` — utility, not Rust-relevant

## Files to Modify

- `.github/workflows/ci.yml` — complete rewrite for Rust

## Coverage Thresholds

| Crate | Current Lines | Threshold | Current Branches |
|-------|:------------:|:---------:|:----------------:|
| pytest-linter | 96.63% | ≥90% | 80.53% |
| vitest-linter | 98.95% | ≥90% | 92.42% |

## Out of Scope

- Mutation testing (too expensive for CI, run locally)
- Release automation (separate concern)
- Crate publishing to crates.io (separate concern)
- Benchmarking
