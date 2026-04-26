use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use vitest_linter::run_cli;

#[derive(Parser)]
#[command(name = "vitest-linter")]
#[command(about = "Detect test smells in Vitest/TypeScript test files")]
struct Cli {
    #[arg(default_values = &["."])]
    paths: Vec<PathBuf>,

    #[arg(long, default_value = "terminal", value_parser = ["terminal", "json"])]
    format: String,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long, default_value_t = false)]
    no_color: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let has_errors = run_cli(&cli.paths, &cli.format, cli.output.as_deref(), cli.no_color)?;

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
