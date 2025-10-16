// src/bin/cli.rs
#[cfg(not(feature = "completion"))]
use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use snapshotter::{SnapshotConfig, Snapshotter};

#[cfg(feature = "completion")]
use clap::{CommandFactory, Parser, Subcommand};

#[cfg(feature = "completion")]
use clap_complete::{generate, shells::Zsh};

#[cfg(feature = "completion")]
/// CLI when compiled with the `completion` feature: use clap derive to enable completions generation.
#[derive(Debug, Parser)]
#[command(name = "snapshotter", about = "Create snapshots of a Rust project")]
struct Cli {
    /// Destination dir (default: .)
    dest: Option<PathBuf>,

    /// Prefix for part files (default: snapshot)
    prefix: Option<String>,

    /// Max lines per part (default: 1500)
    max_lines: Option<usize>,

    /// dry-run mode: don't write files
    #[arg(long = "dry-run", default_value_t = false)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "completion")]
#[derive(Debug, Subcommand)]
enum Commands {
    /// Generate shell completion script (zsh supported). Prints to stdout.
    Completions { shell: String },
}
#[cfg(not(feature = "completion"))]
fn print_usage_and_exit() -> ! {
    eprintln!("Usage: snapshotter [dest_dir] [prefix] [max_lines] [--dry-run]");
    eprintln!("Example: snapshotter ./snapshots snapshot 1500 --dry-run");
    std::process::exit(2);
}

fn main() -> Result<()> {
    // If compiled with `completion` feature, use clap for argument parsing and support `completions`.
    #[cfg(feature = "completion")]
    {
        let cli = Cli::parse();

        // handle completions subcommand
        if let Some(Commands::Completions { shell }) = cli.command {
            // only zsh supported for now (others possible)
            if shell.to_lowercase() == "zsh" {
                let mut cmd = Cli::command();
                let mut stdout = std::io::stdout();
                generate(Zsh, &mut cmd, "snapshotter", &mut stdout);
                return Ok(());
            } else {
                eprintln!("Supported shells: zsh");
                std::process::exit(2);
            }
        }

        let dest_dir = cli.dest.unwrap_or_else(|| PathBuf::from("."));
        let prefix = cli.prefix.unwrap_or_else(|| "snapshot".to_string());
        let max_lines = cli.max_lines.unwrap_or(1500usize);
        let dry_run = cli.dry_run;

        let cfg = SnapshotConfig::new(".", prefix, max_lines)
            .context("failed to build config")?
            .with_dest(dest_dir)
            .with_dry_run(dry_run);

        let snap = Snapshotter::new(cfg);
        let res = snap.run().context("snapshot failed")?;
        println!(
            "Snapshot terminé: {} parts, {} fichiers, {} lignes{}",
            res.parts.len(),
            res.total_files,
            res.total_lines,
            if dry_run { " (dry-run)" } else { "" }
        );
        Ok(())
    }

    // Fallback: original simple parsing when `completion` feature is NOT enabled.
    #[cfg(not(feature = "completion"))]
    {
        let argv: Vec<String> = env::args().collect();
        let mut dry_run = false;
        let mut positionals: Vec<String> = Vec::new();
        for arg in argv.iter().skip(1) {
            if arg == "--dry-run" {
                dry_run = true;
            } else if arg == "--help" || arg == "-h" {
                print_usage_and_exit();
            } else {
                positionals.push(arg.clone());
            }
        }

        let (dest_dir, prefix, max_lines) = match positionals.len() {
            0 => (PathBuf::from("."), "snapshot".to_string(), 1500usize),
            1 => (
                PathBuf::from(&positionals[0]),
                "snapshot".to_string(),
                1500usize,
            ),
            2 => (
                PathBuf::from(&positionals[0]),
                positionals[1].clone(),
                1500usize,
            ),
            3 => {
                let max_lines: usize = positionals[2]
                    .parse()
                    .with_context(|| format!("invalid max_lines: {}", positionals[2]))?;
                (
                    PathBuf::from(&positionals[0]),
                    positionals[1].clone(),
                    max_lines,
                )
            }
            _ => print_usage_and_exit(),
        };

        let cfg = SnapshotConfig::new(".", prefix, max_lines)
            .context("failed to build config")?
            .with_dest(dest_dir)
            .with_dry_run(dry_run);

        let snap = Snapshotter::new(cfg);
        let res = snap.run().context("snapshot failed")?;

        println!(
            "Snapshot terminé: {} parts, {} fichiers, {} lignes{}",
            res.parts.len(),
            res.total_files,
            res.total_lines,
            if dry_run { " (dry-run)" } else { "" }
        );

        Ok(())
    }
}
