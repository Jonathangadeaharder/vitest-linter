---
id: ADR-001
kind: adr
title: Project Architecture
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
checksum: 733c0c0e84b7d2d45ea35d0d039128ddbbd5921a24f40ecaae76b13ad35a9c40
---

**Deciders:** @Jonathangadeaharder

## Context

vitest-linter needs a fast, portable CLI tool for detecting test smells in Vitest test suites. It must be installable via `cargo install`, npm, or prebuilt binary, and integrate with GitHub Actions, VS Code, and CI pipelines via SARIF output.

## Decision

**Rust CLI tool** managed by Cargo. Architecture is a single-crate workspace with these layers:

```
src/
  main.rs          # CLI entrypoint (clap arg parsing)
  lib.rs           # Public API: run_cli(), build_sarif()
  config.rs        # .vitest-linter.toml + package.json merging
  parser.rs        # Tree-sitter TypeScript/TSX AST extraction
  engine.rs        # LintEngine: file discovery → parse → rule eval → suppression
  models.rs        # Core types: Violation, ParsedModule, TestBlock, etc.
  suppression.rs   # Inline comment suppression parsing
  rules/
    mod.rs         # Rule trait, v1_0_rules(), all_rules() registration
    flakiness.rs   # VITEST-FLK-* rules
    maintenance.rs # VITEST-MNT-* rules
    no_category.rs # VITEST-NO-* rules
    prefer.rs      # VITEST-PREF-* rules
    require.rs     # VITEST-REQ-* rules
    consistency.rs # VITEST-CON-* rules
    validation.rs  # VITEST-VAL-* rules
    dependencies.rs# VITEST-DEP-* rules
    playwright.rs  # VITEST-PW-* rules
    selector_classifier.rs # Playwright selector heuristics
tests/
  integration_tests.rs  # Integration tests via tempfile
  corpus.rs             # Corpus-based regression tests
benches/
  lint_large_corpus.rs  # Criterion benchmarks
```

Key architectural properties:
- **Two-phase pipeline:** Phase 1 = parallel file discovery + tree-sitter parsing. Phase 2 = sequential rule evaluation with shared `ModuleGraph`.
- **Rule trait:** Every rule is a struct implementing `Rule` with `id()`, `name()`, `severity()`, `category()`, `check()`, and `applies_to_runtime()`.
- **Runtime gating:** `TestRuntime` enum (Vitest/Playwright/Unknown) detected from import statements. Rules can opt out per runtime.
- **Suppression:** Inline `// vitest-linter-disable-next-line` and `// vitest-linter-disable/enable` range comments.
- **Config:** Walk-up `.vitest-linter.toml` discovery, merged with `package.json` `"vitest-linter"` key.

## Consequences

- Single binary deployment via `cargo-dist` / npm wrapper.
- Rules must be stateless (all state in `ParsedModule` + `Config` + `ModuleGraph`).
- Adding a rule requires: new struct in existing category file, registration in `mod.rs`, integration test, and (for parser-backed rules) parser field.
