//! Resolve one physical address for both process-local admission and all config I/O.
use std::fs;
use std::io;
use std::path::Path;

use crate::asset::project::{ProjectPaths, ResolvedProjectPathIdentity};

pub(super) fn physical_path(path: &Path) -> io::Result<ResolvedProjectPathIdentity> {
    // Capture relative-path context once, including Windows drive-relative inputs.
    // The symlink probe and physical resolver must consume the same absolute path.
    let path = std::path::absolute(path)?;
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "config persistence path did not resolve to an absolute address",
        ));
    }
    // Atomic publication rejects a final symlink rather than replacing or following it.
    // Check before canonicalization so admission cannot silently change that contract.
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "config persistence does not support a final symlink: {}",
                    path.display()
                ),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    ProjectPaths::resolve_identity(&path)
}
