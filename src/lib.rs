pub mod engine;
pub mod models;
pub mod parser;
pub mod rules;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use colored::Colorize;

use engine::LintEngine;
use models::Severity;

#[allow(clippy::missing_errors_doc)]
pub fn run_cli(
    paths: &[PathBuf],
    format: &str,
    output: Option<&Path>,
    no_color: bool,
) -> Result<bool> {
    if no_color {
        colored::control::set_override(false);
    }

    let engine = LintEngine::new()?;
    let violations = engine.lint_paths(paths)?;

    if format == "json" {
        let json = serde_json::to_string_pretty(&violations)?;
        match output {
            Some(path) => fs::write(path, json)?,
            None => println!("{json}"),
        }
    } else {
        let mut out: Box<dyn Write> = match output {
            Some(path) => Box::new(fs::File::create(path)?),
            None => Box::new(std::io::stdout()),
        };

        if violations.is_empty() {
            writeln!(out, "{} No test smells detected.", "\u{2713}".green())?;
        } else {
            for v in &violations {
                let severity_str = match v.severity {
                    Severity::Error => "Error".red().bold().to_string(),
                    Severity::Warning => "Warning".yellow().bold().to_string(),
                    Severity::Info => "Info".blue().bold().to_string(),
                };
                writeln!(
                    out,
                    "{}: {} in {}:{}",
                    severity_str,
                    v.rule_id.cyan(),
                    v.file_path.display().to_string().white(),
                    v.line
                )?;
                writeln!(out, "  {}", v.message)?;
                if let Some(ref suggestion) = v.suggestion {
                    writeln!(out, "  {} {}", "Suggestion:".dimmed(), suggestion.dimmed())?;
                }
                writeln!(out)?;
            }

            let errors = violations
                .iter()
                .filter(|v| v.severity == Severity::Error)
                .count();
            let warnings = violations
                .iter()
                .filter(|v| v.severity == Severity::Warning)
                .count();
            let infos = violations
                .iter()
                .filter(|v| v.severity == Severity::Info)
                .count();

            writeln!(
                out,
                "Found {} violation(s): {} error(s), {} warning(s), {} info",
                violations.len(),
                errors,
                warnings,
                infos
            )?;
        }
    }

    let has_errors = violations.iter().any(|v| v.severity == Severity::Error);
    Ok(has_errors)
}
