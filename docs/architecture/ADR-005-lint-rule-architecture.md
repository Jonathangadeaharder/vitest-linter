---
id: ADR-005
kind: adr
title: Lint Rule Architecture
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
checksum: b214362d6987fe751115cbc3df4fc0ce295f5d5de5fab18f38abed0515be943e
---

**Deciders:** @Jonathangadeaharder

## Context

vitest-linter must support 66 lint rules (8 stable + 58 unstable) across 6 categories. Rules need to analyze tree-sitter AST data, respect config overrides, support inline suppression, and gate by test runtime. The architecture must make adding new rules mechanical.

## Decision

### Rule Trait

Every rule implements this trait:

```rust
pub trait Rule {
    fn id(&self) -> &'static str;              // e.g. "VITEST-FLK-001"
    fn name(&self) -> &'static str;            // e.g. "TimeoutRule"
    fn severity(&self) -> Severity;            // Error | Warning | Info
    fn category(&self) -> Category;            // Flakiness | Maintenance | ...
    fn check(&self, module: &ParsedModule, ctx: &LintContext<'_>, graph: &ModuleGraph) -> Vec<Violation>;
    fn applies_to_runtime(&self, runtime: TestRuntime) -> bool;  // default: !Playwright
}
```

### Rule Registry

Two registration functions in `src/rules/mod.rs`:

| Function | Purpose | Rules |
|----------|---------|-------|
| `v1_0_rules()` | Active by default (no flag) | 8 community-hit rules |
| `all_rules()` | Behind `--unstable-rules` | All 66 rules |

The engine checks `config.rules.is_disabled(rule_id)` before calling `check()`.

### Categories

The `Category` enum has 6 variants. Rules are assigned categories by what they
detect, not by their ID prefix — e.g. `VITEST-NO-001` has `Structure` category,
not a `No-*` category.

```
Flakiness (5)     - Non-deterministic test behavior (VITEST-FLK-*)
Maintenance (18)  - Code quality and test hygiene (VITEST-MNT-* + several NO/PREF/CON rules)
Structure (9)     - Test organization and nesting (VITEST-STR-* + several NO/PREF/REQ/CON rules)
Dependencies (7)  - Mock and import isolation (VITEST-DEP-* + NO-007, PREF-005, CON-003)
Validation (14)   - Correct test API usage (VITEST-VAL-* + several NO/PREF/REQ rules)
Playwright (13)   - Playwright E2E best practices (VITEST-PW-*)
```

Total: 66 (8 stable + 58 unstable). 66 rules in `all_rules()` test.

### AST-Based Analysis

Rules do NOT re-parse source files. They consume `ParsedModule` which is populated by tree-sitter traversal in `TsParser`. Key data structures:

- `TestBlock` — name, file_path, line, assertion_count, flags (uses_settimeout, has_conditional_logic, etc.)
- `DescribeBlock` — name, depth, is_async, title metadata
- `ParsedModule` — test_blocks, describe_blocks, imports, vi_mocks, hook_calls, runtime, playwright module data
- `ModuleGraph` — resolved import edges for cross-file analysis (DEP rules)
- `PlaywrightModule` — tracked calls, locator chains, axe usage

### Rule Evaluation Flow

```
LintEngine::lint_paths()
  └─ discover_files()                # WalkDir → test file filter
  └─ parallel parse (rayon)          # TsParser::parse_file() per file
  └─ ModuleGraph::new()              # Build import graph
  └─ Group modules by config root
  └─ For each (config, group):
       └─ Create LintContext { config, all_modules }
       └─ For each rule:
            └─ skip if config.rules.is_disabled()
            └─ skip if !rule.applies_to_runtime(module.runtime)
            └─ rule.check(module, ctx, graph)
            └─ apply severity override from config
            └─ filter via SuppressionMap
  └─ Sort violations by (file_path, line)
```

## Consequences

- Adding a rule requires 5 mechanical steps: struct + impl, check() logic, registration in mod.rs, integration tests, field in models.rs if parser change needed.
- Cross-file rules (DEP rules) access `ModuleGraph` and `LintContext.all_modules`.
- Runtime gating prevents Playwright rules from firing on Vitest files and vice versa.
- Suppression is post-hoc (filter step), not during parsing — ensures suppression comments don't affect other rules.
