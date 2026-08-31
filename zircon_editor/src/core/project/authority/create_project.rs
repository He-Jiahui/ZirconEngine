use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime_interface::project::{render_project_template, RenderedProjectTemplate};

use super::super::filesystem::{
    canonical_resolved_project_root, resolve_project_path, validate_creation_target,
};
use super::transaction::{
    cleanup_failed_transaction_staging, commit_staged_directory, finalize_empty_target_backup,
    rollback_committed_project,
};
use super::ProjectAuthority;
use crate::core::project::{CreatedProject, NewProjectDraft, ProjectAuthorityError};

static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(1);

impl ProjectAuthority {
    pub fn create_project(
        &self,
        draft: &NewProjectDraft,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        let target = draft.validate_for_creation()?;
        let rendered = render_project_template(draft.template.pack_id(), &draft.project_name)?;
        self.create_rendered_project(&target, rendered)
    }

    pub(crate) fn create_rendered_project(
        &self,
        target: &Path,
        rendered: RenderedProjectTemplate,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        let target = resolve_project_path(target)?;
        validate_creation_target(&target)?;
        let parent = target
            .parent()
            .ok_or_else(|| ProjectAuthorityError::ProjectMissing {
                path: target.to_path_buf(),
            })?;
        fs::create_dir_all(parent)
            .map_err(|source| ProjectAuthorityError::io("create project parent", parent, source))?;
        let transaction = NEXT_TRANSACTION.fetch_add(1, Ordering::Relaxed);
        let stem = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project");
        let staging = parent.join(format!(
            ".{stem}.zircon-staging-{}-{transaction}",
            std::process::id()
        ));
        let backup = parent.join(format!(
            ".{stem}.zircon-backup-{}-{transaction}",
            std::process::id()
        ));

        let mut staging_created = false;
        let result = (|| {
            fs::create_dir(&staging).map_err(|source| {
                ProjectAuthorityError::io("create project staging directory", &staging, source)
            })?;
            staging_created = true;
            for entry in rendered.entries {
                let destination = entry.path.join_to(&staging);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(|source| {
                        ProjectAuthorityError::io("create template directory", parent, source)
                    })?;
                }
                fs::write(&destination, entry.bytes).map_err(|source| {
                    ProjectAuthorityError::io("write template entry", &destination, source)
                })?;
            }
            let staging_paths = ProjectPaths::from_root(&staging).map_err(|source| {
                ProjectAuthorityError::io("resolve staging project paths", &staging, source)
            })?;
            staging_paths.ensure_derived_layout().map_err(|source| {
                ProjectAuthorityError::io("create staging derived layout", &staging, source)
            })?;
            let manifest_path = staging.join("zircon-project.toml");
            let manifest = ProjectManifest::load(&manifest_path)?;
            manifest.save(&manifest_path)?;

            let replaced_empty_target = target.exists();
            commit_staged_directory(
                &staging,
                &target,
                &backup,
                replaced_empty_target,
                |from, to| fs::rename(from, to),
            )?;
            let root = match canonical_resolved_project_root(&target) {
                Ok(root) => root,
                Err(error) => {
                    rollback_committed_project(
                        &staging,
                        &target,
                        &backup,
                        replaced_empty_target,
                        |from, to| fs::rename(from, to),
                    )?;
                    return Err(error);
                }
            };
            let project = match ProjectManager::open_resolved(&root) {
                Ok(project) => project,
                Err(source) => {
                    rollback_committed_project(
                        &staging,
                        &target,
                        &backup,
                        replaced_empty_target,
                        |from, to| fs::rename(from, to),
                    )?;
                    return Err(source.into());
                }
            };
            if let Err(error) =
                finalize_empty_target_backup(&target, &backup, replaced_empty_target)
            {
                drop(project);
                rollback_committed_project(
                    &staging,
                    &target,
                    &backup,
                    replaced_empty_target,
                    |from, to| fs::rename(from, to),
                )?;
                return Err(error);
            }
            let summary = project.manifest().summary();
            Ok(CreatedProject::new(root, summary, project))
        })();

        let preserve_rollback_artifacts = matches!(
            &result,
            Err(ProjectAuthorityError::PostCommitRollbackFailed { .. })
        );
        if result.is_err() {
            cleanup_failed_transaction_staging(
                &staging,
                preserve_rollback_artifacts,
                staging_created,
            );
        }
        result
    }
}
