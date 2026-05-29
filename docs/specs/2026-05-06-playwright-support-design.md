---
checksum: 5cd639928627d157044e6b3b2ca6597567a1aea6799c1c85bb8182346f30c457
---
# Design: Playwright E2E Test Linting + Rule Hardening

**Date:** 2026-05-06  
**Status:** Approved  
**Branch:** `feat/playwright-support`

## Summary

Add Playwright E2E test detection and 13 new `VITEST-PW-*` rules. Harden 7 existing rules based on real-world test smells from the Vidiom audit (issues A1-A7).

## TestRuntime Detection

`ParsedModule.runtime` is set during parsing based on imports:

| Import Source | Runtime |
|---------------|---------|
| `@playwright/test` | `Playwright` |
| `vitest` / any `vitest/...` | `Vitest` |
| none of the above | `Unknown` |

`Rule::applies_to_runtime(TestRuntime)` gates which rules fire. Default: `runtime != Playwright` (Vitest/Unknown only). Override to `true` for rules that apply to both (NoAssertion, FocusedTest, EmptyTest, NoIdenticalTitle).

## Playwright Module Data

`PlaywrightModule` struct tracked during parsing:

```rust
pub struct PlaywrightModule {
    pub calls: Vec<PlaywrightCall>,          // Tracked call expressions
    pub locator_chains: Vec<LocatorChain>,   // getByRole, locator, etc.
    pub evaluate_inner_text: Vec<usize>,     // Lines with evaluate + innerText
    pub uses_axe: bool,                      // injectAxe/checkA11y/AxeBuilder
}
```

## Playwright Rules (13)

### Easy AST-Match (PR 2)

| Rule ID | Name | Detection |
|---------|------|-----------|
| VITEST-PW-001 | PwWaitForTimeoutRule | `page.waitForTimeout(...)` calls |
| VITEST-PW-003 | PwXPathSelectorRule | `xpath=`, `//` strings, `.xpath()` calls |
| VITEST-PW-004 | PwLocatorNthRule | `.nth(N)` positional locators |
| VITEST-PW-005 | PwPageDollarRule | `page.$()` / `page.$$()` raw queries |
| VITEST-PW-010 | PwArbitrarySleepRule | `await new Promise(r => setTimeout(r, N))` |

### Selector-Based (PR 3)

| Rule ID | Name | Detection |
|---------|------|-----------|
| VITEST-PW-002 | PwCssIdSelectorRule | CSS `#id` selectors |
| VITEST-PW-006 | PwEvaluateInnerTextRule | `page.evaluate(() => document.body.innerText)` |
| VITEST-PW-011 | PwHardCssClassChainRule | `.foo > .bar` descendant/child chains |

`classify_selector` helper function with 30+ test fixtures classifies selector strings:

- CSS ID (`#foo`, `input#name`)
- CSS class (`.foo`, `div.bar`)
- CSS attribute (`[data-testid=x]`, `[type=submit]`)
- XPath (`//div`, `xpath=//button`)
- Text (`text=Login`, `"Login"`)
- Role (`getByRole`, role locator)
- TestId (`getByTestId`, `[data-testid=...]`)
- Semver (`>>`, `>`, `+` combinators)

### File-Graph (PR 4)

| Rule ID | Name | Detection |
|---------|------|-----------|
| VITEST-PW-009 | PwDuplicateSpecFileRule | Duplicate spec files (e.g., `foo.spec.ts` + `foo.spec 2.ts`) |

### Heuristic (PR 5)

| Rule ID | Name | Detection |
|---------|------|-----------|
| VITEST-PW-007 | PwTextAssertionOverRoleRule | Text-based `getByText`/`getByLabel` where `getByRole` works |
| VITEST-PW-008 | PwTestIdOverSemanticRoleRule | `getByTestId` where semantic role locator exists |
| VITEST-PW-012 | PwMissingWebFirstAssertionRule | Assertions without web-first matchers (`toHaveText`, `toBeVisible`) |
| VITEST-PW-100 | PwMissingAxeScanRule | No `injectAxe`/`checkA11y`/`AxeBuilder` in Playwright files |

## Rule Hardening (A1-A7)

| Issue | Rule | Fix |
|-------|------|-----|
| A1 | VITEST-MNT-008 MissingMockCleanupRule | Also detect `global.X = vi.fn()` and `vi.stubGlobal(...)` as stubs requiring cleanup |
| A2 | VITEST-DEP-001 BannedModuleMockRule | Add path-segment matching, integration-context boost, config knobs |
| A3 | VITEST-MNT-009 WeakAssertionRule | Add extended matchers (`toBeCalled`, `toHaveReturned`, `toHaveProperty`) |
| A4 | VITEST-MNT-010 ImplementationCoupledRule | Add `data-testid` negative-presence detection |
| A5 | VITEST-FLK-001 TimeoutRule | Detect `Promise`-wrapped `setTimeout` (`new Promise(r => setTimeout(r, N))`) |
| A6 | VITEST-MNT-007 FocusedTestRule | Fire on Playwright `test.only` / `describe.only` |
| A7 | VITEST-PREF-003 PreferToHaveLengthRule | Extended patterns (`.length === 0`, `toEqual(length)` vs `toHaveLength`) |

## Testing Strategy

- Each PW rule: 1 positive + 1 negative integration test
- `classify_selector`: 30+ unit test fixtures (one per selector class)
- Hardenings: extend existing integration tests with new edge cases
- Dogfood CI job: verifies linter detects all smells on known-bad fixtures
