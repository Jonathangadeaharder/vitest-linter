# Spec: Async Assertion Support in Vitest Linter

Enhance the linter to detect unawaited async assertions in Vitest tests.

## Context
In Vitest, assertions using `.resolves` or `.rejects` return a Promise that must be awaited. Failing to do so results in the test finishing before the assertion executes, potentially leading to silent failures where broken code passes.

## Proposed Changes

### 1. Models Update (`src/models.rs`)
- Add `unawaited_async_assertions` count to `TestBlock` struct.

### 2. Parser Enhancements (`src/parser.rs`)
- Update `Analysis` struct to track `unawaited_async_assertions`.
- Add helper method `is_awaited(node: Node) -> bool` to detect `await_expression` parents.
- Update `walk_body` to:
    - Identify `expect` calls.
    - Check for async matchers (`.resolves`, `.rejects`).
    - Verify if they are awaited using the helper.
    - Increment the count in `Analysis`.

### 3. Rule Implementation (`src/rules/maintenance.rs`)
- Create `MissingAwaitAssertionRule` (ID: `VITEST-MNT-006`).
- Rule will generate violations for any test with unawaited async assertions.

### 4. Rule Registration (`src/rules/mod.rs`)
- Add the new rule to the global rule list.

## Testing Plan
- Create an integration test with a missing `await` on a `.resolves` assertion.
- Verify the linter reports the violation.
- Run existing tests to ensure no regressions.
