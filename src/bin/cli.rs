use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use snapshotter::{SnapshotConfig, Snapshotter};

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: snapshotter [dest_dir] [prefix] [max_lines] [--dry-run]");
    eprintln!("Example: snapshotter ./snapshots snapshot 1500 --dry-run");
    std::process::exit(2);
}

fn main() -> Result<()> {
    let argv: Vec<String> = env::args().collect();

    // parse flags: we accept a single flag --dry-run anywhere; remaining positionals:
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
        1 => (PathBuf::from(&positionals[0]), "snapshot".to_string(), 1500usize),
        2 => (PathBuf::from(&positionals[0]), positionals[1].clone(), 1500usize),
        3 => {
            let max_lines: usize = positionals[2]
                .parse()
                .with_context(|| format!("invalid max_lines: {}", positionals[2]))?;
            (PathBuf::from(&positionals[0]), positionals[1].clone(), max_lines)
        }
        _ => print_usage_and_exit(),
    };

    // workspace root = current dir
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
