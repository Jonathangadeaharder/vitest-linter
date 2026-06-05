---
id: TDD-VTLN
kind: tdd
title: vitest-linter
description: >-
  Rust CLI that lints Vitest/TypeScript test files using tree-sitter AST
  analysis
status: draft
date: 2026-05-17T00:00:00.000Z
authors: []
reviewers: []
risk_level: low
scope_type: project
tags:
  - rust
  - linter
  - vitest
  - typescript
  - cli
related: []
checksum: a315273f9c4c04e2ea67945e0b823615652c6e2357289b582b18fabf1602bd4d
---

## Executive Summary

vitest-linter v1.0.0 is a well-architected Rust CLI using tree-sitter for AST-level test smell detection across 66 rules (8 stable, 58 unstable). Multi-channel distribution (cargo-dist for 5 platforms, npm, VSCode extension, GitHub Action, ESLint plugin, sigstore signing) is excellent. The 8:58 stable-to-unstable ratio is the main maturity signal -- unstable rules may change behavior between releases. No coverage threshold in CI and no cargo audit. Recommendation: add dependency scanning, define rule lifecycle policy, and enforce coverage.

## Scope

Assessed: 66 rules across 6 categories (Flakiness, Maintenance, Structure, Dependencies, Validation, Playwright), tree-sitter AST engine, CLI + npm + VSCode + GitHub Action + ESLint plugin distribution channels, cargo-dist build with sigstore, criterion benchmarks, cargo-mutants mutation testing, SonarCloud, 5 ADRs. Excluded: IDE integration beyond VSCode, Docker distribution, fuzz testing.

## Architecture

Single Rust crate with modular rule engine -- rules organized by ID prefix across 9 source modules, with 6 `Category` enum variants for classification. tree-sitter produces CST/AST for pattern matching. Comment-based suppression system (`// vitest-linter-disable-next-line` / `// vitest-linter-disable` / `// vitest-linter-enable`) with mutation testing evidence. Multi-format output: terminal (colored), JSON, SARIF. Zero-config defaults with optional `vitest-linter.toml`. Multi-channel distribution via cargo-dist covering 5 target triples (macOS aarch64/x86_64, Linux aarch64/x86_64, Windows x86_64). npm and VSCode wrapper packages download the prebuilt binary.

## Tech Stack

Rust 2021 edition, tree-sitter 0.24 + tree-sitter-typescript 0.23, clap 4 (derive), serde + serde_json, anyhow, colored, walkdir, toml 0.8, globset, rayon. Build: cargo-dist 0.31, sigstore signing, thin LTO. Benchmarks: criterion with HTML reports. Mutation: cargo-mutants. CI: format + clippy (nightly, pedantic+nursery) + test + coverage. Quality: SonarCloud.

## Code Quality

clippy at pedantic + nursery on nightly -- unusually strict for any Rust project. rust-toolchain pins nightly for clippy, stable for build -- good balance. Swatinem/rust-cache for CI. SonarCloud with lcov report path. No explicit MSRV. 57 unstable rules likely have lower test coverage than the 8 stable rules. No per-rule documentation files (unlike pytest-linter). Rule modules could grow large as rule count increases.

## Security

sigstore signing on release artifacts provides binary integrity verification. Fully offline tool (no network calls, no telemetry, no update checks). SARIF output integrates with GitHub code scanning. No cargo audit in CI. No CodeQL scan on the Rust codebase. npm package downloads prebuilt binary over HTTPS. VSCode extension relies on binary PATH discovery.

## Scalability & Performance

criterion benchmarks for large corpus linting. tree-sitter parsing scales linearly with file size. rayon provides parallel file system traversal. Single-threaded rule evaluation per file. No fuzz testing despite tree-sitter parser (common gap for AST tools).

## Operations & DevOps

5 workflows: ci.yml, release.yml, vitest-linter-action.yml, mutants.yml, pr-agent.yml. Full release automation via cargo-dist with sigstore. No coverage threshold gate -- lint/test can pass without minimum coverage. No separate PR vs merge gate. Release is tag-triggered only -- no canary/nightly channel.

## Dependencies & Third-Party Risk

11 production deps -- lightweight for an AST-based linter. tree-sitter ecosystem is well-maintained. npm package wrapper uses postinstall binary download. VSCode extension assumes binary on PATH. ESLint plugin is a compatibility layer. No Docker image for CI environments.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| 58 unstable rules with no graduation criteria | Medium | Medium | Document rule lifecycle policy |
| No coverage threshold in CI | Medium | Medium | Add cargo-llvm-cov gate with minimum threshold |
| No cargo audit in CI | Medium | Medium | Add cargo audit step |
| npm postinstall may fail in restricted envs | Low | Medium | Document alternative install methods |
| No fuzz testing on tree-sitter parser | Low | Low | Add cargo-fuzz for query robustness |
| No documentation per rule | Low | Low | Add per-rule docs for stable set |

## Recommendations

1. Add cargo audit to CI for dependency vulnerability scanning within 1 month (P0).
2. Implement coverage threshold in CI via cargo-llvm-cov with enforceable minimum within 1 month (P1).
3. Define and document the unstable-to-stable rule graduation process within 1 quarter (P1).
4. Add proptest or cargo-fuzz for tree-sitter parser robustness within 1 quarter (P1).
5. Add prebuilt Docker image for containerized CI environments within 1 quarter (P2).
6. Document per-rule policies for the 8 stable rules within 1 quarter (P2).
