use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::{ProjectManager, ResolvedProjectPath};

use super::super::filesystem::{
    canonical_resolved_project_root, validate_canonical_existing_project_root,
};
use super::ProjectAuthority;
use crate::core::project::{OpenedProject, ProjectAuthorityError};

impl ProjectAuthority {
    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<OpenedProject, ProjectAuthorityError> {
        let root = self.resolve_existing_project_root_with_identity(path)?;
        self.open_resolved_project(&root)
    }

    /// Opens a project from a physical root resolved by an upstream boundary.
    ///
    /// The validation intentionally does not resolve the path again; `ResolvedProjectPath`
    /// already owns the operation identity selected by the caller.
    pub fn open_resolved_project(
        &self,
        root: &ResolvedProjectPath,
    ) -> Result<OpenedProject, ProjectAuthorityError> {
        validate_canonical_existing_project_root(root.operation_path())?;
        Ok(OpenedProject::new(
            ProjectManager::open_resolved(root)?,
            root.clone(),
        ))
    }

    pub fn resolve_existing_project_root(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<PathBuf, ProjectAuthorityError> {
        self.resolve_existing_project_root_with_identity(path)
            .map(ResolvedProjectPath::into_operation_path)
    }

    /// Resolves an existing project input once for callers that subsequently open it.
    ///
    /// The resolved identity retains the physical operation path and the Windows-safe display
    /// view so the next owner does not need a platform-specific path compatibility branch.
    pub(crate) fn resolve_existing_project_root_with_identity(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<ResolvedProjectPath, ProjectAuthorityError> {
        canonical_resolved_project_root(path.as_ref())
    }
}
