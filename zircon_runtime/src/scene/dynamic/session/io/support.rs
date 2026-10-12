use std::fs;
use std::path::Path;

use super::super::RuntimeSessionArchiveError;

pub(super) fn ensure_parent_dir(path: &Path) -> Result<(), RuntimeSessionArchiveError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}
