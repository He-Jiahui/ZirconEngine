use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};

use crate::core::hub_link::{publish_focus_signal, record_recent_project};
use crate::core::project::ProjectAuthority;
use crate::core::recovery::{SessionGuard, SessionGuardAdmission, SessionLockRecord};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::{now_unix_ms, EditorStartupSessionDocument};
use zircon_runtime_interface::hub_protocol::HubSessionToken;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub(super) fn open_project_document(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorProjectDocument, EditorError> {
        let project = ProjectAuthority::default().open_project(path)?;
        self.activate_prepared_project(project.into_project(), Ok)
    }

    pub(super) fn open_project_and_remember_with_session(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let project = ProjectAuthority::default().open_project(path)?;
        self.open_prepared_project_and_remember_with_session(project.into_project())
    }

    pub(super) fn open_prepared_project_and_remember_with_session(
        &self,
        project: ProjectManager,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        self.activate_prepared_project(project, |document| {
            self.host.remember_prepared_project(document)
        })
    }

    pub(super) fn create_project_and_open_with_session(
        &self,
        draft: crate::core::project::NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let project = ProjectAuthority::default().create_project(&draft)?;
        self.open_prepared_project_and_remember_with_session(project.into_project())
    }

    pub(super) fn release_project_session_guard(&self) -> Result<(), EditorError> {
        let mut guard_slot = self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(guard) = guard_slot.as_mut() {
            guard.release().map_err(|error| {
                EditorError::Project(format!(
                    "project runtime closed, but its session guard could not be released: {error}"
                ))
            })?;
        }
        guard_slot.take();
        Ok(())
    }

    /// Returns the immutable identity consumed by the retained-host Hub focus watcher.
    ///
    /// The watcher receives a snapshot only; it has no capability to mutate the session lock.
    pub(crate) fn active_project_session_focus_target(&self) -> Option<(PathBuf, String)> {
        self.project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map(|guard| {
                (
                    guard.project_root().to_path_buf(),
                    guard.record().instance_id().to_string(),
                )
            })
    }

    fn take_hub_launch_session(&self) -> Option<HubSessionToken> {
        self.hub_launch_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }

    fn activate_prepared_project<T>(
        &self,
        project: ProjectManager,
        finish: impl FnOnce(EditorProjectDocument) -> Result<T, EditorError>,
    ) -> Result<T, EditorError> {
        let project_root = project.paths().root().to_path_buf();
        self.admit_project_session(&project_root, || {
            let document = self.host.open_prepared_project(project)?;
            if let Err(error) = self.complete_project_open(&document) {
                return Err(self.rollback_failed_project_activation(error));
            }
            finish(document).map_err(|error| self.rollback_failed_project_activation(error))
        })
    }

    fn admit_project_session<T>(
        &self,
        project_root: &Path,
        activate: impl FnOnce() -> Result<T, EditorError>,
    ) -> Result<T, EditorError> {
        if self
            .project_session_guard
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
        {
            return Err(EditorError::Project(format!(
                "cannot activate `{}` while another editor project session is active",
                ProjectPaths::display_path(project_root).display()
            )));
        }

        let hub_launch_session = self.take_hub_launch_session();
        let mut guard = match SessionGuard::claim(project_root).map_err(|error| {
            EditorError::Project(format!(
                "cannot claim the project session for `{}`: {error}",
                ProjectPaths::display_path(project_root).display()
            ))
        })? {
            SessionGuardAdmission::Acquired(guard) => guard,
            SessionGuardAdmission::Active { record } => {
                if let (Some(session), Some(record)) = (hub_launch_session, record.as_ref()) {
                    publish_focus_signal(project_root, record, session).map_err(|error| {
                        EditorError::Project(format!(
                            "active project session for `{}` could not receive the Hub focus signal: {error}",
                            ProjectPaths::display_path(project_root).display(),
                        ))
                    })?;
                    return Err(EditorError::HubFocusForwarded {
                        process_id: record.process_id(),
                    });
                }
                return Err(EditorError::Project(active_project_session_message(
                    project_root,
                    record.as_ref(),
                )));
            }
            SessionGuardAdmission::Residual(residual) => {
                return Err(EditorError::Project(format!(
                    "project session recovery is required for `{}` after editor instance `{}`; the residual lock was preserved",
                    ProjectPaths::display_path(project_root).display(),
                    residual.record().instance_id(),
                )));
            }
        };

        match activate() {
            Ok(value) => {
                let mut guard_slot = self
                    .project_session_guard
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if guard_slot.is_some() {
                    drop(guard_slot);
                    let concurrent_session = EditorError::Project(format!(
                        "project session for `{}` changed while activation was in progress",
                        ProjectPaths::display_path(project_root).display()
                    ));
                    return match guard.release() {
                        Ok(_) => Err(concurrent_session),
                        Err(release_error) => Err(EditorError::Project(format!(
                            "{concurrent_session}; additionally failed to release the uncommitted session guard: {release_error}"
                        ))),
                    };
                }
                *guard_slot = Some(guard);
                Ok(value)
            }
            Err(activation_error) => match guard.release() {
                Ok(_) => Err(activation_error),
                Err(release_error) => Err(EditorError::Project(format!(
                    "project activation failed: {activation_error}; additionally failed to release its session guard: {release_error}"
                ))),
            },
        }
    }

    fn complete_project_open(&self, document: &EditorProjectDocument) -> Result<(), EditorError> {
        self.configure_project_diagnostics(&document.root_path)?;
        self.apply_project_plugin_manifest(&document.root_path, &document.manifest)?;
        record_recent_project(
            &document.root_path,
            document.manifest.summary(),
            now_unix_ms(),
        )
        .map_err(|error| {
            EditorError::Project(format!(
                "could not update the shared recent-project registry for `{}`: {error}",
                ProjectPaths::display_path(&document.root_path).display()
            ))
        })?;
        let activation = self
            .document_lifecycle
            .begin_project_session(&document.root_path);
        self.publish_document_messages(activation.messages);
        Ok(())
    }

    fn rollback_failed_project_activation(&self, activation_error: EditorError) -> EditorError {
        self.clear_project_plugin_status();
        let registrations = self.plugin_manager().clear_project_registration_reports();
        let close_result = self.host.close_project();
        if close_result.is_ok() {
            self.context().logs().disable_rolling_file();
            self.context().settings().clear_project_layer();
        }
        match (close_result, registrations) {
            (Ok(_), Ok(_)) => activation_error,
            (Err(close_error), _) => EditorError::Project(format!(
                "project activation failed: {activation_error}; additionally failed to roll back the runtime project: {close_error}"
            )),
            (Ok(_), Err(registration_error)) => EditorError::Project(format!(
                "project activation failed: {activation_error}; additionally failed to clear project-native registrations: {registration_error}"
            )),
        }
    }
}

fn active_project_session_message(
    project_root: &Path,
    record: Option<&SessionLockRecord>,
) -> String {
    let project_root = ProjectPaths::display_path(project_root);
    match record {
        Some(record) => format!(
            "project `{}` is already active in editor process {} (instance `{}`)",
            project_root.display(),
            record.process_id(),
            record.instance_id(),
        ),
        None => format!(
            "project `{}` is already active in another editor instance",
            project_root.display(),
        ),
    }
}
