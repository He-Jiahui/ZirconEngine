use std::path::Path;

use zircon_asset::{ProjectManifest, ProjectPaths};

use crate::project::EditorProjectDocument;
use crate::ui::workbench::startup::RecentProjectValidation;

use super::super::editor_manager::EditorManager;
use super::canonical_project_root::canonical_project_root;

impl EditorManager {
    pub(super) fn validate_recent_project(&self, path: &str) -> RecentProjectValidation {
        let Ok(root) = canonical_project_root(Path::new(path)) else {
            return RecentProjectValidation::Missing;
        };
        if !root.exists() {
            return RecentProjectValidation::Missing;
        }
        let Ok(paths) = ProjectPaths::from_root(&root) else {
            return RecentProjectValidation::InvalidProject;
        };
        if !paths.manifest_path().exists() {
            return RecentProjectValidation::Missing;
        }
        if ProjectManifest::load(paths.manifest_path()).is_err() {
            return RecentProjectValidation::InvalidManifest;
        }
        match EditorProjectDocument::load_from_path(&root) {
            Ok(_) => RecentProjectValidation::Valid,
            Err(_) => RecentProjectValidation::InvalidProject,
        }
    }
}
