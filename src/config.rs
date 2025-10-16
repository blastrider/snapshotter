use std::path::{Path, PathBuf};

use crate::error::SnapshotError;

/// Configuration immuable pour l'exécution du snapshot.
#[derive(Clone, Debug)]
pub struct SnapshotConfig {
    pub workspace_root: PathBuf,
    pub dest_dir: PathBuf,
    pub prefix: String,
    pub max_lines: usize,
    pub dry_run: bool,
}

impl SnapshotConfig {
    /// Crée une nouvelle configuration. Valide les paramètres basiques.
    pub fn new<P: AsRef<Path>, S: Into<String>>(
        workspace_root: P,
        prefix: S,
        max_lines: usize,
    ) -> Result<Self, SnapshotError> {
        if max_lines == 0 {
            return Err(SnapshotError::InvalidArgument(
                "max_lines must be > 0".into(),
            ));
        }
        Ok(Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
            dest_dir: PathBuf::from("."),
            prefix: prefix.into(),
            max_lines,
            dry_run: false,
        })
    }

    /// Modifie le répertoire de destination.
    pub fn with_dest<P: AsRef<Path>>(mut self, dest: P) -> Self {
        self.dest_dir = dest.as_ref().to_path_buf();
        self
    }

    /// Active/désactive le mode dry-run (ne crée pas de fichiers).
    pub fn with_dry_run(mut self, dry: bool) -> Self {
        self.dry_run = dry;
        self
    }
}
