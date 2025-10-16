use std::path::{Path, PathBuf};

/// Compte les '\n' dans un string (similaire à wc -l).
pub fn count_newlines(s: &str) -> usize {
    s.as_bytes().iter().filter(|&&b| b == b'\n').count()
}

/// Retourne le chemin relatif `file` par rapport à `base` (ou file si impossible).
pub fn path_relative(file: &Path, base: &Path) -> PathBuf {
    match file.strip_prefix(base) {
        Ok(p) => PathBuf::from(".").join(p),
        Err(_) => file.to_path_buf(),
    }
}
