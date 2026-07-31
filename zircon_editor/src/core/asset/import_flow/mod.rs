//! Editor-owned orchestration for runtime asset imports.

mod error;
mod flight;
mod job;
mod state;
mod submit;

use std::fmt;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use zircon_runtime::asset::{AssetManager, AssetStatusRecord, AssetUri};
use zircon_runtime::core::CoreError;

use crate::core::asset::EditorAssetIndex;
use crate::core::jobs::{EditorJobSystem, JobError, JobId};

pub use error::EditorAssetImportSubmitError;

use self::flight::{ImportFlight, SharedImportReasons};
use self::state::ImportFlowSharedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EditorAssetImportReason {
    Watch,
    DigestMismatch,
    Manual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorAssetImportRequest {
    uri: Arc<AssetUri>,
    reason: EditorAssetImportReason,
}

impl EditorAssetImportRequest {
    pub fn new(uri: AssetUri, reason: EditorAssetImportReason) -> Self {
        Self {
            uri: Arc::new(uri),
            reason,
        }
    }

    pub fn uri(&self) -> &AssetUri {
        self.uri.as_ref()
    }

    pub fn reason(&self) -> EditorAssetImportReason {
        self.reason
    }

    fn shared_uri(&self) -> &Arc<AssetUri> {
        &self.uri
    }
}

#[derive(Clone, Debug)]
pub struct EditorAssetImportResult {
    uri: Arc<AssetUri>,
    reasons: Arc<SharedImportReasons>,
    status: Option<AssetStatusRecord>,
}

impl EditorAssetImportResult {
    fn new(
        uri: Arc<AssetUri>,
        reasons: Arc<SharedImportReasons>,
        status: Option<AssetStatusRecord>,
    ) -> Self {
        Self {
            uri,
            reasons,
            status,
        }
    }

    pub fn uri(&self) -> &AssetUri {
        self.uri.as_ref()
    }

    pub fn reasons(&self) -> Vec<EditorAssetImportReason> {
        self.reasons.snapshot()
    }

    pub fn status(&self) -> Option<&AssetStatusRecord> {
        self.status.as_ref()
    }

    fn estimated_retained_bytes(&self) -> usize {
        let status_bytes = self.status.as_ref().map_or(0, |status| {
            status
                .id
                .len()
                .saturating_add(status.uri.len())
                .saturating_add(
                    status
                        .artifact_uri
                        .as_deref()
                        .map(str::len)
                        .unwrap_or_default(),
                )
                .saturating_add(status.source_hash.len())
                .saturating_add(status.importer_id.len())
                .saturating_add(status.config_hash.len())
        });
        mem::size_of::<Self>()
            .saturating_add(status_bytes)
            .saturating_add(
                self.reasons
                    .len()
                    .saturating_mul(mem::size_of::<EditorAssetImportReason>()),
            )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorAssetImportAdmissionLimits {
    pub(super) max_flights: usize,
    pub(super) max_estimated_bytes: usize,
    pub(super) max_oldest_age: Duration,
}

impl EditorAssetImportAdmissionLimits {
    pub const fn new(
        max_flights: usize,
        max_estimated_bytes: usize,
        max_oldest_age: Duration,
    ) -> Self {
        Self {
            max_flights,
            max_estimated_bytes,
            max_oldest_age,
        }
    }
}

impl Default for EditorAssetImportAdmissionLimits {
    fn default() -> Self {
        Self::new(4_096, 4 * 1024 * 1024, Duration::from_secs(5 * 60))
    }
}

#[derive(Clone)]
pub struct EditorAssetImportTicket {
    id: JobId,
    flight: Arc<ImportFlight>,
}

impl EditorAssetImportTicket {
    pub fn id(&self) -> JobId {
        self.id
    }

    pub fn uri(&self) -> &AssetUri {
        self.flight.uri().as_ref()
    }

    pub fn try_result(&self) -> Option<Result<EditorAssetImportResult, JobError>> {
        self.flight.try_result()
    }

    pub fn wait(&self) -> Result<EditorAssetImportResult, JobError> {
        self.flight.wait()
    }
}

impl fmt::Debug for EditorAssetImportTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorAssetImportTicket")
            .field("uri", &self.uri())
            .field("job_id", &self.id())
            .finish()
    }
}

trait AssetImportBackend: Send + Sync {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError>;
}

struct RuntimeAssetImportBackend {
    manager: Arc<dyn AssetManager>,
}

impl AssetImportBackend for RuntimeAssetImportBackend {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        self.manager.import_asset(&uri.to_string())
    }
}

#[derive(Clone)]
pub struct EditorAssetImportFlow {
    jobs: EditorJobSystem,
    backend: Arc<dyn AssetImportBackend>,
    index: Arc<Mutex<EditorAssetIndex>>,
    state: Arc<ImportFlowSharedState>,
    limits: EditorAssetImportAdmissionLimits,
    #[cfg(test)]
    before_generation_validate: Option<Arc<dyn Fn() + Send + Sync>>,
    #[cfg(test)]
    before_job_submit: Option<Arc<dyn Fn() + Send + Sync>>,
    #[cfg(test)]
    before_wait_admission: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl EditorAssetImportFlow {
    pub fn new(
        jobs: EditorJobSystem,
        manager: Arc<dyn AssetManager>,
        index: Arc<Mutex<EditorAssetIndex>>,
    ) -> Self {
        Self::new_with_limits(
            jobs,
            manager,
            index,
            EditorAssetImportAdmissionLimits::default(),
        )
    }

    pub fn new_with_limits(
        jobs: EditorJobSystem,
        manager: Arc<dyn AssetManager>,
        index: Arc<Mutex<EditorAssetIndex>>,
        limits: EditorAssetImportAdmissionLimits,
    ) -> Self {
        Self::from_backend(
            jobs,
            Arc::new(RuntimeAssetImportBackend { manager }),
            index,
            limits,
        )
    }

    #[cfg(test)]
    fn with_backend<B>(
        jobs: EditorJobSystem,
        backend: Arc<B>,
        index: Arc<Mutex<EditorAssetIndex>>,
    ) -> Self
    where
        B: AssetImportBackend + 'static,
    {
        Self::from_backend(
            jobs,
            backend,
            index,
            EditorAssetImportAdmissionLimits::default(),
        )
    }

    #[cfg(test)]
    fn with_backend_and_limits<B>(
        jobs: EditorJobSystem,
        backend: Arc<B>,
        index: Arc<Mutex<EditorAssetIndex>>,
        limits: EditorAssetImportAdmissionLimits,
    ) -> Self
    where
        B: AssetImportBackend + 'static,
    {
        Self::from_backend(jobs, backend, index, limits)
    }

    fn from_backend(
        jobs: EditorJobSystem,
        backend: Arc<dyn AssetImportBackend>,
        index: Arc<Mutex<EditorAssetIndex>>,
        limits: EditorAssetImportAdmissionLimits,
    ) -> Self {
        Self {
            jobs,
            backend,
            index,
            state: Arc::new(ImportFlowSharedState::default()),
            limits,
            #[cfg(test)]
            before_generation_validate: None,
            #[cfg(test)]
            before_job_submit: None,
            #[cfg(test)]
            before_wait_admission: None,
        }
    }

    #[cfg(test)]
    fn with_before_generation_validate(mut self, hook: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.before_generation_validate = Some(hook);
        self
    }

    #[cfg(test)]
    fn with_before_job_submit(mut self, hook: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.before_job_submit = Some(hook);
        self
    }

    #[cfg(test)]
    fn with_before_wait_admission(mut self, hook: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.before_wait_admission = Some(hook);
        self
    }
}

#[cfg(test)]
mod tests;
