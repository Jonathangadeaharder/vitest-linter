# eslint-plugin-vitest-linter

ESLint plugin that surfaces [vitest-linter](https://github.com/Jonathangadeaharder/vitest-linter) diagnostics through ESLint's reporter. Lets you adopt vitest-linter without changing your existing toolchain.

## Install

```bash
npm install --save-dev eslint-plugin-vitest-linter vitest-linter
```

`vitest-linter` must be installed and available on `PATH` (or in `node_modules/.bin`).

## Usage

### Flat config (ESLint 9+)

```js
// eslint.config.js
import vitestLinter from "eslint-plugin-vitest-linter";

export default [
  {
    files: ["**/*.test.ts", "**/*.spec.ts"],
    ...vitestLinter.configs["flat/recommended"],
  },
];
```

### Classic config (.eslintrc)

```json
{
  "plugins": ["vitest-linter"],
  "overrides": [
    {
      "files": ["*.test.ts", "*.test.tsx", "*.spec.ts", "*.spec.tsx"],
      "extends": ["plugin:vitest-linter/recommended"]
    }
  ]
}
```

## Per-rule severity

Override any rule severity in your ESLint config:

```json
{
  "rules": {
    "vitest-linter/timeout": "error",
    "vitest-linter/no-assertion": "error",
    "vitest-linter/empty-test": "off"
  }
}
```

## Rules

The plugin exposes one ESLint rule per vitest-linter rule. Each rule maps to the corresponding diagnostic from the standalone CLI.

| ESLint rule | Linter ID |
|---|---|
| `timeout` | VITEST-FLK-001 |
| `date-mock` | VITEST-FLK-002 |
| `network-import` | VITEST-FLK-003 |
| `fake-timers-cleanup` | VITEST-FLK-004 |
| `non-deterministic` | VITEST-FLK-005 |
| `no-assertion` | VITEST-MNT-001 |
| `multiple-expect` | VITEST-MNT-002 |
| `conditional-logic` | VITEST-MNT-003 |
| `try-catch` | VITEST-MNT-004 |
| `empty-test` | VITEST-MNT-005 |
| `nested-describe` | VITEST-STR-001 |
| `return-in-test` | VITEST-STR-002 |
| `missing-await-assertion` | VITEST-MNT-006 |
| `focused-test` | VITEST-MNT-007 |
| `missing-mock-cleanup` | VITEST-MNT-008 |
| `weak-assertion` | VITEST-MNT-009 |
| `implementation-coupled` | VITEST-MNT-010 |
| `banned-module-mock` | VITEST-DEP-001 |
| `production-singleton-import` | VITEST-DEP-002 |
| `reset-escape-hatch` | VITEST-DEP-003 |
| `mock-export-validation` | VITEST-DEP-004 |
| `valid-expect` | VITEST-VAL-001 |
| `valid-expect-in-promise` | VITEST-VAL-002 |
| `valid-describe-callback` | VITEST-VAL-003 |
| `valid-title` | VITEST-VAL-004 |
| `no-unneeded-async-expect-function` | VITEST-VAL-005 |
| `no-standalone-expect` | VITEST-NO-001 |
| `no-identical-title` | VITEST-NO-002 |
| `no-commented-out-tests` | VITEST-NO-003 |
| `no-test-prefixes` | VITEST-NO-005 |
| `no-duplicate-hooks` | VITEST-NO-006 |
| `no-import-node-test` | VITEST-NO-007 |
| `no-interpolation-in-snapshots` | VITEST-NO-008 |
| `no-large-snapshots` | VITEST-NO-009 |
| `no-done-callback` | VITEST-NO-013 |
| `no-conditional-expect` | VITEST-NO-014 |
| `prefer-to-be` | VITEST-PREF-001 |
| `prefer-to-contain` | VITEST-PREF-002 |
| `prefer-to-have-length` | VITEST-PREF-003 |
| `prefer-spy-on` | VITEST-PREF-005 |
| `prefer-called-once` | VITEST-PREF-007 |
| `prefer-hooks-on-top` | VITEST-PREF-009 |
| `prefer-hooks-in-order` | VITEST-PREF-010 |
| `prefer-todo` | VITEST-PREF-012 |
| `prefer-mock-promise-shorthand` | VITEST-PREF-013 |
| `prefer-expect-resolves` | VITEST-PREF-014 |
| `require-hook` | VITEST-REQ-001 |
| `require-top-level-describe` | VITEST-REQ-002 |
| `require-to-throw-message` | VITEST-REQ-003 |
| `consistent-test-it` | VITEST-CON-001 |
| `consistent-vitest-vi` | VITEST-CON-003 |
| `hoisted-apis-on-top` | VITEST-CON-004 |
| `pw-wait-for-timeout` | VITEST-PW-001 |
| `pw-evaluate-inner-text` | VITEST-PW-002 |
| `pw-css-id-selector` | VITEST-PW-003 |
| `pw-xpath-selector` | VITEST-PW-004 |
| `pw-locator-nth` | VITEST-PW-005 |
| `pw-arbitrary-sleep` | VITEST-PW-006 |
| `pw-text-assertion-over-role` | VITEST-PW-007 |
| `pw-test-id-over-semantic-role` | VITEST-PW-008 |
| `pw-duplicate-spec-file` | VITEST-PW-009 |
| `pw-page-dollar` | VITEST-PW-010 |
| `pw-hard-css-class-chain` | VITEST-PW-011 |
| `pw-missing-web-first-assertion` | VITEST-PW-012 |
| `pw-missing-axe-scan` | VITEST-PW-100 |

## How it works

The plugin spawns `vitest-linter --format json` for each file ESLint processes. Results are cached per file during a single lint run so the binary is only invoked once per file regardless of how many rules are enabled.

During a single ESLint run, each file is only passed to `vitest-linter` once even if multiple rules are enabled — the results are shared across all rules via an in-memory cache.

## License

MIT
