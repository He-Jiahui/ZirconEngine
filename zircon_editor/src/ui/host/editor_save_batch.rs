use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use thiserror::Error;

use crate::core::asset::{
    SaveDirtyViewCandidate, SaveDirtyViewCompletion, SaveDirtyViewExecutor, SaveDirtyViewFailure,
    SaveDirtyViewFailureKind, SaveDirtyViewsAdmissionError, SaveDirtyViewsApplyError,
    SaveDirtyViewsJobAdapter, SaveDirtyViewsPreflightReport, SaveDirtyViewsRequest,
    SaveDirtyViewsResult,
};
use crate::core::editor_message::DocumentId;
use crate::core::extension::SaveReason;

use super::editor_document_autosave::document_save_mutex_group;
use super::runtime_services::EditorHostRuntimeServices;
use super::{EditorError, EditorManager};

#[derive(Debug, Error)]
pub enum EditorDirtySaveError {
    #[error("dirty save preflight rejected {rejected_documents} document(s)")]
    Preflight { rejected_documents: usize },
    #[error("failed to inspect dirty document {document:?} source {path}: {source}")]
    SourceMetadata {
        document: DocumentId,
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("dirty save adapter completed without an owned request")]
    MissingRequest,
    #[error("dirty save completion is owned by {expected}, not {received}")]
    OwnerMismatch {
        expected: &'static str,
        received: &'static str,
    },
    #[error(transparent)]
    Admission(#[from] SaveDirtyViewsAdmissionError),
    #[error(transparent)]
    Apply(#[from] SaveDirtyViewsApplyError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DirtyDocumentSaveStart {
    NoDirtyDocuments,
    Scheduled,
    Busy { owner: DirtyDocumentSaveOwner },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DirtyDocumentSaveOwner {
    SaveAll,
    ClosePrompt,
}

impl std::fmt::Display for DirtyDocumentSaveOwner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}

impl DirtyDocumentSaveOwner {
    const fn label(self) -> &'static str {
        match self {
            Self::SaveAll => "Save All",
            Self::ClosePrompt => "the close prompt",
        }
    }
}

#[derive(Default)]
struct DirtyDocumentSaveOwnership {
    owner: Option<DirtyDocumentSaveOwner>,
}

impl DirtyDocumentSaveOwnership {
    fn owner(&self) -> Option<DirtyDocumentSaveOwner> {
        self.owner
    }

    fn try_acquire(&mut self, requested: DirtyDocumentSaveOwner) -> DirtyDocumentSaveStart {
        match self.owner {
            Some(owner) => DirtyDocumentSaveStart::Busy { owner },
            None => {
                self.owner = Some(requested);
                DirtyDocumentSaveStart::Scheduled
            }
        }
    }

    fn release(&mut self, received: DirtyDocumentSaveOwner) -> Result<(), EditorDirtySaveError> {
        match self.owner {
            Some(expected) if expected == received => {
                self.owner = None;
                Ok(())
            }
            Some(expected) => Err(EditorDirtySaveError::OwnerMismatch {
                expected: expected.label(),
                received: received.label(),
            }),
            None => Err(EditorDirtySaveError::MissingRequest),
        }
    }
}

/// The sole product coordinator for a document dirty-save batch.
///
/// It owns only admission and completion state. Document serialization stays in
/// the registered toolkit, and the DirtyRegistry applies the save-token result.
pub(super) struct EditorDirtySaveCoordinator {
    adapter: SaveDirtyViewsJobAdapter,
    ownership: DirtyDocumentSaveOwnership,
    pending_request: Option<SaveDirtyViewsRequest>,
}

impl EditorDirtySaveCoordinator {
    pub(super) fn new(jobs: crate::core::jobs::EditorJobSystem) -> Self {
        Self {
            adapter: SaveDirtyViewsJobAdapter::new(jobs),
            ownership: DirtyDocumentSaveOwnership::default(),
            pending_request: None,
        }
    }

    fn owner(&self) -> Option<DirtyDocumentSaveOwner> {
        self.ownership.owner()
    }

    fn schedule(
        &mut self,
        owner: DirtyDocumentSaveOwner,
        request: SaveDirtyViewsRequest,
        executor: Arc<dyn SaveDirtyViewExecutor>,
    ) -> Result<DirtyDocumentSaveStart, EditorDirtySaveError> {
        let admission = self.ownership.try_acquire(owner);
        if matches!(admission, DirtyDocumentSaveStart::Busy { .. }) {
            return Ok(admission);
        }
        let scheduled = match self.adapter.schedule(
            &request,
            |intent| {
                document_save_mutex_group(Path::new(intent.resource_key()))
                    .map_err(|error| error.to_string())
            },
            || executor,
        ) {
            Ok(scheduled) => scheduled,
            Err(error) => {
                self.ownership.release(owner)?;
                return Err(error.into());
            }
        };
        if scheduled {
            self.pending_request = Some(request);
            Ok(DirtyDocumentSaveStart::Scheduled)
        } else {
            self.ownership.release(owner)?;
            Ok(DirtyDocumentSaveStart::NoDirtyDocuments)
        }
    }

    fn poll(
        &mut self,
        owner: DirtyDocumentSaveOwner,
        dirty: &crate::core::asset::DirtyRegistry,
        transactions: &crate::core::editing::engine::EditorTransactionEngine,
    ) -> Result<Option<SaveDirtyViewsResult>, EditorDirtySaveError> {
        match self.ownership.owner() {
            Some(expected) if expected != owner => {
                return Err(EditorDirtySaveError::OwnerMismatch {
                    expected: expected.label(),
                    received: owner.label(),
                });
            }
            Some(_) => {}
            None => return Err(EditorDirtySaveError::MissingRequest),
        }
        let Some(completions) = self.adapter.pump_completed().into_completed() else {
            return Ok(None);
        };
        let request = self
            .pending_request
            .take()
            .ok_or(EditorDirtySaveError::MissingRequest)?;
        self.ownership.release(owner)?;
        Ok(Some(request.apply_completions(
            completions.into_completions(),
            dirty,
            transactions,
        )?))
    }
}

impl EditorManager {
    pub(crate) fn begin_dirty_document_save(
        &self,
        owner: DirtyDocumentSaveOwner,
        documents: impl IntoIterator<Item = DocumentId>,
        reason: SaveReason,
    ) -> Result<DirtyDocumentSaveStart, EditorError> {
        {
            let coordinator = self
                .dirty_save
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(owner) = coordinator.owner() {
                return Ok(DirtyDocumentSaveStart::Busy { owner });
            }
        }
        let documents = documents.into_iter().collect::<BTreeSet<_>>();
        let Some(request) = self.prepare_dirty_document_save_request(&documents)? else {
            return Ok(DirtyDocumentSaveStart::NoDirtyDocuments);
        };

        let executor = Arc::new(EditorDocumentBatchSaveExecutor {
            runtime_services: self.host.runtime_services.clone(),
            reason,
        });
        let mut coordinator = self
            .dirty_save
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        coordinator
            .schedule(owner, request, executor)
            .map_err(EditorError::from)
    }

    pub(crate) fn poll_dirty_document_save(
        &self,
        owner: DirtyDocumentSaveOwner,
    ) -> Result<Option<SaveDirtyViewsResult>, EditorError> {
        let result = {
            let mut coordinator = self
                .dirty_save
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            coordinator.poll(
                owner,
                self.context().dirty_documents(),
                self.context().transactions(),
            )?
        };
        if let Some(result) = result.as_ref() {
            for document in result
                .outcomes()
                .iter()
                .map(|outcome| outcome.document_id())
            {
                self.host
                    .sync_document_dirty_projection_for_document(document)?;
            }
        }
        Ok(result)
    }

    pub(crate) fn dirty_document_save_owner(&self) -> Option<DirtyDocumentSaveOwner> {
        self.dirty_save
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .owner()
    }

    fn prepare_dirty_document_save_request(
        &self,
        documents: &BTreeSet<DocumentId>,
    ) -> Result<Option<SaveDirtyViewsRequest>, EditorError> {
        let toolkit_snapshot = self.host.document_toolkit_snapshot();
        let descriptors = toolkit_snapshot
            .descriptors()
            .iter()
            .map(|descriptor| (descriptor.document_id(), descriptor))
            .collect::<BTreeMap<_, _>>();
        let dirty = self.context().dirty_documents();
        let mut candidates = Vec::new();

        for document in documents {
            let Some(descriptor) = descriptors.get(document) else {
                continue;
            };
            let snapshot = dirty.snapshot(*document)?;
            if !snapshot.is_dirty() {
                continue;
            }
            let source_path = self.host.document_autosave_source_path(*document)?;
            let metadata = fs::metadata(&source_path).map_err(|source| {
                EditorDirtySaveError::SourceMetadata {
                    document: *document,
                    path: source_path.clone(),
                    source,
                }
            })?;
            let resource_key = source_path.to_string_lossy().into_owned();
            let token = dirty.capture_save_token(*document)?;
            let references_valid = self
                .host
                .validate_document_toolkit_references(*document)
                .is_ok();
            candidates.push(
                SaveDirtyViewCandidate::new(
                    snapshot,
                    descriptor.instance_id().clone(),
                    token,
                    resource_key,
                    metadata.len(),
                )
                .with_writable(!metadata.permissions().readonly())
                .with_references_valid(references_valid),
            );
        }

        if candidates.is_empty() {
            return Ok(None);
        }
        let request =
            SaveDirtyViewsRequest::prepare(&toolkit_snapshot, candidates).map_err(|report| {
                EditorDirtySaveError::Preflight {
                    rejected_documents: rejected_document_count(&report),
                }
            })?;
        Ok(Some(request))
    }
}

fn rejected_document_count(report: &SaveDirtyViewsPreflightReport) -> usize {
    report.failures().len()
}

struct EditorDocumentBatchSaveExecutor {
    runtime_services: EditorHostRuntimeServices,
    reason: SaveReason,
}

impl SaveDirtyViewExecutor for EditorDocumentBatchSaveExecutor {
    fn save(
        &self,
        intent: &crate::core::asset::SaveDirtyViewIntent,
        context: &crate::core::jobs::JobContext,
    ) -> SaveDirtyViewCompletion {
        if context.check_cancelled().is_err() {
            return SaveDirtyViewCompletion::Cancelled;
        }
        let completion = self.runtime_services.editor_manager().and_then(|manager| {
            manager
                .host
                .write_document_toolkit(intent.document_id(), self.reason)
        });
        match completion {
            Ok(report) => SaveDirtyViewCompletion::Saved {
                written_bytes: report.written_bytes(),
            },
            Err(error) => SaveDirtyViewCompletion::Failed(SaveDirtyViewFailure::new(
                SaveDirtyViewFailureKind::Write,
                error.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use crate::core::asset::{
        DirtyExternalEffectId, DirtyRegistry, SaveDirtyViewCandidate, SaveDirtyViewCompletion,
        SaveDirtyViewExecutor, SaveDirtyViewsRequest,
    };
    use crate::core::editing::engine::{
        EditCommandError, EditContext, EditorTransactionEngine, SelectionSnapshot,
    };
    use crate::core::editor_message::DocumentId;
    use crate::core::extension::{
        DocumentAutosavePayload, DocumentToolkit, DocumentToolkitDescriptor,
        DocumentToolkitRegistry, SaveCtx, ToolkitInstanceId, ToolkitLayout, ToolkitSaveFailure,
    };
    use crate::core::gateway::EditorRuntimeGatewayHandle;
    use crate::core::jobs::{test_job_system, JobContext};

    use super::{
        DirtyDocumentSaveOwner, DirtyDocumentSaveOwnership, DirtyDocumentSaveStart,
        EditorDirtySaveCoordinator, EditorDirtySaveError,
    };

    #[test]
    fn dirty_save_owner_rejects_competitors_until_the_exact_owner_releases() {
        let mut ownership = DirtyDocumentSaveOwnership::default();

        assert_eq!(
            ownership.try_acquire(DirtyDocumentSaveOwner::ClosePrompt),
            DirtyDocumentSaveStart::Scheduled
        );
        assert_eq!(
            ownership.try_acquire(DirtyDocumentSaveOwner::SaveAll),
            DirtyDocumentSaveStart::Busy {
                owner: DirtyDocumentSaveOwner::ClosePrompt,
            }
        );
        assert!(matches!(
            ownership.release(DirtyDocumentSaveOwner::SaveAll),
            Err(EditorDirtySaveError::OwnerMismatch {
                expected: "the close prompt",
                received: "Save All",
            })
        ));
        assert_eq!(ownership.owner(), Some(DirtyDocumentSaveOwner::ClosePrompt));

        ownership
            .release(DirtyDocumentSaveOwner::ClosePrompt)
            .unwrap();
        assert_eq!(
            ownership.try_acquire(DirtyDocumentSaveOwner::SaveAll),
            DirtyDocumentSaveStart::Scheduled
        );
    }

    #[test]
    fn wrong_owner_poll_cannot_consume_a_real_completed_save_batch() {
        let transactions = Arc::new(EditorTransactionEngine::new(TestEditContext::default()));
        let dirty = DirtyRegistry::new(Arc::clone(&transactions));
        let document = DocumentId::new(1);
        let instance = ToolkitInstanceId::parse("view.asset.owner_poll").unwrap();
        dirty.register_document(document).unwrap();
        dirty
            .mark_external_effect(
                document,
                DirtyExternalEffectId::parse("ui.owner_poll").unwrap(),
            )
            .unwrap();
        let toolkits = DocumentToolkitRegistry::<()>::default();
        toolkits
            .register(Arc::new(TestToolkit {
                descriptor: DocumentToolkitDescriptor::new(
                    document,
                    instance.clone(),
                    "Owner poll fixture",
                    ToolkitLayout::single_tab("layout.owner_poll", "tab.owner_poll").unwrap(),
                ),
            }))
            .unwrap();
        let request = SaveDirtyViewsRequest::prepare(
            &toolkits.snapshot(),
            [SaveDirtyViewCandidate::new(
                dirty.snapshot(document).unwrap(),
                instance,
                dirty.capture_save_token(document).unwrap(),
                "E:/ZirconEngineTests/owner-poll.zdoc",
                1,
            )],
        )
        .unwrap();
        let executor: Arc<dyn SaveDirtyViewExecutor> = Arc::new(
            |_: &crate::core::asset::SaveDirtyViewIntent, _: &JobContext| {
                SaveDirtyViewCompletion::Saved { written_bytes: 1 }
            },
        );
        let mut coordinator = EditorDirtySaveCoordinator::new(test_job_system());
        assert_eq!(
            coordinator
                .schedule(DirtyDocumentSaveOwner::ClosePrompt, request, executor)
                .unwrap(),
            DirtyDocumentSaveStart::Scheduled
        );

        assert!(matches!(
            coordinator.poll(
                DirtyDocumentSaveOwner::SaveAll,
                &dirty,
                transactions.as_ref(),
            ),
            Err(EditorDirtySaveError::OwnerMismatch {
                expected: "the close prompt",
                received: "Save All",
            })
        ));
        assert_eq!(
            coordinator.owner(),
            Some(DirtyDocumentSaveOwner::ClosePrompt)
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match coordinator
                .poll(
                    DirtyDocumentSaveOwner::ClosePrompt,
                    &dirty,
                    transactions.as_ref(),
                )
                .unwrap()
            {
                Some(result) => {
                    assert!(result.all_saved());
                    break;
                }
                None if Instant::now() < deadline => std::thread::yield_now(),
                None => panic!("owned save batch did not terminalize"),
            }
        }
        assert_eq!(coordinator.owner(), None);
    }

    struct TestEditContext {
        gateway: EditorRuntimeGatewayHandle,
    }

    impl Default for TestEditContext {
        fn default() -> Self {
            Self {
                gateway: EditorRuntimeGatewayHandle::detached(),
            }
        }
    }

    impl EditContext for TestEditContext {
        fn runtime_gateway(&self) -> &EditorRuntimeGatewayHandle {
            &self.gateway
        }

        fn selection_snapshot(&self) -> SelectionSnapshot {
            SelectionSnapshot::default()
        }

        fn restore_selection(
            &mut self,
            _snapshot: &SelectionSnapshot,
        ) -> Result<(), EditCommandError> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    struct TestToolkit {
        descriptor: DocumentToolkitDescriptor,
    }

    impl DocumentToolkit<()> for TestToolkit {
        fn descriptor(&self) -> &DocumentToolkitDescriptor {
            &self.descriptor
        }

        fn save(&self, _host: &(), _context: &mut SaveCtx) -> Result<(), ToolkitSaveFailure> {
            Ok(())
        }

        fn autosave_source_path(
            &self,
            _host: &(),
        ) -> Result<std::path::PathBuf, ToolkitSaveFailure> {
            Ok("E:/ZirconEngineTests/owner-poll.zdoc".into())
        }

        fn capture_autosave(
            &self,
            _host: &(),
        ) -> Result<DocumentAutosavePayload, ToolkitSaveFailure> {
            Ok(DocumentAutosavePayload::new(
                "E:/ZirconEngineTests/owner-poll.zdoc",
                Vec::new(),
            ))
        }
    }
}
