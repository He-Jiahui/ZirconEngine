use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::asset::EditorAssetIndex;
use crate::core::jobs::{EditorJob, JobContext, JobError};
use zircon_runtime::asset::{AssetManager, ProjectImportReceipt};

use super::diagnostics::{EditorAssetImportFlightDiagnostics, EditorModelImportDiagnostics};
use super::flight::ImportFlight;
use super::lock::lock_editor_asset_index_recovering_poison;
use super::state::{
    FlightIdentity, ImportFinishAction, ImportFlowSharedState, ImportGenerationKey,
};
use super::{
    AssetImportBackend, EditorAssetImportAdmissionLimits, EditorAssetImportExecutionError,
    EditorAssetImportRequest, EditorAssetImportResult,
};

pub(super) struct ImportLease {
    state: Arc<ImportFlowSharedState>,
    index: Arc<Mutex<EditorAssetIndex>>,
    key: ImportGenerationKey,
    flight_identity: FlightIdentity,
    flight: Arc<ImportFlight>,
    diagnostics: Arc<EditorAssetImportFlightDiagnostics>,
    limits: EditorAssetImportAdmissionLimits,
    finished: bool,
}

impl ImportLease {
    pub(super) fn new(
        state: Arc<ImportFlowSharedState>,
        index: Arc<Mutex<EditorAssetIndex>>,
        key: ImportGenerationKey,
        flight_identity: FlightIdentity,
        flight: Arc<ImportFlight>,
        diagnostics: Arc<EditorAssetImportFlightDiagnostics>,
        limits: EditorAssetImportAdmissionLimits,
    ) -> Self {
        Self {
            state,
            index,
            key,
            flight_identity,
            flight,
            diagnostics,
            limits,
            finished: false,
        }
    }

    fn finish(&mut self, result: Result<EditorAssetImportResult, JobError>) {
        if self.finished {
            return;
        }
        self.finished = true;
        let successful = result.is_ok();
        let completed_result_bytes = result
            .as_ref()
            .map(EditorAssetImportResult::estimated_retained_bytes)
            .unwrap_or_default();
        let action = self.state.finish(
            &self.key,
            self.flight_identity,
            successful,
            completed_result_bytes,
            self.limits,
        );
        if let ImportFinishAction::ClearUuid(token) = action {
            lock_editor_asset_index_recovering_poison(self.index.as_ref())
                .clear_import(token.uuid());
            self.state.complete_uuid_clear(token);
        }
        self.diagnostics.project_result(result.clone());
        self.flight.complete(result);
    }
}

impl Drop for ImportLease {
    fn drop(&mut self) {
        if !self.finished {
            self.finish(Err(JobError::Cancelled));
        }
    }
}

pub(super) struct AssetImportJob {
    request: EditorAssetImportRequest,
    backend: Arc<dyn AssetImportBackend>,
    lease: ImportLease,
}

pub(super) struct AssetImportModelJob {
    manager: Arc<dyn AssetManager>,
    source_path: PathBuf,
    diagnostics: Arc<EditorModelImportDiagnostics>,
    terminal_projected: bool,
}

impl AssetImportModelJob {
    pub(super) fn new(
        manager: Arc<dyn AssetManager>,
        source_path: PathBuf,
        diagnostics: Arc<EditorModelImportDiagnostics>,
    ) -> Self {
        Self {
            manager,
            source_path,
            diagnostics,
            terminal_projected: false,
        }
    }
}

impl Drop for AssetImportModelJob {
    fn drop(&mut self) {
        if !self.terminal_projected {
            self.diagnostics.project_result(Err(JobError::Cancelled));
        }
    }
}

impl EditorJob for AssetImportModelJob {
    type Output = ProjectImportReceipt;

    fn run(mut self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Importing {}", self.source_path.display()));
        let result = match catch_unwind(AssertUnwindSafe(|| {
            self.manager.import_model_source(&self.source_path)
        })) {
            Ok(result) => result.map_err(JobError::failed),
            Err(payload) => Err(JobError::Panicked(panic_message(payload))),
        };
        if result.is_ok() {
            // The Runtime receipt means the transaction is already durable; a later cancellation
            // cannot undo it and must not hide the catalog refresh handoff.
            context.report_progress(1, 1, format!("Imported {}", self.source_path.display()));
        }
        self.terminal_projected = true;
        self.diagnostics.project_result(result.clone());
        result
    }
}

impl AssetImportJob {
    pub(super) fn new(
        request: EditorAssetImportRequest,
        backend: Arc<dyn AssetImportBackend>,
        lease: ImportLease,
    ) -> Self {
        Self {
            request,
            backend,
            lease,
        }
    }

    fn run_import(&self, context: &JobContext) -> Result<EditorAssetImportResult, JobError> {
        context.check_cancelled()?;
        context.report_progress(0, 1, format!("Importing {}", self.request.uri()));
        let status =
            match catch_unwind(AssertUnwindSafe(|| self.backend.import(self.request.uri()))) {
                Ok(result) => result.map_err(JobError::failed)?,
                Err(payload) => return Err(JobError::Panicked(panic_message(payload))),
            }
            .ok_or_else(|| {
                JobError::failed(EditorAssetImportExecutionError::RuntimeDidNotCommit {
                    uri: self.request.uri().clone(),
                })
            })?;
        context.check_cancelled()?;
        context.report_progress(1, 1, format!("Imported {}", self.request.uri()));
        Ok(EditorAssetImportResult::new(
            Arc::clone(self.request.shared_uri()),
            Arc::clone(self.lease.flight.reasons()),
            status,
        ))
    }
}

impl EditorJob for AssetImportJob {
    type Output = EditorAssetImportResult;

    fn run(mut self, context: JobContext) -> Result<Self::Output, JobError> {
        let result = self.run_import(&context);
        self.lease.finish(result.clone());
        result
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string asset import panic payload".to_owned()
    }
}
