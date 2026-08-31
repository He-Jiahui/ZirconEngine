use std::path::Path;

use zircon_runtime::asset::project::ProjectPaths;

use super::super::filesystem::{
    resolve_project_root_identity, validate_canonical_existing_project_root,
};
use super::super::preflight_manifest_reader::inspect_project_manifest;
use super::ProjectAuthority;
use crate::core::project::RecentProjectValidation;

impl ProjectAuthority {
    pub fn validate_recent_project(&self, path: &str) -> RecentProjectValidation {
        let Ok(root) = resolve_project_root_identity(Path::new(path)) else {
            return RecentProjectValidation::InvalidProject;
        };
        if !root.operation_path().exists() {
            return RecentProjectValidation::Missing;
        }
        if validate_canonical_existing_project_root(root.operation_path()).is_err() {
            return RecentProjectValidation::Missing;
        }
        let paths = ProjectPaths::from_resolved_root(&root);
        match inspect_project_manifest(paths.manifest_path()) {
            Ok(inspection) if inspection.migrated_from.is_some() => {
                RecentProjectValidation::RequiresMigration
            }
            Ok(_) => RecentProjectValidation::Valid,
            Err(_) => RecentProjectValidation::InvalidManifest,
        }
    }
}
