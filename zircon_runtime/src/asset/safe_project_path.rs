use std::fs;
use std::path::Path;

use crate::asset::project::ProjectPaths;

pub(crate) fn is_safe_regular_file(root: &Path, path: &Path) -> std::io::Result<bool> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || is_link_or_reparse(&metadata) {
        return Ok(false);
    }
    let physical_path = ProjectPaths::resolve_existing_path(path)?;
    let physical_root = ProjectPaths::resolve_existing_path(root)?;
    Ok(physical_path.starts_with(physical_root))
}

pub(crate) fn is_link_or_reparse(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    }
    #[cfg(not(windows))]
    false
}
