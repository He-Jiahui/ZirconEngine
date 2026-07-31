use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex};

use crate::core::asset::EditorAssetIndex;
use crate::core::jobs::{EditorJob, JobContext, JobError};

use super::flight::ImportFlight;
use super::state::{
    FlightIdentity, ImportFinishAction, ImportFlowSharedState, ImportGenerationKey,
};
use super::{
    AssetImportBackend, EditorAssetImportAdmissionLimits, EditorAssetImportRequest,
    EditorAssetImportResult,
};

pub(super) struct ImportLease {
    state: Arc<ImportFlowSharedState>,
    index: Arc<Mutex<EditorAssetIndex>>,
    key: ImportGenerationKey,
    flight_identity: FlightIdentity,
    flight: Arc<ImportFlight>,
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
        limits: EditorAssetImportAdmissionLimits,
    ) -> Self {
        Self {
            state,
            index,
            key,
            flight_identity,
            flight,
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
            self.index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear_import(token.uuid());
            self.state.complete_uuid_clear(token);
        }
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
            };
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
