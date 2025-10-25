use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Récupère la liste de fichiers à snapshot, triés de façon déterministe.
///
/// Politique :
///  - ./src : uniquement *.rs
///  - ./tests : tous fichiers
///  - ./migrations : tous fichiers
///  - fichiers racine : askama.toml, Cargo.toml (si présents)
pub fn collect_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut list: Vec<PathBuf> = Vec::new();

    let src = root.join("src");
    if src.exists() {
        push_files_with_ext(&src, "rs", &mut list)?;
    }

    let tests = root.join("tests");
    if tests.exists() {
        push_files_recursive(&tests, &mut list)?;
    }

    let migrations = root.join("migrations");
    if migrations.exists() {
        push_files_recursive(&migrations, &mut list)?;
    }

    for candidate in &["askama.toml", "Cargo.toml"] {
        let p = root.join(candidate);
        if p.exists() {
            list.push(p);
        }
    }

    // normalise & sort for determinisme
    list.sort();
    Ok(list)
}

/// Ajoute récursivement tous les fichiers du répertoire.
fn push_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let e = entry?;
        let p = e.path();
        if p.is_dir() {
            push_files_recursive(&p, out)?;
        } else {
            out.push(p);
        }
    }
    Ok(())
}

/// Ajoute récursivement les fichiers ayant l'extension donnée.
fn push_files_with_ext(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let e = entry?;
        let p = e.path();
        if p.is_dir() {
            push_files_with_ext(&p, ext, out)?;
        } else if p.is_file() && p.extension().and_then(OsStr::to_str) == Some(ext) {
            out.push(p);
        }
    }
    Ok(())
}
