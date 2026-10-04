use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime_interface::project::{
    render_project_template, ProjectCreationProvenance, RenderedProjectTemplate,
};

use super::super::filesystem::{
    canonical_resolved_project_root, resolve_project_path_with_identity, validate_creation_target,
};
use super::transaction::{
    cleanup_failed_transaction_staging, commit_staged_directory, finalize_published_project,
    ProjectCreationLease,
};
use super::ProjectAuthority;
use crate::core::project::{CreatedProject, NewProjectDraft, ProjectAuthorityError};

static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(1);

impl ProjectAuthority {
    pub fn create_project(
        &self,
        draft: &NewProjectDraft,
        provenance: &ProjectCreationProvenance,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        self.create_project_for_activation(draft, provenance)
    }

    pub(crate) fn create_project_for_activation(
        &self,
        draft: &NewProjectDraft,
        provenance: &ProjectCreationProvenance,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        let target = draft.validate_for_creation()?;
        let rendered = render_project_template(draft.template, &draft.project_name)?;
        self.create_rendered_project(&target, &rendered, provenance)
    }

    pub(crate) fn create_rendered_project(
        &self,
        target: &Path,
        rendered: &RenderedProjectTemplate,
        provenance: &ProjectCreationProvenance,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        let project_guid = rendered
            .summary
            .project_guid
            .ok_or(ProjectAuthorityError::RenderedTemplateMissingProjectGuid)?;
        let template_receipt = provenance.issue_receipt(rendered.descriptor, project_guid)?;
        let target_identity = resolve_project_path_with_identity(target)?;
        let target = target_identity.operation_path().to_path_buf();
        validate_creation_target(&target)?;
        let parent = target
            .parent()
            .ok_or_else(|| ProjectAuthorityError::ProjectMissing {
                path: target.to_path_buf(),
            })?;
        fs::create_dir_all(parent)
            .map_err(|source| ProjectAuthorityError::io("create project parent", parent, source))?;
        let creation_lease = ProjectCreationLease::acquire(&target)?;
        debug_assert_eq!(creation_lease.target(), target.as_path());
        validate_creation_target(&target)?;
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
            for entry in &rendered.entries {
                let destination = entry.path.join_to(&staging);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(|source| {
                        ProjectAuthorityError::io("create template directory", parent, source)
                    })?;
                }
                fs::write(&destination, &entry.bytes).map_err(|source| {
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
            let mut manifest = ProjectManifest::load(&manifest_path)?;
            manifest.template_receipt = Some(template_receipt);
            manifest.save(&manifest_path)?;

            let staging_root = canonical_resolved_project_root(&staging)?;
            let preflight = self.preflight_resolved_project(&staging_root)?;

            let preflight = preflight.rebind_resolved_project_path(target_identity)?;

            let replaced_empty_target = target.exists();
            commit_staged_directory(
                &staging,
                &target,
                &backup,
                replaced_empty_target,
                |from, to| fs::rename(from, to),
            )?;
            finalize_published_project(&target, &backup, replaced_empty_target)?;
            Ok(CreatedProject::new(preflight))
        })();

        if result.is_err() {
            cleanup_failed_transaction_staging(&staging, false, staging_created);
        }
        result
    }

    pub(crate) fn create_preflighted_project_for_activation(
        &self,
        target: &Path,
        rendered: &RenderedProjectTemplate,
        provenance: &ProjectCreationProvenance,
    ) -> Result<CreatedProject, ProjectAuthorityError> {
        self.create_rendered_project(target, rendered, provenance)
    }
}
