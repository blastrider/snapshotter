use std::io;

use thiserror::Error;

/// Erreurs publiques de la librairie.
#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("No files to snapshot")]
    NoFiles,
}
