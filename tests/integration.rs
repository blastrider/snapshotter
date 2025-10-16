use std::fs::{self, File};
use std::io::Write;
use tempfile::tempdir;

use snapshotter::{SnapshotConfig, SnapshotError, Snapshotter};

#[test]
fn basic_snapshot_flow() -> Result<(), SnapshotError> {
    let td = tempdir().unwrap();
    let root = td.path().join("proj");
    fs::create_dir_all(&root).unwrap();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    let f1 = src.join("a.rs");
    let mut f = File::create(&f1).unwrap();
    writeln!(f, "fn main() {{\nprintln!(\"hello\");\n}}").unwrap();

    let cfg = SnapshotConfig::new(&root, "snap", 10)?.with_dest(td.path().join("out"));
    let s = Snapshotter::new(cfg);
    let res = s.run()?;
    assert!(res.total_files >= 1);
    assert!(res.total_lines > 0);
    Ok(())
}
