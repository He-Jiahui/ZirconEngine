use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use super::NEXT_ATOMIC_FILE_ID;

pub(super) fn unique_sibling_path(directory: &Path, target: &Path, role: &str) -> PathBuf {
    let file_name = target_file_name(target);
    loop {
        let id = NEXT_ATOMIC_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = directory.join(format!(
            ".{file_name}.zr-{role}-{}-{id}",
            std::process::id()
        ));
        if !candidate.exists() {
            return candidate;
        }
    }
}

pub(super) fn target_file_name(target: &Path) -> &str {
    target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("zircon.data")
}

pub(crate) fn is_atomic_write_transaction_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !file_name.starts_with('.') {
        return false;
    }
    [".zr-staging-", ".zr-backup-"].into_iter().any(|marker| {
        let Some((_, suffix)) = file_name.rsplit_once(marker) else {
            return false;
        };
        let Some((process_id, sequence)) = suffix.split_once('-') else {
            return false;
        };
        !process_id.is_empty()
            && !sequence.is_empty()
            && process_id.bytes().all(|byte| byte.is_ascii_digit())
            && sequence.bytes().all(|byte| byte.is_ascii_digit())
    })
}
