use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime_interface::project::{
    render_project_template, ProjectManifestSummary, RenderedProjectTemplate,
};

use super::filesystem::{
    canonical_project_root, validate_creation_target, validate_existing_project_root,
};
use super::{
    CreatedProject, NewProjectDraft, OpenedProject, ProjectAuthorityError, RecentProjectEntry,
    RecentProjectValidation, StoredRecentProjectEntry, StoredStartupSession,
};

const RECENT_PROJECT_LIMIT: usize = 8;
static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(1);

/// Headless-safe owner for Editor project identity, creation, opening, and recents.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProjectAuthority;

impl ProjectAuthority {
    pub(crate) fn decode_startup_session(
        &self,
        value: serde_json::Value,
    ) -> Result<StoredStartupSession, ProjectAuthorityError> {
        serde_json::from_value(value)
            .map_err(|source| ProjectAuthorityError::SessionDecode { source })
    }

    pub(crate) fn encode_startup_session(
        &self,
        session: &StoredStartupSession,
    ) -> Result<serde_json::Value, ProjectAuthorityError> {
        serde_json::to_value(session)
            .map_err(|source| ProjectAuthorityError::SessionEncode { source })
    }

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
        validate_creation_target(target)?;
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

        let result = (|| {
            fs::create_dir(&staging).map_err(|source| {
                ProjectAuthorityError::io("create project staging directory", &staging, source)
            })?;
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
                target,
                &backup,
                replaced_empty_target,
                |from, to| fs::rename(from, to),
            )?;
            Ok(CreatedProject {
                root: target.to_path_buf(),
                summary: rendered.summary,
            })
        })();

        if result.is_err() {
            remove_transaction_path(&staging);
        }
        result
    }

    pub fn open_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<OpenedProject, ProjectAuthorityError> {
        let opened = self.probe_project(path)?;
        let paths = ProjectPaths::from_root(&opened.root).map_err(|source| {
            ProjectAuthorityError::io("resolve project paths", &opened.root, source)
        })?;
        paths.ensure_derived_layout().map_err(|source| {
            ProjectAuthorityError::io("create project derived layout", &opened.root, source)
        })?;
        Ok(opened)
    }

    pub fn probe_project(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<OpenedProject, ProjectAuthorityError> {
        let root = canonical_project_root(path.as_ref())?;
        validate_existing_project_root(&root)?;
        let paths = ProjectPaths::from_root(&root)
            .map_err(|source| ProjectAuthorityError::io("resolve project paths", &root, source))?;
        let manifest = ProjectManifest::load(paths.manifest_path())?;
        Ok(OpenedProject {
            root,
            summary: manifest.summary(),
        })
    }

    pub fn probe_draft(
        &self,
        draft: &NewProjectDraft,
    ) -> Result<OpenedProject, ProjectAuthorityError> {
        self.probe_project(draft.project_root()?)
    }

    pub fn validate_recent_project(&self, path: &str) -> RecentProjectValidation {
        let Ok(root) = canonical_project_root(Path::new(path)) else {
            return RecentProjectValidation::InvalidProject;
        };
        if !root.exists() {
            return RecentProjectValidation::Missing;
        }
        if validate_existing_project_root(&root).is_err() {
            return RecentProjectValidation::Missing;
        }
        let Ok(paths) = ProjectPaths::from_root(&root) else {
            return RecentProjectValidation::InvalidProject;
        };
        match ProjectManifest::load(paths.manifest_path()) {
            Ok(_) => RecentProjectValidation::Valid,
            Err(_) => RecentProjectValidation::InvalidManifest,
        }
    }

    pub fn remember_recent_project(
        &self,
        session: &mut StoredStartupSession,
        path: &str,
        summary: ProjectManifestSummary,
        now_unix_ms: u64,
    ) {
        session.last_project_path = Some(path.to_string());
        session.recent_projects.retain(|entry| entry.path != path);
        session.recent_projects.push(StoredRecentProjectEntry {
            summary,
            path: path.to_string(),
            last_opened_unix_ms: now_unix_ms,
        });
        session.recent_projects.sort_by(|left, right| {
            right
                .last_opened_unix_ms
                .cmp(&left.last_opened_unix_ms)
                .then_with(|| left.path.cmp(&right.path))
        });
        session.recent_projects.truncate(RECENT_PROJECT_LIMIT);
    }

    pub fn forget_recent_project(&self, session: &mut StoredStartupSession, path: &str) {
        session.recent_projects.retain(|entry| entry.path != path);
        if session.last_project_path.as_deref() == Some(path) {
            session.last_project_path = session
                .recent_projects
                .first()
                .map(|entry| entry.path.clone());
        }
    }

    pub fn recent_projects_with_validation<F>(
        &self,
        session: &StoredStartupSession,
        mut validate: F,
    ) -> Vec<RecentProjectEntry>
    where
        F: FnMut(&str) -> RecentProjectValidation,
    {
        session
            .recent_projects
            .iter()
            .map(|entry| RecentProjectEntry {
                summary: entry.summary.clone(),
                path: entry.path.clone(),
                last_opened_unix_ms: entry.last_opened_unix_ms,
                validation: validate(&entry.path),
            })
            .collect()
    }
}

pub(super) fn commit_staged_directory<R>(
    staging: &Path,
    target: &Path,
    backup: &Path,
    replace_empty_target: bool,
    mut rename: R,
) -> Result<(), ProjectAuthorityError>
where
    R: FnMut(&Path, &Path) -> std::io::Result<()>,
{
    if replace_empty_target {
        rename(target, backup).map_err(|source| {
            ProjectAuthorityError::io("stage empty target rollback", target, source)
        })?;
    }

    if let Err(commit_source) = rename(staging, target) {
        if replace_empty_target {
            if let Err(restore_source) = rename(backup, target) {
                return Err(ProjectAuthorityError::CommitRollbackFailed {
                    target: target.to_path_buf(),
                    backup: backup.to_path_buf(),
                    commit_source,
                    restore_source,
                });
            }
        }
        return Err(ProjectAuthorityError::io(
            "commit project template",
            target,
            commit_source,
        ));
    }

    if replace_empty_target {
        // The backup is known to be the previously empty target. Failure to remove this empty
        // directory does not invalidate the committed project, so preserve it for later cleanup.
        match fs::remove_dir(backup) {
            Ok(()) => {}
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
            Err(_cleanup_source) => {}
        }
    }
    Ok(())
}

fn remove_transaction_path(path: &PathBuf) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    } else if path.exists() {
        let _ = fs::remove_file(path);
    }
}
