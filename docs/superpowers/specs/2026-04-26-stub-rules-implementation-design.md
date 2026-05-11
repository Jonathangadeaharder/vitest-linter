# Design: Implementing 8 Stub Rules via Parser Model Enrichment

**Date:** 2026-04-26  
**Status:** Approved  
**Branch:** `feat/rust-rewrite`  
**PR:** #11

## Summary

Implement 8 of 9 stub rules in pytest-linter that currently return empty `Vec<Violation>`. Drop PARAM-004 (ParametrizeNoVariationRule) as it requires semantic analysis beyond AST-level detection. Approach: enrich parser models with new fields and structs, then implement rules against the enriched data.

## Architecture

**Approach A: Enrich Parser Models** — extend `TestFunction` and `Fixture` with new fields computed during tree-sitter parsing. Rules consume the enriched models without re-parsing. This follows the existing pattern in the codebase.

## New Types

### `AssertionInfo` struct (models.rs)

```rust
pub struct AssertionInfo {
    pub is_magic: bool,
    pub is_suboptimal: bool,
    pub has_comparison: bool,
    pub expression_text: String,
    pub line: usize,
}
```

- `is_magic`: Bare literal (`True`, `False`, `0`, `1`) or bare identifier without comparison
- `is_suboptimal`: `assert len(x) == N`, `assert type(x) == Y`, `assert x is not None` (when `assert x` suffices)
- `has_comparison`: Contains `==`, `!=`, `<`, `>`, `<=`, `>=`, `in`, `not in`, `is`, `is not`
- `expression_text`: Raw text of the assertion expression for violation messages
- `line`: Line number within the function

## New Fields on `TestFunction`

| Field | Type | Rules Served |
|-------|------|-------------|
| `assertions` | `Vec<AssertionInfo>` | MNT-002, MNT-003 |
| `parametrize_values` | `Vec<Vec<String>>` | PARAM-002 |
| `uses_cwd_dependency` | `bool` | FLK-004 |
| `uses_pytest_raises` | `bool` | DBC-001 |
| `mutates_fixture_deps` | `Vec<String>` | FIX-007, XDIST-001 |

## Parser Enhancements (parser.rs)

### 1. Assertion Analysis

Walk all `assert_statement` nodes in function body:

- **Magic detection**: Child expression is `true`/`false` literal, integer `0`/`1`, or bare identifier without any comparison operator child
- **Suboptimal detection**: Expression is a `comparison_operator` where:
  - One side is a `call` node with function `len` or `type`
  - Or one side is `not` applied to `None` (could simplify to `assert x`)
- **Comparison detection**: `assert_statement` has a `comparison_operator` child node

### 2. Parametrize Value Extraction

In `count_parametrize_args_ast()`, additionally extract string representations of each element in the parametrize values list. Store as `Vec<Vec<String>>` where each inner Vec is one parametrize decorator's values.

### 3. CWD Dependency Detection

Walk function body `call` nodes for:
- `os.getcwd()`, `os.chdir()` — attribute call on `os` module
- `Path.cwd()`, `Path(".")` — pathlib usage

### 4. pytest.raises Detection

Walk function body `call` nodes for `pytest.raises(...)` — attribute `raises` on identifier `pytest`.

### 5. Fixture Mutation Detection

Walk function body for:
- `call` nodes where function is an attribute (`append`, `extend`, `remove`, `pop`, `clear`, `update`) on an identifier matching a fixture dep name
- `assignment` nodes where target includes subscript access (`subscript`) on a fixture dep name
- `delete` nodes targeting fixture dep names

## Rule Implementations

### Tier 1: Trivial (existing data or simple flag)

#### FIX-009: FixtureOverlyBroadScopeRule
- **Condition:** `fixture.scope >= Module` AND `!fixture.has_yield AND !fixture.has_db_commit AND !fixture.has_db_rollback AND !fixture.uses_file_io`
- **Message:** "Fixture '{name}' has scope '{scope}' but no expensive setup — consider using function scope for better isolation"
- **Suggestion:** "Change fixture scope to 'function'"

#### FLK-004: CwdDependencyRule
- **Condition:** `test.uses_cwd_dependency == true`
- **Message:** "Test '{name}' depends on the current working directory"
- **Suggestion:** "Use absolute paths or tmp_path fixture instead"

#### DBC-001: NoContractHintRule
- **Condition:** `test.has_assertions AND !test.uses_pytest_raises AND !test.has_try_except AND !test.is_parametrized`
- **Message:** "Test '{name}' only tests the happy path — consider adding error/edge case coverage"
- **Suggestion:** "Add tests for error conditions using pytest.raises"

### Tier 2: Simple (new flag consumed directly)

#### FIX-007: FixtureMutationRule
- **Condition:** `!test.mutates_fixture_deps.is_empty()`
- **Cross-reference:** Only flag if the mutated fixture has `returns_mutable == true` (skip immutable fixtures)
- **Message:** "Test '{name}' mutates fixture '{dep}' which may affect other tests"
- **Suggestion:** "Create a fresh copy of the fixture value before modifying it"

#### MNT-002: MagicAssertRule
- **Condition:** Any `assertion.is_magic == true` in `test.assertions`
- **Message:** "Magic assertion at line {line}: '{expression}' — this always passes/fails"
- **Suggestion:** "Replace with a meaningful comparison"

#### MNT-003: SuboptimalAssertRule
- **Condition:** Any `assertion.is_suboptimal == true` in `test.assertions`
- **Message:** "Suboptimal assertion at line {line}: '{expression}'"
- **Suggestion:** "Use a more direct assertion pattern"

### Tier 3: Moderate (cross-reference or value extraction)

#### XDIST-001: XdistSharedStateRule
- **Condition:** For each session-scoped fixture with `returns_mutable`, check if any test across all modules mutates it via `mutates_fixture_deps`
- **Message:** "Session-scoped fixture '{name}' returns mutable state that is modified by tests — unsafe for xdist parallel execution"
- **Suggestion:** "Use function scope or return immutable values"

#### PARAM-002: ParametrizeDuplicateRule
- **Condition:** Any `parametrize_values` entry with duplicate strings
- **Message:** "Parametrize in test '{name}' has duplicate values: {duplicates}"
- **Suggestion:** "Remove duplicate parametrize values"

## Implementation Order

1. **models.rs** — Add `AssertionInfo` struct and 5 new fields to `TestFunction`
2. **parser.rs** — Implement 5 parser enhancements (assertion analysis, parametrize values, CWD, pytest.raises, fixture mutation)
3. **FIX-009** — Trivial, validates existing data path
4. **FLK-004** — Validates new parser flag
5. **DBC-001** — Validates new parser flag
6. **FIX-007** — Validates mutation detection
7. **MNT-002 + MNT-003** — Validates AssertionInfo (implement together)
8. **XDIST-001** — Cross-references FIX-007 data
9. **PARAM-002** — Validates parametrize value extraction

## Testing Strategy

- **Parser unit tests**: Each new detection function gets positive and negative test cases
- **Rule integration tests**: Each rule gets 2-4 tests (triggers, doesn't trigger, edge cases)
- **Coverage target**: ≥90% branch coverage on new code
- All 195 existing tests must continue passing

## Dropped Rule: PARAM-004 (ParametrizeNoVariationRule)

**Why dropped:** Requires semantic analysis — determining whether different parametrize values exercise different code paths. This is essentially lightweight symbolic execution, which is:
- Unreliable at the AST level (high false positive rate)
- Architecturally different from all other rules (needs data flow analysis)
- Better suited for a dynamic analysis tool or coverage-based tooling

**Future directions for PARAM-004:**
1. **Coverage-guided heuristic**: If the project integrates with coverage.py data, compare branch coverage across parametrize cases — identical coverage indicates no variation
2. **Type-based heuristic**: If all parametrize values are the same type with the same truthiness/length sign, flag as potentially non-varying (simple version)
3. **Configurable opt-in**: Make it an optional rule behind a flag, acknowledging the false positive risk
4. **LLM-assisted analysis**: For CI pipelines, an LLM could analyze test intent vs. parametrize values (heavy, but accurate)

## Scope

- **In scope:** 8 rule implementations, parser enrichment, tests
- **Out of scope:** PARAM-004, vitest-linter changes, CLI changes, README updates (will be done separately after implementation)
