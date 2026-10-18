use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use zircon_runtime::core::runtime::ServiceHandle;

use zircon_runtime::asset::project::ProjectPaths;

use crate::core::editor_message::DocumentId;
use crate::core::extension::{DocumentSaveReport, SaveReason};
use crate::core::jobs::{
    EditorJob, EditorJobSpec, JobCategory, JobContext, JobError, JobPriority, MutexGroup,
};
use crate::core::recovery::{
    AutosaveDocumentId, AutosaveDocumentRequest, AutosaveExtension, AutosaveJobPolicy,
    AutosaveSnapshot, AutosaveSnapshotProvenance, AutosaveSnapshotSource, AutosaveSourceDigest,
    AutosaveSourcePath,
};

use super::{EditorError, EditorManager};

#[derive(Clone, Debug)]
pub(crate) struct EditorAutosaveDocumentIdentity {
    document: AutosaveDocumentId,
    source_path: AutosaveSourcePath,
    physical_source_path: PathBuf,
}

impl EditorAutosaveDocumentIdentity {
    pub(crate) fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }
}

impl EditorManager {
    pub(crate) fn closing_autosave_document_request(
        &self,
        manager: ServiceHandle<Self>,
        document: DocumentId,
        instance: crate::ui::workbench::view::ViewInstanceId,
        revision: crate::core::editor_event::DocumentCloseRevision,
        identity: EditorAutosaveDocumentIdentity,
        operation: super::ProjectCloseOperation,
        scene: crate::core::editing::engine::SceneCloseAdmission,
        documents: crate::core::extension::DocumentCloseAdmission,
        payloads: Arc<CloseAutosavePayloadBudget>,
    ) -> Result<AutosaveDocumentRequest, EditorError> {
        let save_mutex = document_save_mutex_group(&identity.physical_source_path)?;
        let metadata_bytes = std::mem::size_of::<EditorDocumentAutosaveSource>().saturating_add(
            identity
                .physical_source_path
                .as_os_str()
                .len()
                .saturating_mul(std::mem::size_of::<u16>()),
        );
        Ok(AutosaveDocumentRequest::new(
            identity.document.clone(),
            AutosaveJobPolicy::for_save_mutex(save_mutex),
            Arc::new(EditorDocumentAutosaveSource {
                manager,
                document,
                dirty_generation: revision.external_generation,
                sequence: next_autosave_sequence(),
                source_path: identity.source_path,
                physical_source_path: identity.physical_source_path,
                close: Some(ClosingAutosaveSource {
                    instance,
                    revision,
                    operation,
                    scene,
                    documents,
                    payloads,
                }),
            }),
        )
        .with_estimated_pending_bytes(metadata_bytes))
    }

    pub(crate) fn autosave_document_identity(
        &self,
        document: DocumentId,
        project_root: &Path,
    ) -> Result<EditorAutosaveDocumentIdentity, EditorError> {
        let source_path = self.host.document_autosave_source_path(document)?;
        editor_autosave_document_identity(project_root, source_path)
    }

    pub(crate) fn autosave_document_request(
        &self,
        manager_handle: ServiceHandle<Self>,
        document: DocumentId,
        dirty_generation: u64,
        identity: EditorAutosaveDocumentIdentity,
    ) -> Result<AutosaveDocumentRequest, EditorError> {
        let save_mutex = document_save_mutex_group(&identity.physical_source_path)?;
        Ok(AutosaveDocumentRequest::new(
            identity.document.clone(),
            AutosaveJobPolicy::for_save_mutex(save_mutex),
            Arc::new(EditorDocumentAutosaveSource {
                manager: manager_handle,
                document,
                dirty_generation,
                sequence: next_autosave_sequence(),
                source_path: identity.source_path,
                physical_source_path: identity.physical_source_path,
                close: None,
            }),
        ))
    }
}

pub(super) fn document_save_mutex_group(source_path: &Path) -> Result<MutexGroup, EditorError> {
    let source = ProjectPaths::resolve_path(source_path)
        .map(|path| path.into_operation_path())
        .map_err(|error| EditorError::Project(error.to_string()))?;
    let source = source.to_string_lossy();
    MutexGroup::parse(format!(
        "save_document_{}",
        blake3::hash(source.as_bytes()).to_hex()
    ))
    .map_err(|error| EditorError::Project(error.to_string()))
}

fn editor_autosave_document_identity(
    project_root: &Path,
    source_path: PathBuf,
) -> Result<EditorAutosaveDocumentIdentity, EditorError> {
    let project_root = ProjectPaths::resolve_path(project_root)
        .map_err(|error| EditorError::Project(error.to_string()))?;
    let physical_source_path = if source_path.is_absolute() {
        ProjectPaths::resolve_path(&source_path)
    } else {
        ProjectPaths::resolve_path_from(&project_root, &source_path)
    }
    .map(|path| path.into_operation_path())
    .map_err(|error| EditorError::Project(error.to_string()))?;
    let relative_path = physical_source_path
        .strip_prefix(project_root.operation_path())
        .map_err(|_| {
            EditorError::Project(format!(
                "autosave source {} is outside project root {}",
                physical_source_path.display(),
                project_root.display_path().display()
            ))
        })?;
    let source_path = AutosaveSourcePath::parse(relative_path)
        .map_err(|error| EditorError::Project(error.to_string()))?;
    Ok(EditorAutosaveDocumentIdentity {
        document: AutosaveDocumentId::from_source_path(&source_path),
        source_path,
        physical_source_path,
    })
}

/// A final close admits one payload at a time, including oversize documents.
/// This records actual retained Vec capacity; it does not claim a fixed byte ceiling.
#[derive(Default)]
pub(crate) struct CloseAutosavePayloadBudget {
    occupied: AtomicBool,
    retained_bytes: AtomicUsize,
    peak_bytes: AtomicUsize,
}

impl CloseAutosavePayloadBudget {
    fn acquire(self: &Arc<Self>) -> Result<CloseAutosavePayloadLease, JobError> {
        self.occupied
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                JobError::failed(std::io::Error::other(
                    "a closing payload is still owned by the preceding write",
                ))
            })?;
        Ok(CloseAutosavePayloadLease {
            budget: Arc::clone(self),
        })
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        self.retained_bytes.load(Ordering::Acquire)
    }

    pub(crate) fn peak_bytes(&self) -> usize {
        self.peak_bytes.load(Ordering::Acquire)
    }
}

struct CloseAutosavePayloadLease {
    budget: Arc<CloseAutosavePayloadBudget>,
}

impl CloseAutosavePayloadLease {
    fn account(&self, capacity: usize) {
        self.budget
            .retained_bytes
            .store(capacity, Ordering::Release);
        self.budget.peak_bytes.fetch_max(capacity, Ordering::AcqRel);
    }
}

impl Drop for CloseAutosavePayloadLease {
    fn drop(&mut self) {
        self.budget.retained_bytes.store(0, Ordering::Release);
        self.budget.occupied.store(false, Ordering::Release);
    }
}

struct ClosingAutosaveSource {
    instance: crate::ui::workbench::view::ViewInstanceId,
    revision: crate::core::editor_event::DocumentCloseRevision,
    operation: super::ProjectCloseOperation,
    scene: crate::core::editing::engine::SceneCloseAdmission,
    documents: crate::core::extension::DocumentCloseAdmission,
    payloads: Arc<CloseAutosavePayloadBudget>,
}

struct EditorDocumentAutosaveSource {
    manager: ServiceHandle<EditorManager>,
    document: DocumentId,
    dirty_generation: u64,
    sequence: u64,
    source_path: AutosaveSourcePath,
    physical_source_path: PathBuf,
    close: Option<ClosingAutosaveSource>,
}

impl AutosaveSnapshotSource for EditorDocumentAutosaveSource {
    fn source_path(&self) -> AutosaveSourcePath {
        self.source_path.clone()
    }

    fn capture(&self, _document: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
        let manager = self.manager.enter().map_err(JobError::failed)?;
        let payload_lease = self
            .close
            .as_ref()
            .map(|close| close.payloads.acquire())
            .transpose()?;
        let payload = if let Some(close) = self.close.as_ref() {
            manager
                .validate_project_close_operation(
                    &close.operation,
                    crate::core::recovery::ProjectSessionEffect::DirtyDocuments,
                )
                .map_err(JobError::failed)?;
            manager
                .context()
                .transactions()
                .validate_close_admission(&close.scene)
                .map_err(JobError::failed)?;
            manager
                .host
                .document_toolkits
                .validate_close_admission(&close.documents)
                .map_err(JobError::failed)?;
            let (_, current) = manager
                .host
                .document_close_state(&close.instance, self.document)
                .map_err(JobError::failed)?;
            if current != close.revision {
                return Err(JobError::failed(std::io::Error::other(
                    "closing document revision changed before capture",
                )));
            }
            let payload = manager
                .host
                .document_toolkits
                .capture_autosave_for_close(self.document, &manager.host, &close.documents)
                .map_err(JobError::failed)?;
            let (_, current) = manager
                .host
                .document_close_state(&close.instance, self.document)
                .map_err(JobError::failed)?;
            if current != close.revision {
                return Err(JobError::failed(std::io::Error::other(
                    "closing document revision changed during capture",
                )));
            }
            manager
                .validate_project_close_operation(
                    &close.operation,
                    crate::core::recovery::ProjectSessionEffect::DirtyDocuments,
                )
                .map_err(JobError::failed)?;
            payload
        } else {
            manager
                .host
                .capture_document_autosave(self.document, self.dirty_generation)
                .map_err(JobError::failed)?
        };
        if let Some(lease) = payload_lease.as_ref() {
            lease.account(payload.retained_capacity());
        }
        drop(manager);
        let captured_source_path = ProjectPaths::resolve_path(payload.source_path())
            .map(|path| path.into_operation_path())
            .unwrap_or_else(|_| payload.source_path().to_path_buf());
        let extension = payload
            .source_path()
            .extension()
            .and_then(|value| value.to_str())
            .ok_or_else(|| {
                JobError::failed(std::io::Error::other(format!(
                    "autosave source {} has no UTF-8 extension",
                    payload.source_path().display()
                )))
            })?;
        let extension = AutosaveExtension::parse(extension).map_err(JobError::failed)?;
        if captured_source_path != self.physical_source_path {
            return Err(JobError::failed(std::io::Error::other(
                "autosave source identity changed after admission",
            )));
        }
        let source_digest =
            AutosaveSourceDigest::observe(&self.physical_source_path).map_err(JobError::failed)?;
        let bytes = payload.into_bytes();
        let snapshot = AutosaveSnapshot::new(
            self.sequence,
            extension,
            self.source_path.clone(),
            AutosaveSnapshotProvenance::capture(self.dirty_generation, source_digest),
            bytes,
        );
        Ok(if let Some(lease) = payload_lease {
            snapshot.with_retained_payload_owner(Arc::new(lease))
        } else {
            snapshot
        })
    }
}

pub(super) struct ForegroundDocumentSaveJob {
    manager: ServiceHandle<EditorManager>,
    instance_id: crate::ui::workbench::view::ViewInstanceId,
    reason: SaveReason,
}

impl ForegroundDocumentSaveJob {
    pub(super) fn new(
        manager: ServiceHandle<EditorManager>,
        instance_id: crate::ui::workbench::view::ViewInstanceId,
        reason: SaveReason,
    ) -> Self {
        Self {
            manager,
            instance_id,
            reason,
        }
    }

    pub(super) fn spec(document: DocumentId, mutex: MutexGroup) -> EditorJobSpec {
        EditorJobSpec::new(
            format!("save_document_{}", document.value()),
            JobCategory::InteractiveSave,
        )
        .with_priority(JobPriority::Interactive)
        .with_mutex_group(mutex)
        .with_estimated_bytes(std::mem::size_of::<Self>().max(1))
    }
}

impl EditorJob for ForegroundDocumentSaveJob {
    type Output = DocumentSaveReport;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let manager = self.manager.enter().map_err(JobError::failed)?;
        manager
            .host
            .save_document_toolkit_canonical(&self.instance_id, self.reason)
            .map_err(JobError::failed)
    }
}

fn next_autosave_sequence() -> u64 {
    static LAST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let wall_clock = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX))
        .unwrap_or(1);
    let mut previous = LAST_SEQUENCE.load(Ordering::Acquire);
    loop {
        let next = wall_clock.max(previous.saturating_add(1));
        match LAST_SEQUENCE.compare_exchange_weak(
            previous,
            next,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return next,
            Err(current) => previous = current,
        }
    }
}

#[cfg(test)]
#[path = "tests/editor_document_autosave.rs"]
mod tests;
