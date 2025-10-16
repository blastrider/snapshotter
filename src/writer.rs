use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;

use crate::collector::collect_files;
use crate::config::SnapshotConfig;
use crate::error::SnapshotError;
use crate::types::{PartSummary, SnapshotResult};
use crate::utils::path_relative;

/// Snapshotter : objet exécutable (stateless).
pub struct Snapshotter {
    cfg: SnapshotConfig,
}

impl Snapshotter {
    /// Constructeur principal.
    pub fn new(cfg: SnapshotConfig) -> Self {
        Self { cfg }
    }

    /// Exécute le snapshot (bloquant).
    ///
    /// Streaming: on compte d'abord les lignes (lecture en flux), puis on écrit le contenu
    /// en flux (sans charger tout le fichier en mémoire).
    pub fn run(&self) -> Result<SnapshotResult, SnapshotError> {
        let files = collect_files(&self.cfg.workspace_root)?;

        if files.is_empty() {
            return Err(SnapshotError::NoFiles);
        }

        if !self.cfg.dry_run {
            fs::create_dir_all(&self.cfg.dest_dir)?;
        }

        let mut part_idx: usize = 1;
        let mut out_path = self
            .cfg
            .dest_dir
            .join(format!("{}_part{}.txt", self.cfg.prefix, part_idx));
        let mut out_file: Option<BufWriter<File>> = if self.cfg.dry_run {
            None
        } else {
            Some(BufWriter::new(File::create(&out_path)?))
        };

        let mut line_count_in_part: usize = 0;
        let mut files_in_part: usize = 0;

        let mut parts: Vec<PartSummary> = Vec::new();
        let mut total_files = 0usize;
        let mut total_lines = 0usize;

        for file in files {
            let rel_path = path_relative(&file, &self.cfg.workspace_root);
            let header = format!("# {}\n", rel_path.display());

            // compter les lignes du fichier (lecture en flux) sans charger tout en mémoire
            let file_line_count = count_file_lines(&file)?;
            // block_lines = header (1) + file lines + 2 blank lines (comme le script original)
            let block_lines = 1usize + file_line_count + 2usize;

            // Trop grand — isoler
            if block_lines >= self.cfg.max_lines {
                // finalize current part if it has content
                if line_count_in_part > 0 || files_in_part > 0 {
                    parts.push(PartSummary {
                        part_index: part_idx,
                        file_count: files_in_part,
                        line_count: line_count_in_part,
                        filename: out_path.clone(),
                    });
                    total_files += files_in_part;
                    total_lines += line_count_in_part;
                    // new part
                    part_idx += 1;
                    out_path = self
                        .cfg
                        .dest_dir
                        .join(format!("{}_part{}.txt", self.cfg.prefix, part_idx));
                    out_file = if self.cfg.dry_run {
                        None
                    } else {
                        Some(BufWriter::new(File::create(&out_path)?))
                    };
                }

                // write the big file into its own part file (or skip actual write in dry-run)
                if !self.cfg.dry_run {
                    if let Some(w) = out_file.as_mut() {
                        write_block_from_path(w, &header, &file)?;
                        w.flush()?;
                    }
                }
                // finalize that part
                parts.push(PartSummary {
                    part_index: part_idx,
                    file_count: 1,
                    line_count: block_lines,
                    filename: out_path.clone(),
                });
                total_files += 1;
                total_lines += block_lines;

                // new part after isolated file
                part_idx += 1;
                out_path = self
                    .cfg
                    .dest_dir
                    .join(format!("{}_part{}.txt", self.cfg.prefix, part_idx));
                out_file = if self.cfg.dry_run {
                    None
                } else {
                    Some(BufWriter::new(File::create(&out_path)?))
                };
                line_count_in_part = 0;
                files_in_part = 0;
                continue;
            }

            // if adding would overflow, start a new part first
            if line_count_in_part > 0 && (line_count_in_part + block_lines) > self.cfg.max_lines {
                parts.push(PartSummary {
                    part_index: part_idx,
                    file_count: files_in_part,
                    line_count: line_count_in_part,
                    filename: out_path.clone(),
                });
                total_files += files_in_part;
                total_lines += line_count_in_part;

                part_idx += 1;
                out_path = self
                    .cfg
                    .dest_dir
                    .join(format!("{}_part{}.txt", self.cfg.prefix, part_idx));
                out_file = if self.cfg.dry_run {
                    None
                } else {
                    Some(BufWriter::new(File::create(&out_path)?))
                };
                line_count_in_part = 0;
                files_in_part = 0;
            }

            // append block — en flux
            if !self.cfg.dry_run {
                if let Some(w) = out_file.as_mut() {
                    write_block_from_path(w, &header, &file)?;
                }
            }

            line_count_in_part += block_lines;
            files_in_part += 1;
        }

        // finalize last part if non-empty
        if line_count_in_part > 0 || files_in_part > 0 {
            parts.push(PartSummary {
                part_index: part_idx,
                file_count: files_in_part,
                line_count: line_count_in_part,
                filename: out_path.clone(),
            });
            total_files += files_in_part;
            total_lines += line_count_in_part;
        }

        // write summary (création du fichier sauf en dry-run)
        if !self.cfg.dry_run {
            let summary_path = self
                .cfg
                .dest_dir
                .join(format!("{}_summary.txt", self.cfg.prefix));
            let mut sum_file = BufWriter::new(File::create(&summary_path)?);
            writeln!(sum_file, "==== Résumé du snapshot ====")?;
            for p in &parts {
                writeln!(
                    sum_file,
                    "Part {}: {} fichiers, {} lignes -> {}",
                    p.part_index,
                    p.file_count,
                    p.line_count,
                    p.filename
                        .file_name()
                        .and_then(OsStr::to_str)
                        .unwrap_or_default()
                )?;
            }
            writeln!(sum_file, "TOTAL: {} fichiers, {} lignes", total_files, total_lines)?;
            sum_file.flush()?;
        }

        Ok(SnapshotResult {
            parts,
            total_files,
            total_lines,
        })
    }
}

/// Compte le nombre de lignes dans un fichier (lecture en flux).
fn count_file_lines(path: &Path) -> io::Result<usize> {
    let f = File::open(path)?;
    let reader = io::BufReader::new(f);
    // lines() cost: allocates String per line; acceptable for counting.
    let mut count = 0usize;
    for _ in reader.lines() {
        count += 1;
    }
    Ok(count)
}

/// Écrit un bloc (header + contenu du fichier) dans le writer en flux.
/// Retourne Ok(()) ; le comptage des lignes est fait séparément via `count_file_lines`.
fn write_block_from_path<W: Write>(w: &mut W, header: &str, path: &Path) -> io::Result<()> {
    // header
    w.write_all(header.as_bytes())?;
    // copy file content
    let mut f = File::open(path)?;
    io::copy(&mut f, w)?;
    // add two trailing newlines as dans le script original
    w.write_all(b"\n\n")?;
    Ok(())
}
