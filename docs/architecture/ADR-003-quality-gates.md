---
id: ADR-003
kind: adr
title: Quality Gates
status: draft
date: 2026-05-17T00:00:00.000Z
authors: []
reviewers: []
tags: []
supersedes: []
superseded_by: []
depends_on: []
blocks: []
implements: []
related: []
external: []
project: vitest-linter
checksum: 36b987bd12e1f38abe04b2d19cd94123ba409ba446a761f360c41ddfb4152462
---

**Deciders:** @Jonathangadeaharder

## Context

The project must enforce code quality at PR time and merge time. Quality gates must cover formatting, linting, testing, coverage, security, and structural integrity.

## Decision

### PR Gate (`pr-gate.yml`)

Required checks (all must pass before merge):

| Check | Tool | Threshold | Runner |
|-------|------|-----------|--------|
| Format | `cargo fmt --check` | Zero diffs | ubuntu-latest / self-hosted |
| Clippy | `cargo +nightly clippy --lib -- -W clippy::all -W clippy::pedantic -W clippy::nursery` | Zero warnings | ubuntu-latest / self-hosted |
| Test | `cargo test` | All pass | ubuntu-latest / self-hosted |
| Coverage | `cargo +nightly llvm-cov --lcov --output-path coverage.lcov --fail-under-lines 90` | Lines ≥90% | ubuntu-latest / self-hosted |
| Audit | `cargo audit` | Zero vulns | ubuntu-latest / self-hosted |
| Dogfood | Run release binary on test file with intentional smells | Violations detected | ubuntu-latest / self-hosted |
| Repo Structure | `structurelint .` | All rules pass | ubuntu-latest / self-hosted |

Aggregated `required-checks` job gates the merge queue — any sub-job failure blocks merge.

### Merge Gate (`merge-gate.yml`)

Currently merged into `ci.yml` (no separate merge-gate.yml). Extends PR gate with:
- SonarCloud scanning (fetches coverage.lcov artifact)
- CodeRabbit review (automated, no human reviewer)
- PR-Agent fallback when CodeRabbit rate-limited

### Branch Protection

`main` branch requires:
- `Required Checks (PR)` status check passing
- 1 approving review (CodeRabbit or PR-Agent)
- Dismiss stale reviews on new pushes

### Self-Hosted Runner

- macOS runner configured for push events (avoids GitHub-hosted macOS minute costs).
- Used by: mutation testing, release publish step.

## Consequences

- PRs that reduce line coverage below 90% are blocked.
- PRs with clippy warnings are blocked (pedantic + nursery).
- PR-Agent requires `ENABLE_PR_AGENT=true` repo variable and self-hosted LM Studio.
- SonarCloud requires `SONAR_TOKEN` secret.
