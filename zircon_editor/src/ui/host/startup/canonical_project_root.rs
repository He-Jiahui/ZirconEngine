use std::fs;
use std::path::{Path, PathBuf};

use crate::ui::workbench::project::project_root_path;

pub(super) fn canonical_project_root(path: &Path) -> Result<PathBuf, std::io::Error> {
    let root = project_root_path(path)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
    if root.exists() {
        fs::canonicalize(root)
    } else {
        Ok(root)
    }
}
