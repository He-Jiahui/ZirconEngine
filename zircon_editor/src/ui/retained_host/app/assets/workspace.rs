use super::super::*;
use crate::core::asset::EditorModelImportTicket;
use crate::core::document::ActiveSceneDocumentIdentity;
use crate::core::jobs::JobSubmitError;
use crate::core::project::{
    ProjectAuthority, ProjectSceneDocument, ProjectSceneLoadTicket, SceneOpenRequest,
};
use crate::ui::host::{PreparedActiveSceneReloadDirtyPolicy, PreparedActiveSceneReloadOutcome};
use crate::ui::retained_host::app::workbench_notifications::{
    import_model_completed_toast, import_model_failed_toast,
};
use std::time::{Duration, Instant};
use zircon_runtime::asset::pipeline::manager::{
    ProjectAssetGenerationToken, ProjectGenerationMatch,
};
use zircon_runtime::asset::ProjectImportReceipt;

mod active_scene_reload_conflict;

pub(in crate::ui::retained_host::app) use active_scene_reload_conflict::ActiveSceneReloadConflict;

const ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_LIMIT: u8 = 3;
const ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_BASE_DELAY: Duration = Duration::from_millis(64);

pub(in crate::ui::retained_host::app) struct PendingActiveSceneReload {
    ticket: ProjectSceneLoadTicket,
    generation: ProjectAssetGenerationToken,
    identity: ActiveSceneDocumentIdentity,
    dirty_policy: PreparedActiveSceneReloadDirtyPolicy,
    reload_requested: bool,
}

pub(in crate::ui::retained_host::app) struct ActiveSceneReloadAdmissionState {
    identity: ActiveSceneDocumentIdentity,
    generation: ProjectAssetGenerationToken,
    consecutive_failures: u8,
    retry_not_before: Option<Instant>,
}

pub(in crate::ui::retained_host::app) struct PendingModelImport {
    ticket: EditorModelImportTicket,
    display_path: String,
    close_requested: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) enum ActiveSceneReloadOutcome {
    Committed,
    Superseded,
    Discarded,
    Conflict {
        identity: ActiveSceneDocumentIdentity,
        generation: ProjectAssetGenerationToken,
    },
}

impl RetainedEditorHost {
    pub(super) fn active_scene_reload_admission_retry_deadline(&self) -> Option<Instant> {
        self.active_scene_reload_admission
            .as_ref()
            .and_then(|admission| admission.retry_not_before)
    }

    pub(in crate::ui::retained_host::app) fn request_active_scene_reload(
        &mut self,
    ) -> Result<(), String> {
        if let Some(pending) = self.pending_active_scene_reload.as_mut() {
            pending.reload_requested = true;
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.active_scene_reload_coalesced",
                1
            );
            return Ok(());
        }
        if let Some(admission) = self.active_scene_reload_admission.as_ref() {
            let same_identity = self
                .editor_manager
                .active_scene_identity_for_session()
                .as_ref()
                == Some(&admission.identity);
            let same_generation = if same_identity {
                let project_asset_manager = self
                    .asset_runtime_access
                    .project_asset_manager()
                    .map_err(|error| error.to_string())?;
                project_asset_manager.check_project_generation(&admission.generation)
                    == ProjectGenerationMatch::Current
            } else {
                false
            };
            if same_identity && same_generation {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.active_scene_reload_admission_coalesced",
                    1
                );
                return Ok(());
            }
            self.active_scene_reload_admission = None;
        }
        self.submit_active_scene_reload(None)
    }

    fn submit_active_scene_reload(
        &mut self,
        previous_admission: Option<ActiveSceneReloadAdmissionState>,
    ) -> Result<(), String> {
        let project_asset_manager = self
            .asset_runtime_access
            .project_asset_manager()
            .map_err(|error| error.to_string())?;
        let Some(snapshot) = project_asset_manager.current_project_generation_snapshot() else {
            return Ok(());
        };
        let (project, generation) = snapshot.into_parts();
        let Some(identity) = self.editor_manager.active_scene_identity_for_session() else {
            return Ok(());
        };
        if identity.project_root() != project.paths().root() {
            return Ok(());
        }
        let Some(dirty_policy) =
            self.active_scene_reload_conflict_dirty_policy(&identity, &generation)
        else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.active_scene_reload_conflict_coalesced",
                1
            );
            return Ok(());
        };
        let scene_uri = ResourceLocator::parse(identity.scene_uri())
            .map_err(|error| format!("active scene source URI is invalid: {error}"))?;
        let ticket = match ProjectAuthority::default().submit_scene_open(
            self.editor_manager.context().jobs(),
            project,
            SceneOpenRequest::new(scene_uri),
        ) {
            Ok(ticket) => ticket,
            Err(error @ JobSubmitError::AdmissionEntryLimitExceeded { .. })
            | Err(error @ JobSubmitError::AdmissionByteLimitExceeded { .. })
            | Err(error @ JobSubmitError::OldestPendingAgeExceeded { .. }) => {
                return self.defer_active_scene_reload_after_admission_failure(
                    identity,
                    generation,
                    previous_admission,
                    error,
                );
            }
            Err(error) => return Err(error.to_string()),
        };
        self.active_scene_reload_admission = None;
        self.pending_active_scene_reload = Some(PendingActiveSceneReload {
            ticket,
            generation,
            identity,
            dirty_policy,
            reload_requested: false,
        });
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn poll_active_scene_reload(&mut self) {
        self.poll_active_scene_reload_admission_retry();
        if self.pending_active_scene_reload.is_none() {
            self.reconcile_active_scene_reload_conflict();
        }
        let Some(pending) = self.pending_active_scene_reload.take() else {
            return;
        };
        let Some(result) = pending.ticket.try_take() else {
            self.pending_active_scene_reload = Some(pending);
            return;
        };
        let reload_requested = pending.reload_requested;
        let completed_identity = pending.identity.clone();
        let completed_generation = pending.generation.clone();
        let dirty_policy = pending.dirty_policy;
        let completion = result
            .map_err(|error| error.to_string())
            .and_then(|document| self.complete_active_scene_reload(pending, document));
        let superseded = matches!(&completion, Ok(ActiveSceneReloadOutcome::Superseded));
        if reload_requested || superseded {
            self.queue_active_scene_reload_retry();
        }
        match completion {
            Ok(ActiveSceneReloadOutcome::Committed) => {
                self.clear_active_scene_reload_conflict_for_identity(&completed_identity);
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.active_scene_reload_committed",
                    1
                );
                self.mark_render_and_presentation_dirty();
            }
            Ok(ActiveSceneReloadOutcome::Superseded) => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.active_scene_reload_superseded",
                    1
                );
            }
            Ok(ActiveSceneReloadOutcome::Discarded) => {
                self.clear_active_scene_reload_conflict_for_identity(&completed_identity);
            }
            Ok(ActiveSceneReloadOutcome::Conflict {
                identity,
                generation,
            }) => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.asset_refresh.active_scene_reload_conflict",
                    1
                );
                self.install_active_scene_reload_conflict(identity, generation);
            }
            Err(error) => {
                if dirty_policy == PreparedActiveSceneReloadDirtyPolicy::Discard {
                    self.restore_active_scene_reload_conflict_after_discard_failure(
                        completed_identity,
                        completed_generation,
                    );
                }
                self.set_status_line(error);
            }
        }
    }

    fn poll_active_scene_reload_admission_retry(&mut self) {
        let Some(admission) = self.active_scene_reload_admission.as_ref() else {
            return;
        };
        let admission_identity = admission.identity.clone();
        let admission_generation = admission.generation.clone();
        let retry_not_before = admission.retry_not_before;
        let active_identity = self.editor_manager.active_scene_identity_for_session();
        if active_identity.as_ref() != Some(&admission_identity) {
            self.active_scene_reload_admission = None;
            return;
        }
        let project_asset_manager = match self.asset_runtime_access.project_asset_manager() {
            Ok(manager) => manager,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        match project_asset_manager.check_project_generation(&admission_generation) {
            ProjectGenerationMatch::Current => {}
            ProjectGenerationMatch::Superseded {
                newer_same_project_generation: true,
            } => {
                self.active_scene_reload_admission = None;
                if let Err(error) = self.submit_active_scene_reload(None) {
                    self.set_status_line(error);
                }
                return;
            }
            ProjectGenerationMatch::Superseded {
                newer_same_project_generation: false,
            } => {
                self.active_scene_reload_admission = None;
                return;
            }
        }
        let Some(retry_not_before) = retry_not_before else {
            return;
        };
        if Instant::now() < retry_not_before {
            return;
        }
        let admission = self
            .active_scene_reload_admission
            .take()
            .expect("an elapsed active-scene reload retry remains present");
        if let Err(error) = self.submit_active_scene_reload(Some(admission)) {
            self.set_status_line(error);
        }
    }

    fn defer_active_scene_reload_after_admission_failure(
        &mut self,
        identity: ActiveSceneDocumentIdentity,
        generation: ProjectAssetGenerationToken,
        previous_admission: Option<ActiveSceneReloadAdmissionState>,
        error: JobSubmitError,
    ) -> Result<(), String> {
        let previous_failures = previous_admission
            .filter(|admission| {
                admission.identity == identity && admission.generation == generation
            })
            .map(|admission| admission.consecutive_failures);
        let Some((consecutive_failures, delay)) =
            next_active_scene_reload_admission_retry(previous_failures)
        else {
            self.active_scene_reload_admission = Some(ActiveSceneReloadAdmissionState {
                identity,
                generation,
                consecutive_failures: previous_failures.unwrap_or_default().saturating_add(1),
                retry_not_before: None,
            });
            zircon_runtime::profile_counter!(
                "editor",
                "ui.asset_refresh.active_scene_reload_admission_retry_exhausted",
                1
            );
            return Err(format!(
                "{error}; active scene reload admission retry limit ({ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_LIMIT}) was exhausted"
            ));
        };

        let now = Instant::now();
        let not_before = now.checked_add(delay).unwrap_or(now);
        self.active_scene_reload_admission = Some(ActiveSceneReloadAdmissionState {
            identity,
            generation,
            consecutive_failures,
            retry_not_before: Some(not_before),
        });
        self.ui.schedule_maintenance_frame_update(not_before);
        zircon_runtime::profile_counter!(
            "editor",
            "ui.asset_refresh.active_scene_reload_admission_retry_scheduled",
            1
        );
        Err(format!(
            "{error}; active scene reload admission retry {consecutive_failures}/{ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_LIMIT} is scheduled after {} ms",
            delay.as_millis()
        ))
    }

    fn complete_active_scene_reload(
        &mut self,
        pending: PendingActiveSceneReload,
        document: ProjectSceneDocument,
    ) -> Result<ActiveSceneReloadOutcome, String> {
        let project_asset_manager = self
            .asset_runtime_access
            .project_asset_manager()
            .map_err(|error| error.to_string())?;
        match project_asset_manager.check_project_generation(&pending.generation) {
            ProjectGenerationMatch::Current => {}
            ProjectGenerationMatch::Superseded {
                newer_same_project_generation: true,
            } => return Ok(ActiveSceneReloadOutcome::Superseded),
            ProjectGenerationMatch::Superseded {
                newer_same_project_generation: false,
            } => return Ok(ActiveSceneReloadOutcome::Discarded),
        }
        if document.scene_uri().to_string() != pending.identity.scene_uri() {
            return Err("prepared scene reload does not match the active scene source".to_owned());
        }
        let authoring_world = {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "active_scene_authoring_prepare"
            );
            self.editor_manager
                .prepare_authoring_world(document.into_world())
                .map_err(|error| error.to_string())?
        };
        match self.runtime.commit_prepared_active_scene_reload(
            project_asset_manager.as_ref(),
            &pending.generation,
            pending.identity.clone(),
            authoring_world,
            pending.dirty_policy,
        )? {
            PreparedActiveSceneReloadOutcome::Reloaded => Ok(ActiveSceneReloadOutcome::Committed),
            PreparedActiveSceneReloadOutcome::Superseded => Ok(ActiveSceneReloadOutcome::Discarded),
            PreparedActiveSceneReloadOutcome::Conflict => Ok(ActiveSceneReloadOutcome::Conflict {
                identity: pending.identity,
                generation: pending.generation,
            }),
            PreparedActiveSceneReloadOutcome::ProjectGenerationSuperseded {
                newer_same_project_generation: true,
            } => Ok(ActiveSceneReloadOutcome::Superseded),
            PreparedActiveSceneReloadOutcome::ProjectGenerationSuperseded {
                newer_same_project_generation: false,
            } => Ok(ActiveSceneReloadOutcome::Discarded),
        }
    }

    fn queue_active_scene_reload_retry(&mut self) {
        let deadline = self
            .asset_refresh_accumulator
            .request_active_scene_reload(std::time::Instant::now());
        self.ui.schedule_maintenance_frame_update(deadline);
    }

    pub(in crate::ui::retained_host::app) fn cancel_pending_active_scene_reload(&mut self) {
        if let Some(pending) = self.pending_active_scene_reload.take() {
            let _ = self
                .editor_manager
                .context()
                .jobs()
                .cancel(pending.ticket.id());
        }
        self.active_scene_reload_admission = None;
        self.dismiss_active_scene_reload_conflict();
    }

    pub(in crate::ui::retained_host::app) fn import_model_into_project(
        &mut self,
    ) -> Result<(), String> {
        if self.pending_asset_deletion.is_some() {
            return Err("an asset deletion is already running".to_owned());
        }
        if self.pending_asset_relocation.is_some() {
            return Err("an asset move is already running".to_owned());
        }
        if self.pending_model_import.is_some() {
            return Err("a model import is already running".to_owned());
        }
        let chrome = self.build_chrome();
        let source = canonical_model_source_path(&chrome.mesh_import_path)
            .map_err(|error| error.to_string())?;
        let ticket = self
            .editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .submit_model_import(source.clone())
            .map_err(|error| error.to_string())?;
        let display_path = ProjectPaths::display_path(&source)
            .to_string_lossy()
            .into_owned();
        self.pending_model_import = Some(PendingModelImport {
            ticket,
            display_path,
            close_requested: false,
        });
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn poll_model_import(&mut self) {
        let Some(pending) = self.pending_model_import.take() else {
            return;
        };
        let Some(result) = pending.ticket.try_take() else {
            self.pending_model_import = Some(pending);
            return;
        };
        if pending.close_requested {
            if let Err(error) = self.commit_project_close() {
                self.set_status_line(error.to_string());
            }
            return;
        }
        let completion = match result {
            Ok(receipt) => self.complete_model_import(receipt, pending.display_path),
            Err(error) => Err(error.to_string()),
        };
        match completion {
            Ok(()) => {
                if let Some(notification) = import_model_completed_toast() {
                    self.publish_activity_toasts(std::slice::from_ref(&notification));
                }
            }
            Err(error) => {
                let error = error.to_string();
                if let Some(notification) = import_model_failed_toast(&error) {
                    self.publish_activity_toasts(std::slice::from_ref(&notification));
                }
                self.set_status_line(error);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn cancel_pending_model_import(&mut self) -> bool {
        let Some(mut pending) = self.pending_model_import.take() else {
            return true;
        };
        let _ = self
            .editor_manager
            .context()
            .jobs()
            .cancel(pending.ticket.id());
        if pending.ticket.try_take().is_some() {
            return true;
        }
        pending.close_requested = true;
        self.pending_model_import = Some(pending);
        false
    }

    fn complete_model_import(
        &mut self,
        receipt: ProjectImportReceipt,
        display_path: String,
    ) -> Result<(), String> {
        // The Runtime transaction deliberately filters its own watcher echoes. Project the
        // committed receipt here so a successful model import is visible without relying on a
        // second scan or on filesystem timing.
        self.editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .refresh_from_runtime_project()
            .map_err(|error| error.to_string())?;
        let model_uri = receipt.source_uri().clone();
        let resource_manager = self
            .resolve_resource_manager()
            .map_err(|error| error.to_string())?;
        let model_id = resolve_ready_handle::<ModelMarker>(resource_manager.as_ref(), &model_uri)?;
        let material_uri = ResourceLocator::parse("res://materials/default.zmaterial")
            .map_err(|error| error.to_string())?;
        let material_id =
            resolve_ready_handle::<MaterialMarker>(resource_manager.as_ref(), &material_uri)?;
        if self
            .runtime
            .import_mesh_asset(model_id, material_id, display_path)
            .map_err(|error| error.to_string())?
        {
            self.mark_render_and_presentation_dirty();
        } else {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        }
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_workspace(&mut self) {
        if let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() {
            let _ = editor_asset_manager.refresh_from_runtime_project();
        }
        self.sync_asset_catalog();
        self.sync_asset_resources();
        self.refresh_selected_asset_details();
        self.refresh_visible_asset_previews();
    }
}

fn active_scene_reload_retry_delay(consecutive_failures: u8) -> Duration {
    let exponent = u32::from(consecutive_failures.saturating_sub(1).min(2));
    ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_BASE_DELAY
        .checked_mul(1_u32 << exponent)
        .unwrap_or(Duration::from_millis(256))
}

fn next_active_scene_reload_admission_retry(
    previous_failures: Option<u8>,
) -> Option<(u8, Duration)> {
    let consecutive_failures = previous_failures.map_or(1, |failures| failures.saturating_add(1));
    (consecutive_failures <= ACTIVE_SCENE_RELOAD_ADMISSION_RETRY_LIMIT).then(|| {
        (
            consecutive_failures,
            active_scene_reload_retry_delay(consecutive_failures),
        )
    })
}

#[cfg(test)]
mod active_scene_reload_retry_tests {
    use std::time::Duration;

    use super::next_active_scene_reload_admission_retry;

    #[test]
    fn admission_retry_backs_off_three_times_then_terminates() {
        assert_eq!(
            next_active_scene_reload_admission_retry(None),
            Some((1, Duration::from_millis(64)))
        );
        assert_eq!(
            next_active_scene_reload_admission_retry(Some(1)),
            Some((2, Duration::from_millis(128)))
        );
        assert_eq!(
            next_active_scene_reload_admission_retry(Some(2)),
            Some((3, Duration::from_millis(256)))
        );
        assert_eq!(next_active_scene_reload_admission_retry(Some(3)), None);
    }
}
