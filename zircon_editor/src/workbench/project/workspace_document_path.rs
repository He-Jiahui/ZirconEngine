use std::path::{Path, PathBuf};

use super::constants::{EDITOR_WORKSPACE_DIR, EDITOR_WORKSPACE_FILE};

pub(in crate::workbench::project) fn workspace_document_path(root: &Path) -> PathBuf {
    root.join(EDITOR_WORKSPACE_DIR).join(EDITOR_WORKSPACE_FILE)
}
