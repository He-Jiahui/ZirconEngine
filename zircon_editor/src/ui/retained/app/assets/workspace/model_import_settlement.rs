//! Host settlement of a Runtime-owned durable model import. Retrying this state
//! never resubmits the import job and never replays a started Scene command.
use super::*;
use crate::core::notifications::{
    DecisionNotification, DecisionOption, DecisionOptionId, DecisionTicket, NotificationId,
    NotificationSource, MAX_DECISION_DISPLAY_SUBJECT_BYTES,
};
use crate::ui::host::editor_asset_manager::EditorModelImportProjection;
use crate::ui::host::ProjectCloseCoordinatorPhase;
use crate::ui::retained_host::app::project_close::RetainedProjectCloseError;
use zircon_runtime::asset::pipeline::manager::ProjectGenerationCommitOutcome;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModelImportSettlementPhase {
    ProjectionPending,
    ProjectionRetryRequired,
    RepairRequired,
    Superseded,
    CloseRetryRequired,
    CloseRepairRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelImportCloseResumePreflight {
    PendingDocumentSave,
    QueuedDocumentSaveAll,
}

impl ModelImportCloseResumePreflight {
    fn from_error(error: &RetainedProjectCloseError) -> Option<Self> {
        match error {
            RetainedProjectCloseError::PendingDocumentSave { .. } => Some(Self::PendingDocumentSave),
            RetainedProjectCloseError::QueuedDocumentSaveAll => Some(Self::QueuedDocumentSaveAll),
            _ => None,
        }
    }
}

pub(super) struct ModelImportSettlement {
    pub(super) receipt: Option<ProjectImportReceipt>,
    pub(super) phase: ModelImportSettlementPhase,
    resume_phase: ModelImportSettlementPhase,
    close_resume_preflight: Option<ModelImportCloseResumePreflight>,
    close_may_have_started: bool,
    pub(super) decision: Option<DecisionTicket>,
    decision_sequence: u64,
    decision_attempted: bool,
    pub(super) scene_apply_started: bool,
    pub(super) error: String,
}

impl ModelImportSettlement {
    fn new(receipt: Option<ProjectImportReceipt>) -> Self {
        Self {
            receipt,
            phase: ModelImportSettlementPhase::ProjectionPending,
            resume_phase: ModelImportSettlementPhase::ProjectionPending,
            close_resume_preflight: None,
            close_may_have_started: false,
            decision: None,
            decision_sequence: 0,
            decision_attempted: false,
            scene_apply_started: false,
            error: String::new(),
        }
    }

    pub(super) fn begin_scene_apply(&mut self) -> bool {
        if self.scene_apply_started || self.phase != ModelImportSettlementPhase::ProjectionPending {
            return false;
        }
        // Install the non-replay fence before invoking the real command.
        self.scene_apply_started = true;
        self.phase = ModelImportSettlementPhase::RepairRequired;
        true
    }

    pub(super) fn run_scene_apply<F>(
        &mut self, blocked: String, panicked: String, apply: F,
    ) -> Result<bool, String>
    where F: FnOnce() -> Result<bool, String> {
        if !self.begin_scene_apply() { return Err(blocked); }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(apply))
            .unwrap_or_else(|_| Err(panicked));
        if let Err(error) = &result {
            self.fail(ModelImportSettlementPhase::RepairRequired, error.clone());
        }
        result
    }

    fn fail(&mut self, phase: ModelImportSettlementPhase, error: String) {
        self.phase = phase;
        self.error = error;
        self.decision_attempted = false;
    }

    fn message_key(&self) -> &'static str {
        match self.phase {
            ModelImportSettlementPhase::ProjectionPending
            | ModelImportSettlementPhase::ProjectionRetryRequired =>
                "editor.asset.model_import.projection_pending",
            ModelImportSettlementPhase::Superseded => "editor.asset.model_import.superseded",
            ModelImportSettlementPhase::CloseRetryRequired =>
                "editor.asset.model_import.close_pending",
            ModelImportSettlementPhase::RepairRequired
            | ModelImportSettlementPhase::CloseRepairRequired =>
                "editor.asset.model_import.repair_required",
        }
    }
}

enum ModelImportProjectionAttempt {
    Inserted,
    Retry(String),
    Repair(String),
    Superseded,
}

impl RetainedEditorHost {
    fn model_import_text(&self, key: &str) -> String {
        self.runtime.context().i18n().translate(key).to_string()
    }

    pub(in crate::ui::retained_host::app) fn import_model_into_project(
        &mut self,
    ) -> Result<(), String> {
        if let Some(mut pending) = self.pending_model_import.take() {
            if pending.settlement.is_some() {
                // ImportModel reviews this attempt; it is never an implicit Retry or a new job.
                self.dismiss_model_import_decision(&mut pending);
                if let Some(settlement) = pending.settlement.as_mut() {
                    settlement.decision_attempted = false;
                }
                self.publish_model_import_decision(&mut pending);
                self.pending_model_import = Some(pending);
                return Ok(());
            }
            self.pending_model_import = Some(pending);
            return Err(self.model_import_text("editor.asset.model_import.busy"));
        }
        let chrome = self.build_chrome().map_err(|error| error.to_string())?;
        let source = canonical_model_source_path(&chrome.mesh_import_path)
            .map_err(|error| error.to_string())?;
        self.submit_model_import_source(source)
    }

    /// Shared by the real picker route and its integration fixture.
    pub(super) fn submit_model_import_source(
        &mut self,
        source: std::path::PathBuf,
    ) -> Result<(), String> {
        if self.pending_asset_deletion.is_some() || self.pending_asset_relocation.is_some()
            || self.pending_model_import.is_some()
        {
            return Err(self.model_import_text("editor.asset.model_import.busy"));
        }
        let identity = self.editor_manager.enter().map_err(|error| error.to_string())?
            .active_scene_identity_for_session()
            .ok_or_else(|| self.model_import_text("editor.asset.model_import.no_scene"))?;
        let ticket = self.editor_asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .submit_model_import(source.clone()).map_err(|error| error.to_string())?;
        let display_path = ProjectPaths::display_path(&source).to_string_lossy().into_owned();
        self.pending_model_import = Some(PendingModelImport {
            ticket, display_path, identity, close_requested: false, close_attempted: false,
            settlement: None,
        });
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn cancel_pending_model_import(&mut self) -> bool {
        let Some(mut pending) = self.pending_model_import.take() else { return true; };
        if !pending.close_requested {
            pending.close_requested = true;
            pending.close_attempted = false;
            self.dismiss_model_import_decision(&mut pending);
        }
        // Cancellation is advisory. In particular, do not consume a successful ticket here.
        if pending.settlement.is_none() {
            let _ = self.runtime.context().jobs().cancel(pending.ticket.id());
        }
        self.pending_model_import = Some(pending);
        false
    }

    pub(in crate::ui::retained_host::app) fn poll_model_import(&mut self) {
        let Some(mut pending) = self.pending_model_import.take() else { return; };
        if pending.settlement.is_none() {
            let Some(result) = pending.ticket.try_take() else {
                self.pending_model_import = Some(pending);
                return;
            };
            match result {
                Ok(receipt) => {
                    // Retain the durable receipt before any fallible Editor work.
                    pending.settlement = Some(ModelImportSettlement::new(Some(receipt)));
                }
                Err(error) if pending.close_requested => {
                    let mut settlement = ModelImportSettlement::new(None);
                    settlement.error = error.to_string();
                    pending.settlement = Some(settlement);
                }
                Err(error) => {
                    self.report_model_import_worker_failure(error.to_string());
                    return;
                }
            }
        }
        // This reads in-memory session identity only. No projection IO is retried by a tick.
        match self.model_import_origin_is_current(&pending) {
            Ok(true) => {}
            Ok(false) => {
                let already_superseded = pending.settlement.as_ref()
                    .is_some_and(|s| s.phase == ModelImportSettlementPhase::Superseded);
                if !already_superseded {
                    self.dismiss_model_import_decision(&mut pending);
                    let message = self.model_import_text("editor.asset.model_import.superseded");
                    if let Some(settlement) = pending.settlement.as_mut() {
                        settlement.fail(ModelImportSettlementPhase::Superseded, message);
                    }
                }
            }
            Err(error) => {
                // Keep the existing phase, ticket and receipt on transient service refusal.
                self.set_status_line(error);
                self.pending_model_import = Some(pending);
                return;
            }
        }
        // A previously offered Resume cannot revoke a real close operation.
        let close_phase = self.project_close_coordinator.phase();
        let mut stale_close_decision = false;
        if pending.close_requested && close_phase != ProjectCloseCoordinatorPhase::Decision {
            if let Some(s) = pending.settlement.as_mut() {
                s.close_may_have_started = true;
                stale_close_decision = s.close_resume_preflight.take().is_some();
                if !matches!(close_phase, ProjectCloseCoordinatorPhase::Quiescing)
                    && !matches!(s.phase, ModelImportSettlementPhase::Superseded
                        | ModelImportSettlementPhase::CloseRepairRequired)
                {
                    s.phase = ModelImportSettlementPhase::CloseRepairRequired;
                    stale_close_decision = true;
                }
            }
        }
        if stale_close_decision {
            self.dismiss_model_import_decision(&mut pending);
            if let Some(s) = pending.settlement.as_mut() { s.decision_attempted = false; }
        }
        let resume_allowed = self.model_import_close_resume_allowed(&pending);
        let action = self.model_import_decision(&pending);
        if let Some(action) = action {
            self.dismiss_model_import_decision(&mut pending);
            let phase = pending.settlement.as_ref().map(|s| s.phase);
            match (action.as_str(), phase) {
                ("keep", _) => return, // explicit acknowledgement; durable evidence remains on disk
                ("retry", Some(ModelImportSettlementPhase::ProjectionRetryRequired))
                    if !pending.close_requested => {
                        if let Some(s) = pending.settlement.as_mut() {
                            s.phase = ModelImportSettlementPhase::ProjectionPending;
                        }
                    }
                ("retry", Some(ModelImportSettlementPhase::CloseRetryRequired))
                    if pending.close_requested => pending.close_attempted = false,
                ("resume", Some(ModelImportSettlementPhase::CloseRetryRequired))
                    if resume_allowed => {
                        pending.close_requested = false;
                        pending.close_attempted = false;
                        if let Some(s) = pending.settlement.as_mut() {
                            s.phase = s.resume_phase;
                            s.close_resume_preflight = None;
                            s.decision_attempted = false;
                        }
                    }
                _ => {} // default, cancel, unknown and stale options never start work
            }
        }
        let phase = pending.settlement.as_ref().map(|s| s.phase);
        let superseded = phase == Some(ModelImportSettlementPhase::Superseded);
        if pending.close_requested && !pending.close_attempted && !superseded {
            pending.close_attempted = true;
            if let Some(s) = pending.settlement.as_mut() {
                if s.phase != ModelImportSettlementPhase::CloseRetryRequired {
                    s.resume_phase = s.phase;
                }
            }
            let began_in_decision = self.project_close_coordinator.phase()
                == ProjectCloseCoordinatorPhase::Decision;
            match self.commit_project_close() {
                Ok(()) => {
                    self.dismiss_model_import_decision(&mut pending);
                    return;
                }
                Err(error) => {
                    // Only Decision/Quiescing can enter the existing close continuation.
                    // Terminal close phases require their existing repair owner.
                    let retryable = matches!(self.project_close_coordinator.phase(),
                        ProjectCloseCoordinatorPhase::Decision | ProjectCloseCoordinatorPhase::Quiescing);
                    let preflight = ModelImportCloseResumePreflight::from_error(&error);
                    let still_decision = self.project_close_coordinator.phase()
                        == ProjectCloseCoordinatorPhase::Decision;
                    if let Some(s) = pending.settlement.as_mut() {
                        let safe_preflight = began_in_decision && still_decision
                            && preflight.is_some() && !s.close_may_have_started;
                        s.close_may_have_started |= !safe_preflight;
                        s.close_resume_preflight = if safe_preflight { preflight } else { None };
                        s.fail(if retryable { ModelImportSettlementPhase::CloseRetryRequired }
                            else { ModelImportSettlementPhase::CloseRepairRequired }, error.to_string());
                    }
                }
            }
        } else if !pending.close_requested
            && phase == Some(ModelImportSettlementPhase::ProjectionPending)
        {
            let has_receipt = pending.settlement.as_ref().is_some_and(|s| s.receipt.is_some());
            if !has_receipt {
                let error = pending.settlement.as_ref().map(|s| s.error.clone()).unwrap_or_default();
                self.report_model_import_worker_failure(error);
                return;
            }
            match self.try_settle_model_import(&mut pending) {
                ModelImportProjectionAttempt::Inserted => {
                    self.dismiss_model_import_decision(&mut pending);
                    if let Some(notification) = import_model_completed_toast() {
                        self.publish_activity_toasts(std::slice::from_ref(&notification));
                    }
                    return;
                }
                ModelImportProjectionAttempt::Retry(error) => {
                    if let Some(s) = pending.settlement.as_mut() {
                        s.fail(ModelImportSettlementPhase::ProjectionRetryRequired, error);
                    }
                }
                ModelImportProjectionAttempt::Repair(error) => {
                    if let Some(s) = pending.settlement.as_mut() {
                        s.fail(ModelImportSettlementPhase::RepairRequired, error);
                    }
                }
                ModelImportProjectionAttempt::Superseded => {
                    let error = self.model_import_text("editor.asset.model_import.superseded");
                    if let Some(s) = pending.settlement.as_mut() {
                        s.fail(ModelImportSettlementPhase::Superseded, error);
                    }
                }
            }
        }
        self.publish_model_import_decision(&mut pending);
        self.pending_model_import = Some(pending);
    }

    fn model_import_close_resume_allowed(&self, pending: &PendingModelImport) -> bool {
        pending.close_requested
            && self.project_close_coordinator.phase() == ProjectCloseCoordinatorPhase::Decision
            && pending.settlement.as_ref().is_some_and(|s|
                s.phase == ModelImportSettlementPhase::CloseRetryRequired
                    && !s.close_may_have_started && s.close_resume_preflight.is_some())
    }

    fn model_import_scene_admission_is_open(&self, pending: &PendingModelImport) -> bool {
        !pending.close_requested
            && self.project_close_coordinator.phase() == ProjectCloseCoordinatorPhase::Decision
            && pending.settlement.as_ref().is_some_and(|s| !s.close_may_have_started)
    }

    fn model_import_origin_is_current(&self, pending: &PendingModelImport) -> Result<bool, String> {
        let access = self.asset_runtime_access.project_asset_manager()
            .map_err(|error| error.to_string())?;
        let Some(snapshot) = access.current_project_generation_snapshot() else { return Ok(false); };
        let (project, _) = snapshot.into_parts();
        if project.activation_identity() != pending.ticket.project_activation_identity() {
            return Ok(false);
        }
        if pending.close_requested {
            return Ok(true);
        }
        let identity = self.editor_manager.enter().map_err(|error| error.to_string())?
            .active_scene_identity_for_session();
        Ok(identity.as_ref() == Some(&pending.identity))
    }

    fn try_settle_model_import(&mut self, pending: &mut PendingModelImport)
        -> ModelImportProjectionAttempt
    {
        use ModelImportProjectionAttempt as Attempt;
        if !self.model_import_scene_admission_is_open(pending) {
            return Attempt::Repair(self.model_import_text("editor.asset.model_import.repair_required"));
        }
        let receipt = match pending.settlement.as_ref().and_then(|s| s.receipt.clone()) {
            Some(receipt) => receipt,
            None => return Attempt::Repair(self.model_import_text("editor.asset.model_import.repair_required")),
        };
        let editor_assets = match self.editor_asset_manager_at_use_point() {
            Ok(manager) => manager,
            Err(error) => return Attempt::Retry(error.to_string()),
        };
        // Only this explicit attempt may perform fallible source projection.
        if let Err(error) = editor_assets.refresh_from_runtime_project() {
            return Attempt::Retry(error.to_string());
        }
        let published = match editor_assets.model_import_projection(
            pending.ticket.project_activation_identity(), &receipt)
        {
            EditorModelImportProjection::Published { generation_sequence } => generation_sequence,
            EditorModelImportProjection::AwaitingPublication =>
                return Attempt::Retry(self.model_import_text("editor.asset.model_import.projection_not_published")),
            EditorModelImportProjection::Superseded => return Attempt::Superseded,
            EditorModelImportProjection::Unsupported =>
                return Attempt::Repair(self.model_import_text("editor.asset.model_import.unsupported_projection")),
        };
        drop(editor_assets);
        let project_assets = match self.asset_runtime_access.project_asset_manager() {
            Ok(manager) => manager,
            Err(error) => return Attempt::Retry(error.to_string()),
        };
        let Some(snapshot) = project_assets.current_project_generation_snapshot() else {
            return Attempt::Superseded;
        };
        let (project, generation) = snapshot.into_parts();
        if project.activation_identity() != pending.ticket.project_activation_identity() {
            return Attempt::Superseded;
        }
        if receipt.committed_records().iter().any(|record| {
            project.registry().get_by_locator(record.primary_locator()) != Some(record)
        }) {
            return Attempt::Superseded;
        }
        if project.catalog_input_generation().sequence() != published {
            return Attempt::Retry(self.model_import_text("editor.asset.model_import.generation_changed"));
        }
        let resources = match self.resolve_resource_manager() {
            Ok(manager) => manager,
            Err(error) => return Attempt::Retry(error.to_string()),
        };
        let model = match resolve_ready_handle::<ModelMarker>(&*resources, receipt.source_uri()) {
            Ok(handle) => handle,
            Err(error) => return Attempt::Retry(error),
        };
        let material_uri = match ResourceLocator::parse("res://materials/default.zmaterial") {
            Ok(uri) => uri,
            Err(error) => return Attempt::Repair(error.to_string()),
        };
        let material = match resolve_ready_handle::<MaterialMarker>(&*resources, &material_uri) {
            Ok(handle) => handle,
            Err(error) => return Attempt::Retry(error),
        };
        drop(resources);
        match self.model_import_origin_is_current(pending) {
            Ok(true) => {}
            Ok(false) => return Attempt::Superseded,
            Err(error) => return Attempt::Retry(error),
        }
        let display_path = pending.display_path.clone();
        let result = project_assets.commit_if_project_generation(&generation, || {
            if !self.model_import_scene_admission_is_open(pending) {
                return Err(self.model_import_text("editor.asset.model_import.repair_required"));
            }
            let Some(settlement) = pending.settlement.as_mut() else {
                return Err(self.model_import_text("editor.asset.model_import.repair_required"));
            };
            settlement.run_scene_apply(
                self.model_import_text("editor.asset.model_import.repair_required"),
                self.model_import_text("editor.asset.model_import.scene_apply_panicked"),
                || self.runtime.import_mesh_asset(model, material, display_path)
                    .map_err(|error| error.to_string()),
            )
        });
        match result {
            ProjectGenerationCommitOutcome::Superseded { .. } =>
                Attempt::Retry(self.model_import_text("editor.asset.model_import.generation_changed")),
            ProjectGenerationCommitOutcome::Committed(Err(error)) => Attempt::Repair(error),
            ProjectGenerationCommitOutcome::Committed(Ok(changed)) => {
                if changed { self.mark_render_and_presentation_dirty(); }
                else { self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA); }
                Attempt::Inserted
            }
        }
    }

    fn report_model_import_worker_failure(&mut self, error: String) {
        if let Some(notification) = import_model_failed_toast(&error) {
            self.publish_activity_toasts(std::slice::from_ref(&notification));
        }
        self.set_status_line(error);
    }

    fn model_import_decision(&self, pending: &PendingModelImport) -> Option<String> {
        let ticket = pending.settlement.as_ref()?.decision.as_ref()?;
        let center = self.runtime.context().notifications().decisions().ok()?;
        center.snapshot().into_iter().find(|snapshot| snapshot.ticket() == ticket)
            .and_then(|snapshot| snapshot.resolved()
                .map(|receipt| receipt.option_id().as_str().to_owned()))
    }

    fn dismiss_model_import_decision(&mut self, pending: &mut PendingModelImport) {
        if let Some(ticket) = pending.settlement.as_mut().and_then(|s| s.decision.take()) {
            if let Ok(center) = self.runtime.context().notifications().decisions() {
                let _ = center.cancel(&ticket);
            }
            self.refresh_activity_notification_presentation();
        }
    }

    fn publish_model_import_decision(&mut self, pending: &mut PendingModelImport) {
        let resume_allowed = self.model_import_close_resume_allowed(pending);
        let Some(s) = pending.settlement.as_mut() else { return; };
        if s.decision.is_some() || s.decision_attempted { return; }
        s.decision_attempted = true; // capacity/service/expiry never causes a tick retry
        let Some(sequence) = s.decision_sequence.checked_add(1) else {
            self.set_status_line(s.error.clone());
            return;
        };
        s.decision_sequence = sequence;
        let publish = || -> Result<DecisionTicket, String> {
            let mut options = Vec::new();
            let retry = match s.phase {
                ModelImportSettlementPhase::ProjectionRetryRequired =>
                    Some("editor.asset.model_import.retry_projection"),
                ModelImportSettlementPhase::CloseRetryRequired =>
                    Some("editor.asset.model_import.retry_close"),
                _ => None,
            };
            if let Some(label) = retry {
                options.push(DecisionOption::new(DecisionOptionId::parse("retry")
                    .map_err(|error| error.to_string())?, label).map_err(|error| error.to_string())?);
            }
            if resume_allowed {
                options.push(DecisionOption::new(DecisionOptionId::parse("resume")
                    .map_err(|error| error.to_string())?, "editor.asset.model_import.resume")
                    .map_err(|error| error.to_string())?);
            }
            let keep = DecisionOptionId::parse("keep").map_err(|error| error.to_string())?;
            let defer = DecisionOptionId::parse("defer").map_err(|error| error.to_string())?;
            options.push(DecisionOption::new(keep, "editor.asset.model_import.keep")
                .map_err(|error| error.to_string())?);
            options.push(DecisionOption::new(defer.clone(), "editor.asset.model_import.defer")
                .map_err(|error| error.to_string())?);
            let mut subject = String::new();
            for ch in pending.display_path.chars() {
                if subject.len() + ch.len_utf8() > MAX_DECISION_DISPLAY_SUBJECT_BYTES { break; }
                subject.push(ch);
            }
            let notification = DecisionNotification::new(
                NotificationId::parse(format!("editor.asset.model_import.{}.{}",
                    pending.ticket.id().value(), sequence)).map_err(|error| error.to_string())?,
                NotificationSource::builtin("editor.assets").map_err(|error| error.to_string())?,
                "editor.asset.model_import.review.title", s.message_key(), options)
                .map_err(|error| error.to_string())?
                .with_default_option(defer.clone()).map_err(|error| error.to_string())?
                .with_cancel_option(defer).map_err(|error| error.to_string())?
                .with_display_subject(subject).map_err(|error| error.to_string())?;
            self.runtime.context().notifications().decisions().map_err(|error| error.to_string())?
                .publish(notification).map_err(|error| error.to_string())
        };
        match publish() {
            Ok(ticket) => { s.decision = Some(ticket); self.refresh_activity_notification_presentation(); }
            Err(error) => self.set_status_line(error),
        }
        if !s.error.is_empty() { self.set_status_line(s.error.clone()); }
    }
}
