mod boundary;
mod directory_transaction;
mod recent_projects;
mod root_resolution;
mod scene_document;
mod template_creation;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

fn temp_root(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "zircon-editor-project-authority-{label}-{}-{}",
        std::process::id(),
        NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}
