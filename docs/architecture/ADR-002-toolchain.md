---
id: ADR-002
kind: adr
title: Toolchain
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
checksum: 538c952ff7c39807f3e8fa371c91788f88c9035ac088772bd14ea53791f2f5ef
---

**Deciders:** @Jonathangadeaharder

## Context

The project needs a consistent Rust toolchain for building, linting, formatting, and auditing. Multiple toolchains are involved (stable for build/test, nightly for clippy pedantic checks and coverage).

## Decision

| Tool | Version | Config Source | Purpose |
|------|---------|---------------|---------|
| `cargo` (Rust) | edition 2021, stable | `Cargo.toml` | Build, test, bench |
| `rustfmt` | nightly via dtolnay/rust-toolchain | `rustfmt.toml` (project root) | Formatting enforcement |
| `clippy` | nightly with `clippy::all`, `pedantic`, `nursery` | `.cargo/config.toml` | Linting (zero-warnings gate) |
| `cargo-llvm-cov` | nightly with `llvm-tools-preview` | CLI args | Branch coverage (`--fail-under-lines 90`) |
| `cargo-audit` | stable | `Cargo.lock` | Dependency vulnerability scanning |
| `cargo-mutants` | stable | `.cargo/mutants.toml` | Mutation testing (score ≥75% gate) |
| `cargo-dist` | 0.28.0 | `Cargo.toml` (workspace metadata) | Binary release + npm publishing |
| `cargo-deny` | stable (planned) | `deny.toml` (planned) | License + duplicate dep checking |
| `structurelint` | Go 1.24 | `.structurelint.yml` | Repo structure linting |

CI workflows (`./.github/workflows/`):
- `ci.yml`: fmt → clippy → test → coverage → audit → dogfood → repo-structure → required-checks
- `merge-gate.yml`: CI + SonarCloud
- `release.yml`: cargo-dist multi-arch builds
- `mutants.yml`: Scheduled mutation testing (self-hosted runner)
- `pr-agent.yml`: PR-Agent for automated review (LM Studio, self-hosted)

## Consequences

- Nightly toolchain required for clippy + coverage CI jobs (stable for everything else).
- Rust cache (`Swatinem/rust-cache@v2`) essential to keep CI under 5 min.
- Self-hosted macOS runner for push events (faster than GitHub-hosted).
