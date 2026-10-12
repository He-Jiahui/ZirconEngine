//! 编辑器导入入口按资源身份与源摘要共享同一代任务，合并触发原因并保留有界完成结果；后端负责实际运行时导入，调用端通过票据观察，不自行接管导入标记。
//! Editor-owned orchestration for runtime asset imports.

#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "error.rs"]
mod error;
#[path = "flight.rs"]
mod flight;
#[path = "job.rs"]
mod job;
#[path = "lock.rs"]
mod lock;
#[path = "model_submit.rs"]
mod model_submit;
#[path = "model_ticket.rs"]
mod model_ticket;
#[path = "state.rs"]
mod state;
#[path = "submit.rs"]
mod submit;

use std::fmt;
use std::mem;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime::asset::pipeline::manager::{ProjectAssetManager, PROJECT_ASSET_MANAGER_NAME};
use zircon_runtime::asset::project::ProjectActivationIdentity;
use zircon_runtime::asset::{AssetStatusRecord, AssetUri, ProjectImportReceipt};
use zircon_runtime::core::manager::{resolve_manager_service, ManagerServiceHandle};
use zircon_runtime::core::runtime::ServiceCallGuard;
use zircon_runtime::core::{CoreError, CoreWeak};

use crate::core::asset::EditorAssetIndex;
use crate::core::jobs::{EditorJobSystem, JobError, JobId};
use crate::core::logging::EditorLogService;

pub use error::{EditorAssetImportExecutionError, EditorAssetImportSubmitError};
pub use model_ticket::EditorModelImportTicket;

use self::diagnostics::EditorAssetImportDiagnostics;
use self::flight::{ImportFlight, SharedImportReasons};
use self::state::ImportFlowSharedState;

const DEFAULT_INLINE_GENERATION_REVALIDATIONS: usize = 1;

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
    status: AssetStatusRecord,
}

impl EditorAssetImportResult {
    fn new(
        uri: Arc<AssetUri>,
        reasons: Arc<SharedImportReasons>,
        status: AssetStatusRecord,
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

    pub fn status(&self) -> &AssetStatusRecord {
        &self.status
    }

    fn estimated_retained_bytes(&self) -> usize {
        let status_bytes = self
            .status
            .id
            .len()
            .saturating_add(self.status.uri.len())
            .saturating_add(
                self.status
                    .artifact_uri
                    .as_deref()
                    .map(str::len)
                    .unwrap_or_default(),
            )
            .saturating_add(self.status.source_hash.len())
            .saturating_add(self.status.importer_id.len())
            .saturating_add(self.status.config_hash.len());
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
    pub(super) max_inline_generation_revalidations: usize,
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
            max_inline_generation_revalidations: DEFAULT_INLINE_GENERATION_REVALIDATIONS,
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

    pub fn wait_until(
        &self,
        deadline: Instant,
    ) -> Option<Result<EditorAssetImportResult, JobError>> {
        self.flight.wait_until(deadline)
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

#[derive(Clone)]
struct RegisteredImportSource {
    core: CoreWeak,
    handle: ManagerServiceHandle<ProjectAssetManager>,
    activation_identity: ProjectActivationIdentity,
}

impl RegisteredImportSource {
    fn enter(&self) -> Result<ServiceCallGuard<ProjectAssetManager>, CoreError> {
        let core = self.core.upgrade().ok_or(CoreError::RuntimeUnavailable)?;
        resolve_manager_service(&core, self.handle.clone())
    }

    fn import_asset(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        let manager = self.enter()?;
        manager.import_asset_for_project(&self.activation_identity, &uri.to_string())
    }

    fn import_model_source(&self, source_path: &Path) -> Result<ProjectImportReceipt, CoreError> {
        let manager = self.enter()?;
        manager.import_model_source_for_project(&self.activation_identity, source_path)
    }
}

struct RuntimeAssetImportBackend {
    source: RegisteredImportSource,
}

impl AssetImportBackend for RuntimeAssetImportBackend {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        self.source.import_asset(uri)
    }
}

#[derive(Clone)]
pub struct EditorAssetImportFlow {
    jobs: EditorJobSystem,
    backend: Arc<dyn AssetImportBackend>,
    model_source: Option<RegisteredImportSource>,
    index: Arc<Mutex<EditorAssetIndex>>,
    diagnostics: EditorAssetImportDiagnostics,
    state: Arc<ImportFlowSharedState>,
    limits: EditorAssetImportAdmissionLimits,
    #[cfg(test)]
    before_generation_validate: Option<Arc<dyn Fn() + Send + Sync>>,
    #[cfg(test)]
    before_job_submit: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl EditorAssetImportFlow {
    pub(crate) fn project_activation_identity(&self) -> Option<&ProjectActivationIdentity> {
        self.model_source
            .as_ref()
            .map(|source| &source.activation_identity)
    }

    fn matches_index_activation(&self, index: &EditorAssetIndex) -> bool {
        // Only the private test backend is unbound. Normal constructors require an activation.
        self.project_activation_identity()
            .is_none_or(|expected| index.project_activation_identity() == Some(expected))
    }

    fn inactive_project_error() -> CoreError {
        CoreError::Initialization(
            PROJECT_ASSET_MANAGER_NAME.to_owned(),
            "asset import belongs to an inactive project activation".to_owned(),
        )
    }

    pub fn new(
        jobs: EditorJobSystem,
        core: CoreWeak,
        original: ManagerServiceHandle<ProjectAssetManager>,
        index: Arc<Mutex<EditorAssetIndex>>,
        logs: Arc<EditorLogService>,
    ) -> Result<Self, CoreError> {
        Self::new_with_limits(
            jobs,
            core,
            original,
            index,
            EditorAssetImportAdmissionLimits::default(),
            logs,
        )
    }

    /// Captures one index activation; no admitted manager guard is retained in the queue.
    pub fn new_with_limits(
        jobs: EditorJobSystem,
        core: CoreWeak,
        original: ManagerServiceHandle<ProjectAssetManager>,
        index: Arc<Mutex<EditorAssetIndex>>,
        limits: EditorAssetImportAdmissionLimits,
        logs: Arc<EditorLogService>,
    ) -> Result<Self, CoreError> {
        let activation_identity = lock::lock_editor_asset_index_recovering_poison(index.as_ref())
            .project_activation_identity()
            .cloned()
            .ok_or_else(|| CoreError::ServiceUnavailable("active Editor project".to_owned()))?;
        let source = RegisteredImportSource {
            core,
            handle: original,
            activation_identity,
        };
        let admitted = source.enter()?;
        if !admitted.is_project_activation_current(&source.activation_identity) {
            return Err(Self::inactive_project_error());
        }
        drop(admitted);
        let current = lock::lock_editor_asset_index_recovering_poison(index.as_ref())
            .project_activation_identity()
            == Some(&source.activation_identity);
        if !current {
            return Err(Self::inactive_project_error());
        }
        Ok(Self::from_backend(
            jobs,
            Arc::new(RuntimeAssetImportBackend {
                source: source.clone(),
            }),
            Some(source),
            index,
            limits,
            logs,
        ))
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
            None,
            index,
            EditorAssetImportAdmissionLimits::default(),
            Arc::new(EditorLogService::default()),
        )
    }

    #[cfg(test)]
    fn with_backend_and_logs<B>(
        jobs: EditorJobSystem,
        backend: Arc<B>,
        index: Arc<Mutex<EditorAssetIndex>>,
        logs: Arc<EditorLogService>,
    ) -> Self
    where
        B: AssetImportBackend + 'static,
    {
        Self::from_backend(
            jobs,
            backend,
            None,
            index,
            EditorAssetImportAdmissionLimits::default(),
            logs,
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
        Self::from_backend(
            jobs,
            backend,
            None,
            index,
            limits,
            Arc::new(EditorLogService::default()),
        )
    }

    fn from_backend(
        jobs: EditorJobSystem,
        backend: Arc<dyn AssetImportBackend>,
        model_source: Option<RegisteredImportSource>,
        index: Arc<Mutex<EditorAssetIndex>>,
        limits: EditorAssetImportAdmissionLimits,
        logs: Arc<EditorLogService>,
    ) -> Self {
        Self {
            jobs,
            backend,
            model_source,
            index,
            diagnostics: EditorAssetImportDiagnostics::new(logs),
            state: Arc::new(ImportFlowSharedState::default()),
            limits,
            #[cfg(test)]
            before_generation_validate: None,
            #[cfg(test)]
            before_job_submit: None,
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
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
