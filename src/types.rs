use std::path::PathBuf;

/// Résumé d'une part (part N).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartSummary {
    pub part_index: usize,
    pub file_count: usize,
    pub line_count: usize,
    pub filename: PathBuf,
}

/// Résultat global de l'exécution.
#[derive(Debug, Clone)]
pub struct SnapshotResult {
    pub parts: Vec<PartSummary>,
    pub total_files: usize,
    pub total_lines: usize,
}
