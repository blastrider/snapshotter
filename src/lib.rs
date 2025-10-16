//! snapshotter — librairie pour créer des snapshots découpés d'un dépôt Rust.
//!
//! MSRV: 1.90.0
//!
//! Exemple rapide:
//! ```no_run
//! use snapshotter::{SnapshotConfig, Snapshotter};
//! let cfg = SnapshotConfig::new(".", "snapshot", 1500).unwrap();
//! let res = Snapshotter::new(cfg).run().unwrap();
//! println!("Parts: {}", res.parts.len());
//! ```

pub mod collector;
pub mod config;
pub mod error;
pub mod types;
pub mod utils;
pub mod writer;

pub use crate::config::SnapshotConfig;
pub use crate::error::SnapshotError;
pub use crate::types::{PartSummary, SnapshotResult};
pub use crate::writer::Snapshotter;
