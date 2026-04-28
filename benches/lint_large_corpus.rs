//! Criterion benchmark: lint a synthetic large corpus.
//!
//! The fixture is generated in-process from the benchmark configuration: a set
//! of `.test.ts` files whose count varies by benchmark, with each file's
//! length scaling with the configured number of generated test blocks. All
//! files are written to a temporary directory so the benchmark is
//! self-contained and reproducible.

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::TempDir;
use vitest_linter::engine::LintEngine;

/// Produces a single `.test.ts` file with `n_tests` test blocks.
/// Each block exercises a variety of patterns so the parser has real work to do.
fn generate_test_file(n_tests: usize) -> String {
    let mut buf = String::from(
        r#"import { describe, it, expect, vi } from 'vitest';
import axios from 'axios';

"#,
    );

    for i in 0..n_tests {
        // Mix of passing and smell-containing tests so rules fire occasionally.
        match i % 5 {
            0 => buf.push_str(&format!(
                r#"it('test_{i}_clean', () => {{
  expect({i} + 1).toBe({});
}});

"#,
                i + 1
            )),
            1 => buf.push_str(&format!(
                r#"it('test_{i}_timeout', () => {{
  setTimeout(() => {{}}, 100);
  expect(true).toBe(true);
}});

"#
            )),
            2 => buf.push_str(&format!(
                r#"it.skip('test_{i}_skipped', () => {{
  expect({i}).toBe({i});
}});

"#
            )),
            3 => buf.push_str(&format!(
                r#"describe('group_{i}', () => {{
  describe('nested_{i}', () => {{
    it('deep_{i}', () => {{
      expect(true).toBe(true);
    }});
  }});
}});

"#
            )),
            _ => buf.push_str(&format!(
                r#"it('test_{i}_cond', () => {{
  if ({i} > 0) {{
    expect({i}).toBeGreaterThan(0);
  }}
}});

"#
            )),
        }
    }

    buf
}

/// Write `n_files` test files into `dir`, each containing `tests_per_file` tests.
fn write_corpus(dir: &TempDir, n_files: usize, tests_per_file: usize) -> Vec<PathBuf> {
    let content = generate_test_file(tests_per_file);
    (0..n_files)
        .map(|i| {
            let path = dir.path().join(format!("file_{i:04}.test.ts"));
            std::fs::write(&path, &content).expect("write fixture");
            path
        })
        .collect()
}

fn bench_lint_corpus(c: &mut Criterion) {
    // Configurations: (files, tests_per_file) → total ~lines = files × tests_per_file × ~8
    let configs: &[(usize, usize)] = &[
        (10, 10),  //    ~800 lines  – warm-up
        (100, 10), //   ~8 000 lines
        (200, 50), //  ~80 000 lines – approaches 100 K
        (250, 50), // ~100 000 lines – target
    ];

    let mut group = c.benchmark_group("lint_large_corpus");

    for &(n_files, tests_per_file) in configs {
        let approx_lines = n_files * tests_per_file * 8;
        group.throughput(Throughput::Elements(approx_lines as u64));

        group.bench_with_input(
            BenchmarkId::new(
                "files",
                format!("{n_files}_files_{tests_per_file}_tests_per_file"),
            ),
            &(n_files, tests_per_file),
            |b, &(nf, tp)| {
                // Setup: write files once before benchmark iterations.
                let dir = TempDir::new().expect("tmpdir");
                let paths = write_corpus(&dir, nf, tp);
                let engine = LintEngine::new().expect("engine");

                b.iter(|| {
                    engine.lint_paths(&paths).expect("lint");
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_lint_corpus);
criterion_main!(benches);
