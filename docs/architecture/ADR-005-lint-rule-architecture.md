# ADR-005: Lint Rule Architecture

**Status:** Accepted  
**Date:** 2026-05-17  
**Deciders:** @Jonathangadeaharder

## Context

vitest-linter must support 65 lint rules (8 stable + 57 unstable) across 7 categories. Rules need to analyze tree-sitter AST data, respect config overrides, support inline suppression, and gate by test runtime. The architecture must make adding new rules mechanical.

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
| `all_rules()` | Behind `--unstable-rules` | All 65 rules |

The engine checks `config.rules.is_disabled(rule_id)` before calling `check()`.

### Categories

```
Flakiness (5)    - VITEST-FLK-*    → Non-deterministic test behavior
Maintenance (18) - VITEST-MNT-*   → Code quality and test hygiene
Structure (9)    - VITEST-STR-*   → Test organization and nesting
Dependencies (7) - VITEST-DEP-*    → Mock and import isolation
Validation (5)   - VITEST-VAL-*   → Correct test API usage
Playwright (13)  - VITEST-PW-*    → Playwright E2E best practices
No-* (10)        - VITEST-NO-*    → Banned patterns
Prefer-* (10)    - VITEST-PREF-*  → Idiomatic matchers
Require-* (3)    - VITEST-REQ-*   → Required patterns
Consistency (3)  - VITEST-CON-*   → Consistent style
```

Total: 66 (8 stable + 58 unstable). Duplicate: 66 rules in `all_rules()` test.

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
