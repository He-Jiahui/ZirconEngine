use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::RuntimeSessionArchiveError;

static TEMP_ARCHIVE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(super) fn ensure_parent_dir(path: &Path) -> Result<(), RuntimeSessionArchiveError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub(super) fn temporary_archive_path(path: &Path, extension: &str) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("runtime-session-archive");
    let counter = TEMP_ARCHIVE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    parent.join(format!(
        ".{file_name}.{}.{}.{}.{}",
        process::id(),
        unique,
        counter,
        extension
    ))
}
